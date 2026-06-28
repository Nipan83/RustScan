//! Search logic for matching patterns against file contents.
//!
//! Recursively walks a directory tree and performs a case-sensitive
//! substring search in each regular file.

use std::fs;
use std::io;
use std::path::Path;
use std::process;

/// Run a search for `pattern` under `path`.
///
/// Recursively walks `path` and searches each regular file for lines
/// containing `pattern` as a case-sensitive substring. Directories that
/// cannot be read are skipped with a warning; unreadable or non-UTF-8
/// files are likewise warned about and skipped.
pub fn run_search(pattern: &str, path: &Path) {
    if !path.exists() {
        eprintln!("error: path does not exist: {}", path.display());
        process::exit(1);
    }

    if !path.is_dir() {
        eprintln!("error: not a directory: {}", path.display());
        process::exit(1);
    }

    walk_dir(path, pattern);
}

/// Recursively visit `dir` and search each regular file for `pattern`.
fn walk_dir(dir: &Path, pattern: &str) {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(err) => {
            warn_skip_dir(dir, &err);
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
            walk_dir(&path, pattern);
        } else if file_type.is_file() {
            search_file(&path, pattern);
        }
        // Symlinks and other special files are ignored for now.
    }
}

/// Open `path` as UTF-8 text and print lines containing `pattern`.
///
/// Output format for each match: `path:line_number:line_contents`
/// (1-based line numbers). Prints nothing if there are no matches.
fn search_file(path: &Path, pattern: &str) {
    let contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(err) => {
            eprintln!("warning: skipping {}: {}", path.display(), err);
            return;
        }
    };

    for (idx, line) in contents.lines().enumerate() {
        if line.contains(pattern) {
            println!("{}:{}:{}", path.display(), idx + 1, line);
        }
    }
}

fn warn_skip_dir(dir: &Path, err: &io::Error) {
    eprintln!("warning: skipping directory {}: {}", dir.display(), err);
}
