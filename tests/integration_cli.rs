//! Integration tests exercising the `rustscan` binary end-to-end.

use std::fs;
use std::process::Command;
use tempfile::tempdir;

fn bin() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_rustscan"));
    cmd.env_remove("RUST_LOG");
    cmd
}

#[test]
fn searches_literal_pattern() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("a.txt"), "hello world\n").unwrap();
    fs::write(dir.path().join("b.txt"), "nope\n").unwrap();

    let output = bin()
        .arg("hello")
        .arg(dir.path())
        .output()
        .expect("run rustscan");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("hello world"));
    assert!(!stdout.contains("nope"));
}

#[test]
fn ignore_case_and_line_numbers() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("c.txt"), "Hello\n").unwrap();

    let output = bin()
        .args(["-i", "-n", "hello"])
        .arg(dir.path())
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(":1:Hello"));
}

#[test]
fn count_mode() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("d.txt"), "x\nx\n").unwrap();

    let output = bin().args(["-c", "x"]).arg(dir.path()).output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.trim().ends_with(":2"));
}

#[test]
fn invalid_regex_exits_nonzero() {
    let dir = tempdir().unwrap();
    let output = bin().arg("(").arg(dir.path()).output().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("invalid regular expression"));
}

#[test]
fn hidden_skipped_unless_flag() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join(".secret"), "secret\n").unwrap();
    fs::write(dir.path().join("vis.txt"), "secret\n").unwrap();

    let without = bin().arg("secret").arg(dir.path()).output().unwrap();
    let with = bin()
        .args(["--hidden", "secret"])
        .arg(dir.path())
        .output()
        .unwrap();
    let s0 = String::from_utf8_lossy(&without.stdout);
    let s1 = String::from_utf8_lossy(&with.stdout);
    assert!(s0.contains("vis.txt"));
    assert!(!s0.contains(".secret"));
    assert!(s1.contains(".secret"));
}

#[test]
fn regex_metacharacter_dot() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("a.txt"), "abc\n").unwrap();
    fs::write(dir.path().join("b.txt"), "a.c\n").unwrap();

    let output = bin().arg("a.c").arg(dir.path()).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("abc"));
    assert!(stdout.contains("a.c"));
}
