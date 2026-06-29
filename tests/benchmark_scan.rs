//! Benchmark-style tests: large trees and sequential vs parallel timings.
//!
//! Builds repositories of at least 10k files, measures wall-clock time for
//! sequential and parallel pipelines, and asserts the parallel path completes
//! within budget. Detailed comparison numbers are printed (visible with
//! `cargo test -- --nocapture`) and summarized in `BENCHMARKS.md`.

use rustscan::pipeline::{SearchConfig, run_parallel, run_sequential};
use std::fs;
use std::time::{Duration, Instant};
use tempfile::tempdir;

const FILE_COUNT: usize = 10_000;
const MAX_DURATION: Duration = Duration::from_secs(60);

fn build_large_tree(file_count: usize) -> tempfile::TempDir {
    let dir = tempdir().unwrap();
    for i in 0..file_count {
        let sub = dir.path().join(format!("bucket_{}", i % 100));
        fs::create_dir_all(&sub).unwrap();
        let body = if i % 17 == 0 {
            format!("line\nTARGET_{i}\nend\n")
        } else {
            format!("noise_{i}\nmore\nlines\n")
        };
        fs::write(sub.join(format!("f_{i}.txt")), body).unwrap();
    }
    dir
}

#[test]
fn large_directory_scan_completes_quickly() {
    let dir = build_large_tree(FILE_COUNT);
    let start = Instant::now();
    let results = run_parallel(
        "TARGET_",
        dir.path(),
        SearchConfig {
            include_hidden: false,
            ..SearchConfig::default()
        },
    )
    .expect("search succeeds");
    let elapsed = start.elapsed();

    let expected_hits = (0..FILE_COUNT).filter(|i| i % 17 == 0).count();
    assert_eq!(results.total_hits(), expected_hits);
    assert!(
        elapsed < MAX_DURATION,
        "parallel scan of {FILE_COUNT} files took {elapsed:?}, budget {MAX_DURATION:?}"
    );
}

#[test]
fn sequential_and_parallel_agree_on_10k_files() {
    let dir = build_large_tree(FILE_COUNT);
    let cfg = SearchConfig::default();
    let seq = run_sequential("TARGET_", dir.path(), cfg).unwrap();
    let par = run_parallel("TARGET_", dir.path(), cfg).unwrap();
    assert_eq!(seq, par, "structured results must match on large trees");
}

#[test]
fn prints_sequential_vs_parallel_timing_10k() {
    let dir = build_large_tree(FILE_COUNT);
    let cfg = SearchConfig::default();
    let pattern = "TARGET_";

    // Warm-up (populate OS page cache similarly for both).
    let _ = run_sequential(pattern, dir.path(), cfg);
    let _ = run_parallel(pattern, dir.path(), cfg);

    let t0 = Instant::now();
    let seq = run_sequential(pattern, dir.path(), cfg).unwrap();
    let seq_elapsed = t0.elapsed();

    let t1 = Instant::now();
    let par = run_parallel(pattern, dir.path(), cfg).unwrap();
    let par_elapsed = t1.elapsed();

    assert_eq!(seq, par);

    let speedup = seq_elapsed.as_secs_f64() / par_elapsed.as_secs_f64().max(1e-9);
    eprintln!(
        "\n=== 10k-file scan timing (see BENCHMARKS.md) ===\n\
         files:     {FILE_COUNT}\n\
         hits:      {}\n\
         sequential:{seq_elapsed:?}\n\
         parallel:  {par_elapsed:?}\n\
         speedup:   {speedup:.2}x\n\
         rayon threads (default pool): {}\n",
        seq.total_hits(),
        rayon::current_num_threads()
    );

    // Parallel should not be pathologically slower on multi-core hosts.
    // Allow 3× sequential as a soft ceiling (I/O-bound / single-core CI).
    assert!(
        par_elapsed < seq_elapsed.saturating_mul(3) + Duration::from_millis(500),
        "parallel {par_elapsed:?} far slower than sequential {seq_elapsed:?}"
    );
}
