//! Filesystem traversal: recursively discover regular files to search.
//!
//! Supports sequential and Rayon-parallel directory walks. Parallel mode
//! descends into subdirectories concurrently; file paths are returned in an
//! unspecified order (callers sort results after matching for determinism).

use crate::utils::is_hidden;
use rayon::prelude::*;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Options controlling which paths are visited.
#[derive(Debug, Clone, Copy, Default)]
pub struct TraverseOptions {
    /// When false (default), skip entries whose names start with `.` and do
    /// not descend into hidden directories.
    pub include_hidden: bool,
}

/// Recursively walk `root` and invoke `visit` for every regular file found (sequential).
///
/// Unreadable directories and entries emit a warning on stderr and are skipped
/// so the walk can continue. Symlinks and non-regular files are ignored.
pub fn walk_files<F>(root: &Path, options: TraverseOptions, mut visit: F) -> io::Result<()>
where
    F: FnMut(&Path),
{
    walk_dir_sequential(root, options, &mut visit);
    Ok(())
}

/// Collect all regular file paths under `root` (sequential).
pub fn collect_file_paths(root: &Path, options: TraverseOptions) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let _ = walk_files(root, options, |p| paths.push(p.to_path_buf()));
    paths
}

/// Collect all regular file paths under `root`, walking subdirectories in parallel.
///
/// Uses Rayon to process sibling directories concurrently. Does not take a lock
/// per file; each recursive call returns its own `Vec` which is concatenated.
pub fn collect_file_paths_parallel(root: &Path, options: TraverseOptions) -> Vec<PathBuf> {
    walk_dir_parallel(root, options)
}

fn walk_dir_sequential<F>(dir: &Path, options: TraverseOptions, visit: &mut F)
where
    F: FnMut(&Path),
{
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(err) => {
            eprintln!("warning: skipping directory {}: {}", dir.display(), err);
            return;
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(err) => {
                eprintln!("warning: skipping entry under {}: {}", dir.display(), err);
                continue;
            }
        };

        let path = entry.path();

        if !options.include_hidden && is_hidden(&path) {
            continue;
        }

        let file_type = match entry.file_type() {
            Ok(ft) => ft,
            Err(err) => {
                eprintln!(
                    "warning: skipping {}: could not determine type: {}",
                    path.display(),
                    err
                );
                continue;
            }
        };

        if file_type.is_dir() {
            walk_dir_sequential(&path, options, visit);
        } else if file_type.is_file() {
            visit(&path);
        }
    }
}

/// Parallel recursive walk: partition entries into files and subdirs, recurse
/// on subdirs with `par_iter`, then concatenate without shared mutable state.
fn walk_dir_parallel(dir: &Path, options: TraverseOptions) -> Vec<PathBuf> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(err) => {
            eprintln!("warning: skipping directory {}: {}", dir.display(), err);
            return Vec::new();
        }
    };

    let mut files = Vec::new();
    let mut dirs = Vec::new();

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(err) => {
                eprintln!("warning: skipping entry under {}: {}", dir.display(), err);
                continue;
            }
        };

        let path = entry.path();

        if !options.include_hidden && is_hidden(&path) {
            continue;
        }

        let file_type = match entry.file_type() {
            Ok(ft) => ft,
            Err(err) => {
                eprintln!(
                    "warning: skipping {}: could not determine type: {}",
                    path.display(),
                    err
                );
                continue;
            }
        };

        if file_type.is_dir() {
            dirs.push(path);
        } else if file_type.is_file() {
            files.push(path);
        }
    }

    let nested: Vec<PathBuf> = dirs
        .into_par_iter()
        .flat_map(|sub| walk_dir_parallel(&sub, options))
        .collect();

    files.extend(nested);
    files
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn walks_nested_visible_files() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("a/b")).unwrap();
        fs::write(dir.path().join("root.txt"), "x").unwrap();
        fs::write(dir.path().join("a/mid.txt"), "x").unwrap();
        fs::write(dir.path().join("a/b/leaf.txt"), "x").unwrap();

        let mut paths = collect_file_paths(dir.path(), TraverseOptions::default());
        paths.sort();
        assert_eq!(paths.len(), 3);
    }

    #[test]
    fn parallel_walk_finds_same_files() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("a/b")).unwrap();
        fs::write(dir.path().join("root.txt"), "x").unwrap();
        fs::write(dir.path().join("a/mid.txt"), "x").unwrap();
        fs::write(dir.path().join("a/b/leaf.txt"), "x").unwrap();

        let mut seq = collect_file_paths(dir.path(), TraverseOptions::default());
        let mut par = collect_file_paths_parallel(dir.path(), TraverseOptions::default());
        seq.sort();
        par.sort();
        assert_eq!(seq, par);
    }

    #[test]
    fn skips_hidden_by_default() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("visible.txt"), "x").unwrap();
        fs::write(dir.path().join(".secret"), "x").unwrap();
        fs::create_dir_all(dir.path().join(".hid")).unwrap();
        fs::write(dir.path().join(".hid/inside.txt"), "x").unwrap();

        let paths = collect_file_paths(dir.path(), TraverseOptions::default());
        assert_eq!(paths.len(), 1);
        assert!(paths[0].ends_with("visible.txt"));

        let par = collect_file_paths_parallel(dir.path(), TraverseOptions::default());
        assert_eq!(par.len(), 1);
    }

    #[test]
    fn includes_hidden_when_requested() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("visible.txt"), "x").unwrap();
        fs::write(dir.path().join(".secret"), "x").unwrap();

        let paths = collect_file_paths(
            dir.path(),
            TraverseOptions {
                include_hidden: true,
            },
        );
        assert_eq!(paths.len(), 2);
    }
}
