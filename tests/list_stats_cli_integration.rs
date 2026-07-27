use std::fs;
use std::process::{Command, Output};

use tempfile::TempDir;

fn write_valid_note(root: &TempDir) {
    let notes_dir = root.path().join("notes");
    fs::create_dir_all(&notes_dir).expect("failed to create notes directory");
    fs::write(
        notes_dir.join("valid-note.md"),
        "---\ntitle: Valid Note\ndate: 2026-07-27\ntags: [valid]\nupdated_at: 2026-07-27\n---\n\nBody\n",
    )
    .expect("failed to write valid note");
}

fn write_broken_note(root: &TempDir) {
    let notes_dir = root.path().join("notes");
    fs::create_dir_all(&notes_dir).expect("failed to create notes directory");
    fs::write(notes_dir.join("broken-note.md"), "not frontmatter")
        .expect("failed to write broken note");
}

fn run(root: &TempDir, command: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ztk"))
        .arg(command)
        .env("ZTK_NOTES_DIR", root.path().join("notes"))
        .env_remove("ZTK_CONFIG")
        .current_dir(root.path())
        .output()
        .expect("failed to execute ztk command")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn list_prints_valid_notes_and_warns_about_malformed_notes() {
    let root = TempDir::new().expect("failed to create temp dir");
    write_valid_note(&root);
    write_broken_note(&root);

    let output = run(&root, "list");

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(stdout(&output).contains("valid-note"));
    assert!(stderr(&output).contains("skipped broken-note.md"));
    assert!(stderr(&output).contains("skipped 1 unreadable note(s)"));
}

#[test]
fn stats_counts_readable_notes_and_reports_skipped_notes() {
    let root = TempDir::new().expect("failed to create temp dir");
    write_valid_note(&root);
    write_broken_note(&root);

    let output = run(&root, "stats");

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "Total notes: 1\n");
    assert!(stderr(&output).contains("skipped 1 unreadable note(s)"));
}

#[test]
fn list_distinguishes_all_unreadable_notes_from_an_empty_repository() {
    let root = TempDir::new().expect("failed to create temp dir");
    write_broken_note(&root);

    let unreadable = run(&root, "list");
    assert!(unreadable.status.success());
    assert!(stdout(&unreadable).contains("No readable notes found."));
    assert!(stderr(&unreadable).contains("skipped 1 unreadable note(s)"));

    let empty_root = TempDir::new().expect("failed to create empty temp dir");
    let empty = run(&empty_root, "list");
    assert!(empty.status.success());
    assert!(stdout(&empty).contains("No notes found."));
    assert!(stderr(&empty).is_empty());
}

#[test]
fn list_and_stats_fail_when_notes_path_is_not_a_directory() {
    let root = TempDir::new().expect("failed to create temp dir");
    fs::write(root.path().join("notes"), "this path should be a directory")
        .expect("failed to create invalid notes path");

    for command in ["list", "stats"] {
        let output = run(&root, command);

        assert!(!output.status.success(), "{command} should fail");
        assert!(stderr(&output).contains("failed to create notes directory"));
    }
}
