use std::fs;
use std::process::{Command, Output};

use tempfile::TempDir;

fn write_note(root: &TempDir, slug: &str) {
    let notes_dir = root.path().join("notes");
    fs::create_dir_all(&notes_dir).expect("failed to create notes directory");
    fs::write(
        notes_dir.join(format!("{slug}.md")),
        "---\ntitle: Delete Me\ndate: 2026-07-27\ntags: []\nupdated_at: 2026-07-27\n---\n\nBody\n",
    )
    .expect("failed to write test note");
}

fn run_delete(root: &TempDir, slug: &str, force: bool) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_ztk"));
    command
        .arg("delete")
        .arg(slug)
        .env("ZTK_NOTES_DIR", root.path().join("notes"))
        .env_remove("ZTK_CONFIG")
        .current_dir(root.path());

    if force {
        command.arg("--force");
    }

    command.output().expect("failed to execute ztk delete")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn force_moves_note_to_recoverable_trash_without_interactive_confirmation() {
    let root = TempDir::new().expect("failed to create temp dir");
    write_note(&root, "delete-me");
    let note_path = root.path().join("notes/delete-me.md");

    let output = run_delete(&root, "delete-me", true);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(stdout(&output).contains("note moved to trash: delete-me"));
    assert!(!note_path.exists());
    let trash = root.path().join("notes/.trash");
    assert_eq!(fs::read_dir(trash).unwrap().count(), 2);
}

#[test]
fn non_interactive_delete_requires_force_and_preserves_note() {
    let root = TempDir::new().expect("failed to create temp dir");
    write_note(&root, "keep-me");
    let note_path = root.path().join("notes/keep-me.md");

    let output = run_delete(&root, "keep-me", false);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("Re-run with `--force`"));
    assert!(note_path.exists());
}

#[test]
fn missing_note_is_reported_before_confirmation_is_requested() {
    let root = TempDir::new().expect("failed to create temp dir");

    let output = run_delete(&root, "missing", false);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("Note not found"));
    assert!(!stderr(&output).contains("requires confirmation"));
}

#[test]
fn invalid_slug_is_reported_before_confirmation_is_requested() {
    let root = TempDir::new().expect("failed to create temp dir");

    let output = run_delete(&root, "../outside", false);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("Invalid slug"));
    assert!(!stderr(&output).contains("requires confirmation"));
}
