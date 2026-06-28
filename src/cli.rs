//! Command-line interface definitions and argument parsing.

use clap::Parser;
use std::path::PathBuf;

/// Fast recursive file search utility.
///
/// Searches for a pattern within files under the given directory.
#[derive(Debug, Parser)]
#[command(
    name = "rustscan",
    version,
    about = "Search for a pattern in files under a directory",
    long_about = "A fast recursive file search tool. Provide a search pattern and a directory path to scan."
)]
pub struct Cli {
    /// The search pattern to look for
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
}

impl Cli {
    /// Parse command-line arguments, exiting with a helpful message on error.
    pub fn parse() -> Self {
        <Self as Parser>::parse()
    }
}
