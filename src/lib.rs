//! RustScan library: composable recursive regex search pipeline.
//!
//! # Pipeline overview
//!
//! 1. [`pattern`] — compile the user pattern into a reusable [`regex::Regex`]
//! 2. [`traverse`] — walk the filesystem (sequential or Rayon-parallel) and yield paths
//! 3. [`matching`] — apply the shared regex to each line of a file
//! 4. [`result`] — structured hit / per-file aggregates
//! 5. [`collect`] — orchestrate traversal + matching into sorted [`result::SearchResults`]
//! 6. [`output`] — render results through interchangeable formatters
//!
//! Parallel collection shares one compiled regex across Rayon workers and sorts
//! by path before rendering so output order is deterministic.
//!
//! The binary (`main`) parses CLI flags, runs [`pipeline::run`], and prints
//! formatter output. Library consumers can use the same pieces independently.

pub mod collect;
pub mod matching;
pub mod output;
pub mod pattern;
pub mod pipeline;
pub mod result;
pub mod traverse;
pub mod utils;

pub use collect::{collect_matches, collect_matches_parallel, collect_matches_sequential};
pub use output::{CountFormatter, Formatter, LineFormatter};
pub use pattern::compile_pattern;
pub use pipeline::{SearchConfig, SearchError, run, run_parallel, run_sequential};
pub use result::{FileMatches, MatchHit, SearchResults};
pub use traverse::{collect_file_paths, collect_file_paths_parallel, walk_files};
pub use utils::is_hidden;
