# RustScan

A fast recursive file search CLI written in Rust. Walks a directory tree and finds lines that contain a given substring pattern (not a regular expression).

## Requirements

- [Rust](https://www.rust-lang.org/) (edition 2024 toolchain; recent stable is fine)

## Build

```bash
cargo build --release
```

The binary is written to `target/release/rustscan` (or `target/debug/rustscan` for a debug build).

## Usage

```bash
rustscan [OPTIONS] <PATTERN> <PATH>
```

| Argument / option | Description |
| ----------------- | ----------- |
| `PATTERN` | Substring to search for (not a regular expression). Case-sensitive by default. |
| `PATH` | Directory to search recursively |
| `-i`, `--ignore-case` | Match without regard to letter case |
| `-n`, `--line-number` | Include 1-based line numbers in the output |

### Examples

```bash
# Case-sensitive search in the current project
cargo run -- "TODO" .

# Case-insensitive search
cargo run -- -i "todo" .

# Include line numbers
cargo run -- -n "TODO" .

# Both flags together
cargo run -- -i -n "todo" ./src

# Release binary
./target/release/rustscan -n "fn main" ./src

# Help
cargo run -- --help
```

### Output format

By default, each matching line is printed as:

```text
path:line_contents
```

With `-n` / `--line-number`, matches include the 1-based line number:

```text
path:line_number:line_contents
```

Files with no matches produce no output. Line contents are printed as they appear in the file (original casing is preserved even when `--ignore-case` is used).

**Default (no flags):**

```text
src/search.rs:pub fn run_search(pattern: &str, path: &Path) {
```

**With `-n`:**

```text
src/search.rs:17:pub fn run_search(pattern: &str, path: &Path) {
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
├── main.rs    # Entry point; wires CLI options into search
├── cli.rs     # Argument parsing (clap)
├── search.rs  # Directory traversal and substring search
└── utils.rs   # Shared helpers (placeholder for later)
```

Traversal (`walk_dir`), per-file search (`search_file`), and match printing (`print_match`) are kept separate so each has a single responsibility. CLI flags are passed as simple values so search does not depend on the clap types.

## Development

```bash
cargo fmt
cargo build
cargo clippy -- -D warnings
cargo run -- -i -n "pattern" ./some/dir
```

## License

See the repository for license information (if added).
