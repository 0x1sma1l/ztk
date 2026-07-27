use std::fs;
use std::process::{Command, Output};

use tempfile::TempDir;

fn zet(root: &TempDir, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_zet"))
        .args(args)
        .current_dir(root.path())
        .output()
        .expect("failed to execute zet command")
}

fn zet_with_editor(root: &TempDir, slug: &str, editor: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_zet"))
        .args(["edit", slug])
        .env("EDITOR", editor)
        .current_dir(root.path())
        .output()
        .expect("failed to execute zet edit")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn write_note(root: &TempDir, slug: &str, body: &str) {
    let notes_dir = root.path().join("notes");
    fs::create_dir_all(&notes_dir).expect("failed to create notes directory");
    fs::write(
        notes_dir.join(format!("{slug}.md")),
        format!(
            "---\ntitle: Test Note\ndate: 2026-07-27\ntags: [test]\nupdated_at: 2026-07-27\n---\n\n{body}\n"
        ),
    )
    .expect("failed to write test note");
}

#[test]
fn new_creates_a_note_with_validated_metadata() {
    let root = TempDir::new().expect("failed to create temp dir");

    let output = zet(&root, &["new", "Rust Ownership", "--tags", "rust,learning"]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(stdout(&output).contains("note created: notes/rust-ownership.md"));

    let content = fs::read_to_string(root.path().join("notes/rust-ownership.md"))
        .expect("created note should be readable");
    assert!(content.contains("title: Rust Ownership"));
    assert!(content.contains("- rust"));
    assert!(content.contains("- learning"));
    assert!(content.contains("updated_at:"));
    assert!(content.contains("# Rust Ownership"));
}

#[test]
fn new_uses_a_deterministic_suffix_when_slug_exists() {
    let root = TempDir::new().expect("failed to create temp dir");

    let first = zet(&root, &["new", "Same Title"]);
    let second = zet(&root, &["new", "Same Title"]);

    assert!(first.status.success(), "stderr: {}", stderr(&first));
    assert!(second.status.success(), "stderr: {}", stderr(&second));
    assert!(root.path().join("notes/same-title.md").exists());
    assert!(root.path().join("notes/same-title-1.md").exists());
    assert!(stdout(&second).contains("notes/same-title-1.md"));
}

#[test]
fn new_rejects_invalid_tags_without_creating_a_note() {
    let root = TempDir::new().expect("failed to create temp dir");

    let output = zet(&root, &["new", "Invalid Tags", "--tags", "rust,bad!"]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("Invalid tags"));
    assert!(!root.path().join("notes/invalid-tags.md").exists());
}

#[test]
fn view_renders_an_existing_note_body() {
    let root = TempDir::new().expect("failed to create temp dir");
    write_note(&root, "view-me", "# Visible Heading\n\nVisible body text.");

    let output = zet(&root, &["view", "view-me"]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let output_text = stdout(&output);
    assert!(output_text.contains("Viewing: notes/view-me.md"));
    assert!(output_text.contains("Visible Heading"));
    assert!(output_text.contains("Visible body text."));
}

#[test]
fn view_reports_missing_and_invalid_slugs() {
    let root = TempDir::new().expect("failed to create temp dir");

    let missing = zet(&root, &["view", "missing"]);
    assert!(!missing.status.success());
    assert!(stderr(&missing).contains("Note not found"));

    let invalid = zet(&root, &["view", "../outside"]);
    assert!(!invalid.status.success());
    assert!(stderr(&invalid).contains("Invalid slug"));
}

#[test]
fn edit_succeeds_when_editor_exits_zero() {
    let root = TempDir::new().expect("failed to create temp dir");
    write_note(&root, "edit-me", "Body before editor.");

    let output = zet_with_editor(&root, "edit-me", "true");

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let content = fs::read_to_string(root.path().join("notes/edit-me.md"))
        .expect("edited note should remain readable");
    assert!(content.contains("updated_at:"));
    assert!(content.contains("Body before editor."));
}

#[test]
fn edit_reports_editor_failure_without_post_processing_note() {
    let root = TempDir::new().expect("failed to create temp dir");
    write_note(&root, "edit-me", "Original body.");
    let note_path = root.path().join("notes/edit-me.md");
    let before = fs::read_to_string(&note_path).expect("setup note should be readable");

    let output = zet_with_editor(&root, "edit-me", "false");

    assert!(!output.status.success());
    assert!(stderr(&output).contains("Editor exited with a non-zero status"));
    assert_eq!(
        fs::read_to_string(note_path).expect("note should remain readable"),
        before
    );
}

#[test]
fn edit_reports_a_missing_note_before_launching_editor() {
    let root = TempDir::new().expect("failed to create temp dir");

    let output = zet_with_editor(&root, "missing", "true");

    assert!(!output.status.success());
    assert!(stderr(&output).contains("Note not found"));
}
