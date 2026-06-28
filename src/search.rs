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
///
/// Output modes:
/// - `count_only`: print `path:match_count` for files with at least one match
///   (`show_line_number` is ignored in this mode).
/// - otherwise: print each matching line as `path:line_contents`, or as
///   `path:line_number:line_contents` when `show_line_number` is true.
///
/// Directories that cannot be read are skipped with a warning; unreadable
/// or non-UTF-8 files are likewise warned about and skipped.
pub fn run_search(
    pattern: &str,
    path: &Path,
    ignore_case: bool,
    show_line_number: bool,
    count_only: bool,
) {
    if !path.exists() {
        eprintln!("error: path does not exist: {}", path.display());
        process::exit(1);
    }

    if !path.is_dir() {
        eprintln!("error: not a directory: {}", path.display());
        process::exit(1);
    }

    walk_dir(path, pattern, ignore_case, show_line_number, count_only);
}

/// Recursively visit `dir` and search each regular file for `pattern`.
fn walk_dir(
    dir: &Path,
    pattern: &str,
    ignore_case: bool,
    show_line_number: bool,
    count_only: bool,
) {
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
            walk_dir(&path, pattern, ignore_case, show_line_number, count_only);
        } else if file_type.is_file() {
            search_file(&path, pattern, ignore_case, show_line_number, count_only);
        }
        // Symlinks and other special files are ignored for now.
    }
}

/// Open `path` as UTF-8 text and report lines containing `pattern`.
///
/// Performs a single pass over the file: matching is decided by
/// [`line_matches`]; output is delegated to the print helpers.
fn search_file(
    path: &Path,
    pattern: &str,
    ignore_case: bool,
    show_line_number: bool,
    count_only: bool,
) {
    let contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(err) => {
            eprintln!("warning: skipping {}: {}", path.display(), err);
            return;
        }
    };

    // Lowercase the pattern once when ignoring case so the per-line path stays shared.
    let pattern_lower = ignore_case.then(|| pattern.to_lowercase());

    let mut match_count = 0usize;

    for (idx, line) in contents.lines().enumerate() {
        if !line_matches(line, pattern, pattern_lower.as_deref()) {
            continue;
        }

        match_count += 1;

        // Count mode defers output until the pass completes; line mode prints immediately.
        if !count_only {
            print_line_match(path, idx + 1, line, show_line_number);
        }
    }

    if count_only && match_count > 0 {
        print_count_match(path, match_count);
    }
}

/// Print a single line match (`path:line` or `path:N:line`).
fn print_line_match(path: &Path, line_number: usize, line: &str, show_line_number: bool) {
    if show_line_number {
        println!("{}:{}:{}", path.display(), line_number, line);
    } else {
        println!("{}:{}", path.display(), line);
    }
}

/// Print a per-file match count (`path:count`).
fn print_count_match(path: &Path, match_count: usize) {
    println!("{}:{}", path.display(), match_count);
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
