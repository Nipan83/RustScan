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
| `-n`, `--line-number` | Include 1-based line numbers in line-by-line output |
| `-c`, `--count` | Print `path:match_count` per file instead of each matching line |

When `--count` is set, it takes precedence over line-by-line output, so `--line-number` has no effect. `--ignore-case` still applies in count mode.

### Examples

```bash
# Case-sensitive search in the current project
cargo run -- "TODO" .

# Case-insensitive search
cargo run -- -i "todo" .

# Include line numbers
cargo run -- -n "TODO" .

# Per-file match counts
cargo run -- -c "TODO" .

# Count with case-insensitive matching
cargo run -- -c -i "todo" ./src

# Line mode with both matching flags
cargo run -- -i -n "todo" ./src

# Release binary
./target/release/rustscan -n "fn main" ./src

# Help
cargo run -- --help
```

### Output format

**Line mode** (default, when `--count` is not set):

```text
path:line_contents
```

With `-n` / `--line-number`:

```text
path:line_number:line_contents
```

**Count mode** (`-c` / `--count`):

```text
path:match_count
```

Only files with at least one match are printed. Files with zero matches produce no output.

Line contents are printed as they appear in the file (original casing is preserved even when `--ignore-case` is used).

**Default (no flags):**

```text
src/search.rs:pub fn run_search(pattern: &str, path: &Path) {
```

**With `-n`:**

```text
src/search.rs:17:pub fn run_search(pattern: &str, path: &Path) {
```

**With `-c`:**

```text
src/search.rs:3
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

Traversal (`walk_dir`), matching (`line_matches`), and output (`print_line_match` / `print_count_match`) stay separate so each has a single responsibility. Each file is scanned in one pass. CLI flags are passed as simple values so search does not depend on the clap types.

## Development

```bash
cargo fmt
cargo build
cargo clippy -- -D warnings
cargo run -- -c -i "pattern" ./some/dir
```

## License

See the repository for license information (if added).
