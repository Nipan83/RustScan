//! Command-line interface definitions and argument parsing.

use clap::Parser;
use rustscan::SearchConfig;
use std::path::PathBuf;

/// Fast recursive file search utility.
#[derive(Debug, Parser)]
#[command(
    name = "rustscan",
    version,
    about = "Search for a regex pattern in files under a directory",
    long_about = "A fast recursive file search tool. Provide a regular expression pattern and a directory path to scan. Use --ignore-case for case-insensitive matching."
)]
pub struct Cli {
    /// Regular expression pattern to search for
    #[arg(value_name = "PATTERN")]
    pub pattern: String,

    /// Directory path to search in
    #[arg(value_name = "PATH")]
    pub path: PathBuf,

    /// Perform a case-insensitive search
    #[arg(short = 'i', long = "ignore-case", default_value_t = false)]
    pub ignore_case: bool,

    /// Prefix each match with its 1-based line number
    #[arg(short = 'n', long = "line-number", default_value_t = false)]
    pub line_number: bool,

    /// Print only a per-file count of matching lines (`path:count`)
    #[arg(short = 'c', long = "count", default_value_t = false)]
    pub count: bool,

    /// Include hidden files and directories (names starting with `.`)
    #[arg(long = "hidden", default_value_t = false)]
    pub hidden: bool,
}

impl Cli {
    pub fn parse_args() -> Self {
        <Self as Parser>::parse()
    }

    /// Map CLI flags into a library [`SearchConfig`].
    pub fn search_config(&self) -> SearchConfig {
        SearchConfig {
            ignore_case: self.ignore_case,
            show_line_number: self.line_number,
            count_only: self.count,
            include_hidden: self.hidden,
        }
    }
}
