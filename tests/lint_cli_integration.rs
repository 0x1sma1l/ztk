use std::fs;
use std::process::{Command, Output};

use tempfile::TempDir;

fn write_note(root: &TempDir, slug: &str, title: &str, date: &str, tags: &str) {
    let notes_dir = root.path().join("notes");
    fs::create_dir_all(&notes_dir).expect("failed to create notes directory");

    let content = format!(
        "---\ntitle: {title}\ndate: {date}\ntags: {tags}\nupdated_at: 2026-07-27\n---\n\n# Body\n"
    );

    fs::write(notes_dir.join(format!("{slug}.md")), content).expect("failed to write test note");
}

fn run_lint(root: &TempDir, fix: bool) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_ztk"));
    command
        .arg("lint")
        .env("ZTK_NOTES_DIR", root.path().join("notes"))
        .env_remove("ZTK_CONFIG")
        .current_dir(root.path());

    if fix {
        command.arg("--fix");
    }

    command.output().expect("failed to execute ztk lint")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn assert_no_ansi(output: &Output) {
    assert!(!output.stdout.windows(2).any(|bytes| bytes == b"\x1b["));
    assert!(!output.stderr.windows(2).any(|bytes| bytes == b"\x1b["));
}

#[test]
fn lint_succeeds_when_all_notes_are_clean() {
    let root = TempDir::new().expect("failed to create temp dir");
    write_note(&root, "clean", "Clean", "2026-07-27", "[rust]");

    let output = run_lint(&root, false);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_no_ansi(&output);
    assert!(stdout(&output).contains("Done: 1 files, 0 fixed, 0 failed"));
}

#[test]
fn lint_without_fix_fails_when_an_issue_exists() {
    let root = TempDir::new().expect("failed to create temp dir");
    write_note(&root, "missing-title", "''", "2026-07-27", "[]");

    let output = run_lint(&root, false);

    assert!(!output.status.success());
    assert!(stdout(&output).contains("Done: 1 files, 0 fixed, 1 failed"));
    assert!(stderr(&output).contains("Lint failed: 1 file(s) contain issues"));
}

#[test]
fn lint_with_fix_succeeds_when_all_issues_are_fixed() {
    let root = TempDir::new().expect("failed to create temp dir");
    write_note(
        &root,
        "duplicate-tags",
        "Duplicate Tags",
        "2026-07-27",
        "[rust, Rust]",
    );

    let output = run_lint(&root, true);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(stdout(&output).contains("Done: 1 files, 1 fixed, 0 failed"));

    let second_pass = run_lint(&root, false);
    assert!(
        second_pass.status.success(),
        "fixed note should pass a later lint: {}",
        stderr(&second_pass)
    );
}

#[test]
fn lint_with_fix_fails_when_an_unfixable_issue_remains() {
    let root = TempDir::new().expect("failed to create temp dir");
    write_note(&root, "invalid-date", "Invalid Date", "27-07-2026", "[]");

    let output = run_lint(&root, true);

    assert!(!output.status.success());
    assert!(stdout(&output).contains("Done: 1 files, 0 fixed, 1 failed"));
    assert!(stdout(&output).contains("Invalid `date` date `27-07-2026`"));
    assert!(stderr(&output).contains("Lint failed: 1 file(s) contain issues"));
}

#[test]
fn lint_with_fix_fails_for_mixed_fixed_and_unfixable_notes() {
    let root = TempDir::new().expect("failed to create temp dir");
    write_note(
        &root,
        "duplicate-tags",
        "Duplicate Tags",
        "2026-07-27",
        "[rust, Rust]",
    );
    write_note(&root, "invalid-date", "Invalid Date", "27-07-2026", "[]");

    let output = run_lint(&root, true);

    assert!(!output.status.success());
    assert!(stdout(&output).contains("Done: 2 files, 1 fixed, 1 failed"));
    assert!(stderr(&output).contains("Lint failed: 1 file(s) contain issues"));
}
