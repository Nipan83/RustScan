//! Criterion benchmarks: sequential vs parallel on large directory scans.

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use rustscan::pipeline::{SearchConfig, run_parallel, run_sequential};
use std::fs;
use tempfile::tempdir;

fn build_tree(file_count: usize) -> tempfile::TempDir {
    let dir = tempdir().unwrap();
    for i in 0..file_count {
        let sub = dir.path().join(format!("b{}", i % 50));
        fs::create_dir_all(&sub).unwrap();
        let body = if i % 10 == 0 {
            "needle in a haystack\n"
        } else {
            "hay hay hay\n"
        };
        fs::write(sub.join(format!("{i}.txt")), body).unwrap();
    }
    dir
}

fn bench_seq_vs_par(c: &mut Criterion) {
    let mut group = c.benchmark_group("scan_seq_vs_par");
    for &n in &[1_000usize, 10_000] {
        let dir = build_tree(n);
        let root = dir.path().to_path_buf();
        let cfg = SearchConfig::default();

        group.bench_with_input(BenchmarkId::new("sequential", n), &n, |b, _| {
            b.iter(|| {
                let results = run_sequential("needle", &root, cfg).unwrap();
                assert!(results.total_hits() > 0);
            });
        });

        group.bench_with_input(BenchmarkId::new("parallel", n), &n, |b, _| {
            b.iter(|| {
                let results = run_parallel("needle", &root, cfg).unwrap();
                assert!(results.total_hits() > 0);
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_seq_vs_par);
criterion_main!(benches);
