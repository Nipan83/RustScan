//! Collect structured matches by combining traversal and per-file matching.
//!
//! Parallel collection walks directories and searches files with Rayon. A single
//! compiled [`Regex`] is shared by reference across worker threads (no
//! per-thread recompilation). Results are gathered via Rayon's lock-free
//! `collect` into per-worker vectors, then sorted for deterministic output.

use crate::matching;
use crate::result::{FileMatches, SearchResults};
use crate::traverse::{
    TraverseOptions, collect_file_paths, collect_file_paths_parallel, walk_files,
};
use rayon::prelude::*;
use regex::Regex;
use std::path::Path;

/// Walk `root`, match each regular file against `regex`, and collect hits.
///
/// Uses the **parallel** implementation by default (Rayon traversal + search).
/// Prefer [`collect_matches_sequential`] when comparing modes or debugging.
///
/// Files that cannot be read as UTF-8 (or fail to open) produce a warning on
/// stderr and are skipped. Only files with at least one matching line appear
/// in the returned [`SearchResults`]. File entries are sorted by path.
pub fn collect_matches(root: &Path, regex: &Regex, include_hidden: bool) -> SearchResults {
    collect_matches_parallel(root, regex, include_hidden)
}

/// Sequential traversal and sequential per-file matching (baseline / tests).
pub fn collect_matches_sequential(
    root: &Path,
    regex: &Regex,
    include_hidden: bool,
) -> SearchResults {
    let mut results = SearchResults::new();
    let options = TraverseOptions { include_hidden };

    let _ = walk_files(root, options, |path| {
        match matching::match_file(path, regex) {
            Ok(hits) if !hits.is_empty() => {
                results.push(FileMatches {
                    path: path.to_path_buf(),
                    hits,
                });
            }
            Ok(_) => {}
            Err(err) => {
                eprintln!("warning: skipping {}: {}", path.display(), err);
            }
        }
    });

    results.sort_deterministic();
    results
}

/// Parallel directory walk and parallel per-file matching with a shared regex.
///
/// 1. Discover paths with [`collect_file_paths_parallel`] (no shared lock).
/// 2. Search files with `par_iter`, borrowing the same `regex` on every worker.
/// 3. `filter_map` + Rayon `collect` builds a `Vec<FileMatches>` without a
///    global mutex on the hot path.
/// 4. Sort by path so formatted output matches the sequential baseline.
pub fn collect_matches_parallel(root: &Path, regex: &Regex, include_hidden: bool) -> SearchResults {
    let options = TraverseOptions { include_hidden };
    let paths = collect_file_paths_parallel(root, options);

    // `Regex` is `Sync`; one compiled automaton is reused by all threads.
    let files: Vec<FileMatches> = paths
        .into_par_iter()
        .filter_map(|path| match matching::match_file(&path, regex) {
            Ok(hits) if !hits.is_empty() => Some(FileMatches { path, hits }),
            Ok(_) => None,
            Err(err) => {
                eprintln!("warning: skipping {}: {}", path.display(), err);
                None
            }
        })
        .collect();

    let mut results = SearchResults { files };
    results.sort_deterministic();
    results
}

/// Sequential path discovery + parallel file matching (hybrid; useful in benches).
#[allow(dead_code)]
pub fn collect_matches_parallel_search_only(
    root: &Path,
    regex: &Regex,
    include_hidden: bool,
) -> SearchResults {
    let options = TraverseOptions { include_hidden };
    let paths = collect_file_paths(root, options);

    let files: Vec<FileMatches> = paths
        .into_par_iter()
        .filter_map(|path| match matching::match_file(&path, regex) {
            Ok(hits) if !hits.is_empty() => Some(FileMatches { path, hits }),
            Ok(_) => None,
            Err(err) => {
                eprintln!("warning: skipping {}: {}", path.display(), err);
                None
            }
        })
        .collect();

    let mut results = SearchResults { files };
    results.sort_deterministic();
    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pattern::compile_pattern;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn collects_only_files_with_hits() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("hit.txt"), "hello\n").unwrap();
        fs::write(dir.path().join("miss.txt"), "nope\n").unwrap();

        let re = compile_pattern("hello", false).unwrap();
        let results = collect_matches(dir.path(), &re, false);
        assert_eq!(results.files.len(), 1);
        assert!(results.files[0].path.ends_with("hit.txt"));
        assert_eq!(results.files[0].match_count(), 1);
    }

    #[test]
    fn respects_hidden_flag() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join(".secret"), "hello\n").unwrap();
        fs::write(dir.path().join("vis.txt"), "hello\n").unwrap();

        let re = compile_pattern("hello", false).unwrap();
        assert_eq!(collect_matches(dir.path(), &re, false).files.len(), 1);
        assert_eq!(collect_matches(dir.path(), &re, true).files.len(), 2);
    }

    #[test]
    fn sequential_and_parallel_agree() {
        let dir = tempdir().unwrap();
        for i in 0..20 {
            let name = format!("f{i}.txt");
            let body = if i % 3 == 0 { "needle\n" } else { "hay\n" };
            fs::write(dir.path().join(name), body).unwrap();
        }
        let re = compile_pattern("needle", false).unwrap();
        let seq = collect_matches_sequential(dir.path(), &re, false);
        let par = collect_matches_parallel(dir.path(), &re, false);
        assert_eq!(seq, par);
    }
}
