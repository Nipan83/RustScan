# RustScan

A fast recursive file search CLI written in Rust. Walks a directory tree and finds lines that match a **regular expression** pattern (powered by the [`regex`](https://docs.rs/regex) crate).

The project is split into a **library** (`rustscan`) implementing a composable search pipeline and a thin **CLI binary** that parses flags and prints formatted results.

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
| `PATTERN` | Regular expression to search for (Rust `regex` syntax). Case-sensitive by default. |
| `PATH` | Directory to search recursively |
| `-i`, `--ignore-case` | Case-insensitive matching (regex `i` flag) |
| `-n`, `--line-number` | Include 1-based line numbers in line-by-line output |
| `-c`, `--count` | Print `path:match_count` per file instead of each matching line |
| `--hidden` | Include hidden files and directories (names starting with `.`) |

When `--count` is set, it takes precedence over line-by-line output, so `--line-number` has no effect. `--ignore-case` still applies in count mode.

Hidden entries are **skipped by default**; use `--hidden` to search them. Hidden directories are not descended into unless `--hidden` is set.

### Behavioral note (regex vs substring)

`PATTERN` is always treated as a regular expression. Metacharacters such as `.`, `*`, `+`, `?`, `[]`, and `()` have special meaning. Escape them for literals (for example `a\.c`).

The pattern is **compiled once** before any directory walk. An invalid pattern prints an error and exits with status `1` without reading the filesystem.

### Examples

```bash
cargo run -- "TODO" .
cargo run -- -n "TODO|FIXME" ./src
cargo run -- -i "hello|world" .
cargo run -- -c "TODO" .
cargo run -- --hidden "SECRET" .
cargo run -- "(" .   # fails fast: invalid regex
```

### Output format

**Line mode** (default):

```text
path:line_contents
```

With `-n`:

```text
path:line_number:line_contents
```

**Count mode** (`-c`):

```text
path:match_count
```

Only files with at least one matching line are printed.

## Architecture overview

Search is a **pipeline** that separates concerns and avoids “search and print” coupling. The default implementation runs **traversal and file search in parallel with Rayon**, shares one compiled regex across workers, and **sorts results by path** before rendering so output order is deterministic.

```text
CLI (src/cli.rs, src/main.rs)
        │
        ▼
┌───────────────────┐
│  pattern          │  compile user string → Regex (once, shared &Sync)
└─────────┬─────────┘
          ▼
┌───────────────────┐
│  traverse (Rayon) │  parallel dir walk, honor --hidden, yield paths
└─────────┬─────────┘
          ▼
┌───────────────────┐
│  matching (Rayon) │  par_iter files; apply shared Regex per line
└─────────┬─────────┘
          ▼
┌───────────────────┐
│  collect          │  FileMatches → sort_deterministic → SearchResults
└─────────┬─────────┘
          ▼
┌───────────────────┐
│  output           │  LineFormatter | CountFormatter → stdout
└───────────────────┘
```

| Module | Responsibility |
| ------ | -------------- |
| `pattern` | `compile_pattern` — regex build + case-insensitivity |
| `traverse` | Sequential + parallel recursive walks; hidden filtering |
| `matching` | `match_lines` / `match_file` against a compiled regex |
| `result` | `MatchHit`, `FileMatches`, `SearchResults` (+ path sort) |
| `collect` | `collect_matches_parallel` / `_sequential`; lock-free Rayon `collect` |
| `output` | `Formatter` trait; line vs count renderers |
| `pipeline` | `run` / `run_parallel` / `run_sequential`, `run_and_write` |
| `utils` | `is_hidden` and small path helpers |

The binary maps CLI flags to [`SearchConfig`](src/pipeline.rs), calls `run_and_write` (parallel), and maps errors to exit codes. Use `run_sequential` for benchmarks and regression comparisons. Performance notes: [BENCHMARKS.md](BENCHMARKS.md).

## Developer guide

### Layout

```text
src/
├── lib.rs       # crate root, re-exports
├── main.rs      # CLI entry
├── cli.rs       # clap definitions → SearchConfig
├── pattern.rs
├── traverse.rs
├── matching.rs
├── result.rs
├── collect.rs
├── output.rs
├── pipeline.rs
└── utils.rs
tests/
├── integration_cli.rs   # full binary tests
└── benchmark_scan.rs    # large-tree timing guardrail
benches/
└── scan_bench.rs        # criterion benchmarks (`cargo bench`)
```

### Commands

```bash
cargo fmt
cargo test                 # unit + integration + parallel regression + 10k timing
cargo test --test parallel_regression
cargo test --test benchmark_scan -- --nocapture
cargo clippy --all-targets -- -D warnings
cargo doc --no-deps --open # library API docs
cargo bench --bench scan_bench
cargo run -- -i -n "fn " ./src
```

See [BENCHMARKS.md](BENCHMARKS.md) for sequential vs parallel timings (~3.6× on a 10-core host / 10k files in one measured debug run).

### Extending

- **New output style**: implement `output::Formatter` and select it from `formatter_for` or from your own binary.
- **New traversal filter**: extend `TraverseOptions` and apply checks only in `traverse` (keep matching unaware of FS policy).
- **Library use**:

```rust
use rustscan::{run, LineFormatter, Formatter, SearchConfig};
use std::path::Path;

let results = run(r"TODO|FIXME", Path::new("."), SearchConfig::default())?;
let text = LineFormatter { show_line_number: true }.format_string(&results);
print!("{text}");
```

### Errors and warnings

| Situation | Behavior |
| --------- | -------- |
| Missing `PATTERN` or `PATH` | Clap usage error, non-zero exit |
| Invalid regular expression | `error: invalid regular expression: …`, exit `1` (before traversal) |
| `PATH` missing / not a directory | `error:` on stderr, exit `1` |
| Unreadable directory or non-UTF-8 file | `warning:` on stderr; search continues |

## License

See the repository for license information (if added).
