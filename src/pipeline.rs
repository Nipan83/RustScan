//! End-to-end search pipeline: compile → collect → (caller formats).

use crate::collect::{collect_matches, collect_matches_parallel, collect_matches_sequential};
use crate::output::{Formatter, formatter_for};
use crate::pattern::compile_pattern;
use crate::result::SearchResults;
use std::path::Path;

/// Configuration for a full search run (CLI-agnostic).
#[derive(Debug, Clone, Copy, Default)]
pub struct SearchConfig {
    pub ignore_case: bool,
    pub show_line_number: bool,
    pub count_only: bool,
    pub include_hidden: bool,
}

/// Errors that abort the pipeline before or during search.
#[derive(Debug)]
pub enum SearchError {
    InvalidPattern(regex::Error),
    PathMissing,
    NotADirectory,
}

impl std::fmt::Display for SearchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SearchError::InvalidPattern(err) => {
                write!(f, "invalid regular expression: {err}")
            }
            SearchError::PathMissing => write!(f, "path does not exist"),
            SearchError::NotADirectory => write!(f, "not a directory"),
        }
    }
}

impl std::error::Error for SearchError {}

fn validate_root(root: &Path) -> Result<(), SearchError> {
    if !root.exists() {
        return Err(SearchError::PathMissing);
    }
    if !root.is_dir() {
        return Err(SearchError::NotADirectory);
    }
    Ok(())
}

/// Compile `pattern`, validate `root`, and collect structured matches (parallel).
///
/// Pattern compilation happens **before** any filesystem traversal so invalid
/// regexes fail fast. The same [`regex::Regex`] is shared across Rayon workers.
/// The caller renders [`SearchResults`] with a [`Formatter`].
pub fn run(pattern: &str, root: &Path, config: SearchConfig) -> Result<SearchResults, SearchError> {
    run_parallel(pattern, root, config)
}

/// Parallel pipeline (default).
pub fn run_parallel(
    pattern: &str,
    root: &Path,
    config: SearchConfig,
) -> Result<SearchResults, SearchError> {
    let regex =
        compile_pattern(pattern, config.ignore_case).map_err(SearchError::InvalidPattern)?;
    validate_root(root)?;
    Ok(collect_matches_parallel(
        root,
        &regex,
        config.include_hidden,
    ))
}

/// Sequential pipeline (benchmark baseline and regression comparisons).
pub fn run_sequential(
    pattern: &str,
    root: &Path,
    config: SearchConfig,
) -> Result<SearchResults, SearchError> {
    let regex =
        compile_pattern(pattern, config.ignore_case).map_err(SearchError::InvalidPattern)?;
    validate_root(root)?;
    Ok(collect_matches_sequential(
        root,
        &regex,
        config.include_hidden,
    ))
}

/// Run the (parallel) pipeline and write formatted output to `out`.
pub fn run_and_write<W: std::io::Write>(
    pattern: &str,
    root: &Path,
    config: SearchConfig,
    out: &mut W,
) -> Result<(), SearchError> {
    let results = run(pattern, root, config)?;
    let formatter = formatter_for(config.count_only, config.show_line_number);
    formatter
        .write(&results, out)
        .expect("writing search output should not fail for stdout-like writers");
    Ok(())
}

/// Build the formatter that matches `config` (for callers that already have results).
pub fn formatter(config: SearchConfig) -> Box<dyn Formatter> {
    formatter_for(config.count_only, config.show_line_number)
}

// Re-export default collect for documentation links.
#[allow(unused_imports)]
use collect_matches as _;

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn invalid_pattern_before_traversal() {
        let dir = tempdir().unwrap();
        let err = run("(", dir.path(), SearchConfig::default()).unwrap_err();
        assert!(matches!(err, SearchError::InvalidPattern(_)));
    }

    #[test]
    fn missing_path() {
        let err = run(
            "x",
            Path::new("/nonexistent/rustscan/path"),
            SearchConfig::default(),
        )
        .unwrap_err();
        assert!(matches!(err, SearchError::PathMissing));
    }

    #[test]
    fn full_pipeline_finds_matches() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("a.txt"), "hello\nworld\n").unwrap();
        let results = run("hello", dir.path(), SearchConfig::default()).unwrap();
        assert_eq!(results.total_hits(), 1);
    }

    #[test]
    fn sequential_and_parallel_identical() {
        let dir = tempdir().unwrap();
        for i in 0..50 {
            fs::create_dir_all(dir.path().join(format!("d{i}"))).unwrap();
            fs::write(
                dir.path().join(format!("d{i}/f.txt")),
                if i % 2 == 0 { "hit\n" } else { "miss\n" },
            )
            .unwrap();
        }
        let cfg = SearchConfig::default();
        let seq = run_sequential("hit", dir.path(), cfg).unwrap();
        let par = run_parallel("hit", dir.path(), cfg).unwrap();
        assert_eq!(seq, par);
    }

    #[test]
    fn run_and_write_count_mode() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("a.txt"), "hello\nhello\n").unwrap();
        let mut buf = Vec::new();
        run_and_write(
            "hello",
            dir.path(),
            SearchConfig {
                count_only: true,
                ..SearchConfig::default()
            },
            &mut buf,
        )
        .unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains(":2"));
    }
}
