//! Output formatters: render [`SearchResults`] without owning search logic.

use crate::result::SearchResults;
use std::io::{self, Write};

/// Renders structured search results to a writer.
pub trait Formatter {
    fn write(&self, results: &SearchResults, out: &mut dyn Write) -> io::Result<()>;

    /// Convenience: format into a UTF-8 string.
    fn format_string(&self, results: &SearchResults) -> String {
        let mut buf = Vec::new();
        self.write(results, &mut buf).expect("write to vec");
        String::from_utf8(buf).expect("utf-8 output")
    }
}

/// Line-oriented output: `path:line` or `path:N:line`.
#[derive(Debug, Clone, Copy, Default)]
pub struct LineFormatter {
    pub show_line_number: bool,
}

impl Formatter for LineFormatter {
    fn write(&self, results: &SearchResults, out: &mut dyn Write) -> io::Result<()> {
        for file in &results.files {
            for hit in &file.hits {
                if self.show_line_number {
                    writeln!(
                        out,
                        "{}:{}:{}",
                        file.path.display(),
                        hit.line_number,
                        hit.line
                    )?;
                } else {
                    writeln!(out, "{}:{}", file.path.display(), hit.line)?;
                }
            }
        }
        Ok(())
    }
}

/// Count-oriented output: `path:match_count` per file with ≥1 hit.
///
/// Takes precedence over line-oriented display when selected by the CLI.
#[derive(Debug, Clone, Copy, Default)]
pub struct CountFormatter;

impl Formatter for CountFormatter {
    fn write(&self, results: &SearchResults, out: &mut dyn Write) -> io::Result<()> {
        for file in &results.files {
            let count = file.match_count();
            if count > 0 {
                writeln!(out, "{}:{}", file.path.display(), count)?;
            }
        }
        Ok(())
    }
}

/// Select the appropriate formatter for CLI flags.
///
/// When `count_only` is true, [`CountFormatter`] is used and `show_line_number`
/// is ignored.
pub fn formatter_for(count_only: bool, show_line_number: bool) -> Box<dyn Formatter> {
    if count_only {
        Box::new(CountFormatter)
    } else {
        Box::new(LineFormatter { show_line_number })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::result::{FileMatches, MatchHit, SearchResults};
    use std::path::PathBuf;

    fn sample_results() -> SearchResults {
        let mut r = SearchResults::new();
        r.push(FileMatches {
            path: PathBuf::from("a.txt"),
            hits: vec![
                MatchHit {
                    line_number: 1,
                    line: "hello".into(),
                },
                MatchHit {
                    line_number: 3,
                    line: "hello again".into(),
                },
            ],
        });
        r
    }

    #[test]
    fn line_formatter_without_numbers() {
        let out = LineFormatter {
            show_line_number: false,
        }
        .format_string(&sample_results());
        assert_eq!(out, "a.txt:hello\na.txt:hello again\n");
    }

    #[test]
    fn line_formatter_with_numbers() {
        let out = LineFormatter {
            show_line_number: true,
        }
        .format_string(&sample_results());
        assert_eq!(out, "a.txt:1:hello\na.txt:3:hello again\n");
    }

    #[test]
    fn count_formatter() {
        let out = CountFormatter.format_string(&sample_results());
        assert_eq!(out, "a.txt:2\n");
    }

    #[test]
    fn formatter_for_prefers_count() {
        let f = formatter_for(true, true);
        let out = f.format_string(&sample_results());
        assert_eq!(out, "a.txt:2\n");
    }
}
