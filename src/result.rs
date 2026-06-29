//! Structured search results produced by the collection stage.

use std::path::PathBuf;

/// A single matching line within a file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchHit {
    /// 1-based line number in the source file.
    pub line_number: usize,
    /// Full line contents (without trailing newline).
    pub line: String,
}

/// All matches found in one file (only files with ≥1 hit are collected).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileMatches {
    pub path: PathBuf,
    pub hits: Vec<MatchHit>,
}

impl FileMatches {
    /// Number of matching lines in this file.
    pub fn match_count(&self) -> usize {
        self.hits.len()
    }
}

/// Aggregated results for an entire search run.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SearchResults {
    pub files: Vec<FileMatches>,
}

impl SearchResults {
    pub fn new() -> Self {
        Self { files: Vec::new() }
    }

    pub fn push(&mut self, file: FileMatches) {
        self.files.push(file);
    }

    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    /// Total matching lines across all files.
    pub fn total_hits(&self) -> usize {
        self.files.iter().map(FileMatches::match_count).sum()
    }

    /// Sort files by path so rendering is deterministic regardless of
    /// parallel completion order. Hits within each file are already in
    /// ascending line-number order from matching.
    pub fn sort_deterministic(&mut self) {
        self.files.sort_by(|a, b| a.path.cmp(&b.path));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_match_count() {
        let fm = FileMatches {
            path: PathBuf::from("a.txt"),
            hits: vec![
                MatchHit {
                    line_number: 1,
                    line: "x".into(),
                },
                MatchHit {
                    line_number: 2,
                    line: "y".into(),
                },
            ],
        };
        assert_eq!(fm.match_count(), 2);
    }

    #[test]
    fn results_total_hits() {
        let mut r = SearchResults::new();
        r.push(FileMatches {
            path: PathBuf::from("a"),
            hits: vec![MatchHit {
                line_number: 1,
                line: "a".into(),
            }],
        });
        r.push(FileMatches {
            path: PathBuf::from("b"),
            hits: vec![
                MatchHit {
                    line_number: 1,
                    line: "b".into(),
                },
                MatchHit {
                    line_number: 2,
                    line: "c".into(),
                },
            ],
        });
        assert_eq!(r.total_hits(), 3);
        assert!(!r.is_empty());
    }

    #[test]
    fn sort_deterministic_orders_by_path() {
        let mut r = SearchResults::new();
        r.push(FileMatches {
            path: PathBuf::from("z.txt"),
            hits: vec![],
        });
        r.push(FileMatches {
            path: PathBuf::from("a.txt"),
            hits: vec![],
        });
        r.sort_deterministic();
        assert_eq!(r.files[0].path, PathBuf::from("a.txt"));
        assert_eq!(r.files[1].path, PathBuf::from("z.txt"));
    }
}
