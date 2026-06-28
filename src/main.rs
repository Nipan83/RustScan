mod cli;
mod search;
mod utils;

use cli::Cli;
use search::run_search;

fn main() {
    let args = Cli::parse();
    run_search(&args.pattern, &args.path);
}
