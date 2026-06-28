//! Search logic for matching patterns against file contents.
//!
//! Recursively walks a directory tree and performs a substring search
//! in each regular file (case-sensitive by default).

use std::fs;
use std::io;
use std::path::Path;
use std::process;

/// Run a search for `pattern` under `path`.
///
/// Recursively walks `path` and searches each regular file for lines
/// containing `pattern` as a substring. When `ignore_case` is true, the
/// comparison is case-insensitive; otherwise it is case-sensitive.
/// When `show_line_number` is true, matches are printed as
/// `path:line_number:line_contents`; otherwise as `path:line_contents`.
/// Directories that cannot be read are skipped with a warning; unreadable
/// or non-UTF-8 files are likewise warned about and skipped.
pub fn run_search(pattern: &str, path: &Path, ignore_case: bool, show_line_number: bool) {
    if !path.exists() {
        eprintln!("error: path does not exist: {}", path.display());
        process::exit(1);
    }

    if !path.is_dir() {
        eprintln!("error: not a directory: {}", path.display());
        process::exit(1);
    }

    walk_dir(path, pattern, ignore_case, show_line_number);
}

/// Recursively visit `dir` and search each regular file for `pattern`.
fn walk_dir(dir: &Path, pattern: &str, ignore_case: bool, show_line_number: bool) {
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
            walk_dir(&path, pattern, ignore_case, show_line_number);
        } else if file_type.is_file() {
            search_file(&path, pattern, ignore_case, show_line_number);
        }
        // Symlinks and other special files are ignored for now.
    }
}

/// Open `path` as UTF-8 text and print lines containing `pattern`.
///
/// Prints nothing if there are no matches.
fn search_file(path: &Path, pattern: &str, ignore_case: bool, show_line_number: bool) {
    let contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(err) => {
            eprintln!("warning: skipping {}: {}", path.display(), err);
            return;
        }
    };

    // Lowercase the pattern once when ignoring case so the per-line path stays shared.
    let pattern_lower = ignore_case.then(|| pattern.to_lowercase());

    for (idx, line) in contents.lines().enumerate() {
        if line_matches(line, pattern, pattern_lower.as_deref()) {
            print_match(path, idx + 1, line, show_line_number);
        }
    }
}

/// Print a single match using one formatting path controlled by `show_line_number`.
fn print_match(path: &Path, line_number: usize, line: &str, show_line_number: bool) {
    if show_line_number {
        println!("{}:{}:{}", path.display(), line_number, line);
    } else {
        println!("{}:{}", path.display(), line);
    }
}

/// Return true if `line` contains `pattern`.
///
/// When `pattern_lower` is `Some`, comparison is case-insensitive using that
/// pre-lowercased needle; otherwise the match is case-sensitive.
fn line_matches(line: &str, pattern: &str, pattern_lower: Option<&str>) -> bool {
    match pattern_lower {
        Some(needle) => line.to_lowercase().contains(needle),
        None => line.contains(pattern),
    }
}

fn warn_skip_dir(dir: &Path, err: &io::Error) {
    eprintln!("warning: skipping directory {}: {}", dir.display(), err);
}
