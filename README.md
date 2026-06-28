# RustScan

A fast recursive file search CLI written in Rust. Walks a directory tree and finds lines that contain a case-sensitive substring pattern.

## Requirements

- [Rust](https://www.rust-lang.org/) (edition 2024 toolchain; recent stable is fine)

## Build

```bash
cargo build --release
```

The binary is written to `target/release/rustscan` (or `target/debug/rustscan` for a debug build).

## Usage

```bash
rustscan <PATTERN> <PATH>
```

| Argument | Description |
| -------- | ----------- |
| `PATTERN` | Case-sensitive substring to search for (not a regular expression) |
| `PATH` | Directory to search recursively |

### Examples

```bash
# Search the current project for "TODO"
cargo run -- "TODO" .

# Release binary
./target/release/rustscan "fn main" ./src

# Help
cargo run -- --help
```

### Output format

Each matching line is printed as:

```text
path:line_number:line_contents
```

Line numbers are **1-based**. Files with no matches produce no output.

Example:

```text
src/search.rs:17:pub fn run_search(pattern: &str, path: &Path) {
src/main.rs:10:    run_search(&args.pattern, &args.path);
```

### Errors and warnings

| Situation | Behavior |
| --------- | -------- |
| Missing `PATTERN` or `PATH` | Clap prints a usage error and exits non-zero |
| `PATH` does not exist or is not a directory | `error:` on stderr, exit code `1` |
| Directory cannot be read (permissions, I/O) | `warning: skipping directory …` on stderr; walk continues |
| File cannot be opened or is not valid UTF-8 | `warning: skipping …` on stderr; remaining files are still scanned |

Symlinks and non-regular files are ignored for now.

## Project layout

```text
src/
├── main.rs    # Entry point; wires CLI to search
├── cli.rs     # Argument parsing (clap)
├── search.rs  # Directory traversal and substring search
└── utils.rs   # Shared helpers (placeholder for later)
```

Traversal (`walk_dir`) and per-file search (`search_file`) are kept separate so each has a single responsibility.

## Development

```bash
cargo fmt
cargo build
cargo clippy -- -D warnings
cargo run -- "pattern" ./some/dir
```

## License

See the repository for license information (if added).
