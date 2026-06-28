//! Search logic for matching patterns against file contents.
//!
//! This milestone implements recursive filesystem traversal only.
//! Pattern matching will be added in a later step.

use std::fs;
use std::io;
use std::path::Path;
use std::process;

/// Run a search for `pattern` under `path`.
///
/// Recursively walks `path` and prints every regular file found.
/// Directories that cannot be read are skipped with a warning.
/// Pattern matching is not performed yet; `pattern` is reserved for later use.
pub fn run_search(_pattern: &str, path: &Path) {
    if !path.exists() {
        eprintln!("error: path does not exist: {}", path.display());
        process::exit(1);
    }

    if !path.is_dir() {
        eprintln!("error: not a directory: {}", path.display());
        process::exit(1);
    }

    let mut count = 0usize;
    walk_dir(path, &mut count);
    println!("\nFound {} file(s)", count);
}

/// Recursively visit `dir`, printing regular files and counting them.
fn walk_dir(dir: &Path, count: &mut usize) {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(err) => {
            warn_skip(dir, &err);
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
            walk_dir(&path, count);
        } else if file_type.is_file() {
            println!("{}", path.display());
            *count += 1;
        }
        // Symlinks and other special files are ignored for now.
    }
}

fn warn_skip(dir: &Path, err: &io::Error) {
    eprintln!("warning: skipping directory {}: {}", dir.display(), err);
}
