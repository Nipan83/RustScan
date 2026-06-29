//! Line-level matching of a compiled regex against file contents.

use crate::result::MatchHit;
use regex::Regex;
use std::fs;
use std::io;
use std::path::Path;

/// Find all lines in `contents` that match `regex`.
///
/// Line numbers in the returned hits are **1-based**.
pub fn match_lines(contents: &str, regex: &Regex) -> Vec<MatchHit> {
    contents
        .lines()
        .enumerate()
        .filter_map(|(idx, line)| {
            if regex.is_match(line) {
                Some(MatchHit {
                    line_number: idx + 1,
                    line: line.to_string(),
                })
            } else {
                None
            }
        })
        .collect()
}

/// Read `path` as UTF-8 and return matching lines, or an I/O error.
///
/// Callers typically treat non-UTF-8 / unreadable files as skippable warnings.
pub fn match_file(path: &Path, regex: &Regex) -> io::Result<Vec<MatchHit>> {
    let contents = fs::read_to_string(path)?;
    Ok(match_lines(&contents, regex))
}

/// Return true if a single `line` matches `regex`.
pub fn line_matches(regex: &Regex, line: &str) -> bool {
    regex.is_match(line)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pattern::compile_pattern;

    #[test]
    fn finds_matching_lines_with_numbers() {
        let re = compile_pattern("hello", false).unwrap();
        let hits = match_lines("a\nhello\nb\nhello world\n", &re);
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].line_number, 2);
        assert_eq!(hits[0].line, "hello");
        assert_eq!(hits[1].line_number, 4);
    }

    #[test]
    fn respects_case_insensitive_regex() {
        let re = compile_pattern("hello", true).unwrap();
        let hits = match_lines("Hello\nHELLO\nx\n", &re);
        assert_eq!(hits.len(), 2);
    }

    #[test]
    fn empty_when_no_match() {
        let re = compile_pattern("zzz", false).unwrap();
        assert!(match_lines("hello\nworld\n", &re).is_empty());
    }

    #[test]
    fn line_matches_helper() {
        let re = compile_pattern(r"\d+", false).unwrap();
        assert!(line_matches(&re, "abc 123"));
        assert!(!line_matches(&re, "abc"));
    }
}
