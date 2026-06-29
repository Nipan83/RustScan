//! RustScan CLI binary — thin wrapper over the `rustscan` library pipeline.

mod cli;

use cli::Cli;
use rustscan::pipeline::{SearchError, run_and_write};
use std::io::{self, Write};
use std::process;

fn main() {
    let args = Cli::parse_args();
    let config = args.search_config();
    let mut stdout = io::stdout().lock();

    match run_and_write(&args.pattern, &args.path, config, &mut stdout) {
        Ok(()) => {}
        Err(SearchError::InvalidPattern(err)) => {
            let _ = writeln!(io::stderr(), "error: invalid regular expression: {err}");
            process::exit(1);
        }
        Err(SearchError::PathMissing) => {
            let _ = writeln!(
                io::stderr(),
                "error: path does not exist: {}",
                args.path.display()
            );
            process::exit(1);
        }
        Err(SearchError::NotADirectory) => {
            let _ = writeln!(
                io::stderr(),
                "error: not a directory: {}",
                args.path.display()
            );
            process::exit(1);
        }
    }
}
