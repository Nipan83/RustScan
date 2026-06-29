# Parallel vs sequential benchmarks

RustScan’s default pipeline uses **Rayon** for:

1. **Directory traversal** — sibling subdirectories are walked with `par_iter` (`collect_file_paths_parallel`).
2. **File search** — each discovered path is matched with `par_iter`, sharing one compiled `regex::Regex` (`Sync`, no per-thread recompilation).

Results are **sorted by path** (`SearchResults::sort_deterministic`) before formatting so output matches the sequential baseline regardless of worker completion order. Collection uses Rayon’s `collect` into a `Vec` (no global mutex on the hot path).

## How to measure

```bash
# Timing guardrail + printed comparison (10k files)
cargo test --test benchmark_scan prints_sequential_vs_parallel_timing_10k -- --nocapture

# Full regression (seq == par on structured + formatted output)
cargo test --test parallel_regression

# Criterion (optional; builds larger dep graph)
cargo bench --bench scan_bench
```

## Representative run (this machine)

Captured with `cargo test --test benchmark_scan prints_sequential_vs_parallel_timing_10k -- --nocapture` after a warm-up pass (debug build, temp tree on local disk).

| Metric | Value |
| ------ | ----- |
| Host CPUs (logical) | 10 |
| Rayon default pool size | 10 threads |
| Files in tree | 10 000 (100 buckets × ~100 files) |
| Matching lines (`TARGET_`) | 589 |
| **Sequential** wall time | **~628 ms** |
| **Parallel** wall time | **~173 ms** |
| **Speedup** | **~3.6×** |

### CPU utilization and scalability

- **Sequential**: typically saturates **~1 core** during matching; I/O wait still appears when reading many small files.
- **Parallel**: Rayon schedules work across the pool (**up to 10 workers** here). Observed wall-clock improvement (~3.6× on 10 cores) is **sub-linear** because:
  - Traversal and `read_to_string` are I/O-heavy.
  - Debug builds spend extra time on bounds checks / lack of inlining.
  - Synchronization is minimal (no result mutex), but the OS filesystem and page cache dominate beyond a handful of cores.
- **Scalability expectation**: on SSD-backed trees with larger file bodies or costlier regexes, speedup usually grows toward the core count; on tiny files in a warm cache, gains plateau earlier. Release builds (`cargo run --release` / `cargo bench`) narrow overhead and often improve both modes, preserving the parallel advantage.

Re-run on your hardware and update the table if you need machine-specific numbers. Release-mode criterion:

```bash
cargo bench --bench scan_bench -- --sample-size 20
```

compares `sequential/1000`, `parallel/1000`, `sequential/10000`, and `parallel/10000`.

## Correctness

Parallel and sequential modes are compared in:

- `tests/parallel_regression.rs` — flags (`-i`, `--hidden`), line and count formatters, multi-run stability.
- `tests/benchmark_scan.rs` — `sequential_and_parallel_agree_on_10k_files`.
- Unit tests in `collect` / `pipeline` / `traverse` for smaller fixtures.

Formatted stdout is identical when the same `Formatter` is applied to both result sets (paths sorted; hits stay in line-number order within each file).
