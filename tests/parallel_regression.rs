//! Regression: sequential and parallel pipelines must produce identical results
//! and identical formatted output for the same inputs and flags.

use rustscan::output::{CountFormatter, Formatter, LineFormatter};
use rustscan::pipeline::{SearchConfig, run_parallel, run_sequential};
use std::fs;
use tempfile::tempdir;

fn build_fixture() -> tempfile::TempDir {
    let dir = tempdir().unwrap();
    for i in 0..100 {
        let sub = dir.path().join(format!("bucket_{}", i % 10));
        fs::create_dir_all(&sub).unwrap();
        let body = match i % 5 {
            0 => "hello world\n",
            1 => "HELLO\nline2\n",
            2 => "nope\n",
            3 => "hello\nhello\n",
            _ => "other\n",
        };
        fs::write(sub.join(format!("file_{i}.txt")), body).unwrap();
        // Hidden entries (ignored unless --hidden)
        if i % 20 == 0 {
            fs::write(sub.join(format!(".hid_{i}")), "hello hidden\n").unwrap();
        }
    }
    dir
}

#[test]
fn sequential_equals_parallel_default_config() {
    let dir = build_fixture();
    let cfg = SearchConfig::default();
    let seq = run_sequential("hello", dir.path(), cfg).unwrap();
    let par = run_parallel("hello", dir.path(), cfg).unwrap();
    assert_eq!(seq, par);
}

#[test]
fn sequential_equals_parallel_ignore_case() {
    let dir = build_fixture();
    let cfg = SearchConfig {
        ignore_case: true,
        ..SearchConfig::default()
    };
    let seq = run_sequential("hello", dir.path(), cfg).unwrap();
    let par = run_parallel("hello", dir.path(), cfg).unwrap();
    assert_eq!(seq, par);
}

#[test]
fn sequential_equals_parallel_with_hidden() {
    let dir = build_fixture();
    let cfg = SearchConfig {
        include_hidden: true,
        ..SearchConfig::default()
    };
    let seq = run_sequential("hello", dir.path(), cfg).unwrap();
    let par = run_parallel("hello", dir.path(), cfg).unwrap();
    assert_eq!(seq, par);
}

#[test]
fn formatted_line_output_identical() {
    let dir = build_fixture();
    let cfg = SearchConfig {
        ignore_case: true,
        show_line_number: true,
        ..SearchConfig::default()
    };
    let seq = run_sequential(r"hell\w+", dir.path(), cfg).unwrap();
    let par = run_parallel(r"hell\w+", dir.path(), cfg).unwrap();
    let fmt = LineFormatter {
        show_line_number: true,
    };
    assert_eq!(fmt.format_string(&seq), fmt.format_string(&par));
}

#[test]
fn formatted_count_output_identical() {
    let dir = build_fixture();
    let cfg = SearchConfig {
        count_only: true,
        ignore_case: true,
        include_hidden: true,
        ..SearchConfig::default()
    };
    let seq = run_sequential("hello", dir.path(), cfg).unwrap();
    let par = run_parallel("hello", dir.path(), cfg).unwrap();
    assert_eq!(
        CountFormatter.format_string(&seq),
        CountFormatter.format_string(&par)
    );
}

/// Run the same search several times in parallel mode; output stays stable.
#[test]
fn parallel_is_deterministic_across_runs() {
    let dir = build_fixture();
    let cfg = SearchConfig::default();
    let a = run_parallel("hello", dir.path(), cfg).unwrap();
    let b = run_parallel("hello", dir.path(), cfg).unwrap();
    let c = run_parallel("hello", dir.path(), cfg).unwrap();
    assert_eq!(a, b);
    assert_eq!(b, c);
}
