use std::fs;
use std::process::{Command, Output};

use tempfile::TempDir;

fn ztk(root: &TempDir, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ztk"))
        .args(args)
        .env("ZTK_NOTES_DIR", root.path().join("notes"))
        .env_remove("ZTK_CONFIG")
        .env_remove("VISUAL")
        .env("EDITOR", "true")
        .current_dir(root.path())
        .output()
        .expect("failed to execute ztk command")
}

fn ztk_with_editor(root: &TempDir, slug: &str, editor: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ztk"))
        .args(["edit", slug])
        .env("ZTK_NOTES_DIR", root.path().join("notes"))
        .env_remove("ZTK_CONFIG")
        .env_remove("VISUAL")
        .env("EDITOR", editor)
        .current_dir(root.path())
        .output()
        .expect("failed to execute ztk edit")
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

    let output = ztk(&root, &["new", "Rust Ownership", "--tags", "rust,learning"]);

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

    let first = ztk(&root, &["new", "Same Title"]);
    let second = ztk(&root, &["new", "Same Title"]);

    assert!(first.status.success(), "stderr: {}", stderr(&first));
    assert!(second.status.success(), "stderr: {}", stderr(&second));
    assert!(root.path().join("notes/same-title.md").exists());
    assert!(root.path().join("notes/same-title-1.md").exists());
    assert!(stdout(&second).contains("notes/same-title-1.md"));
}

#[test]
fn new_rejects_invalid_tags_without_creating_a_note() {
    let root = TempDir::new().expect("failed to create temp dir");

    let output = ztk(&root, &["new", "Invalid Tags", "--tags", "rust,bad!"]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("Invalid tags"));
    assert!(!root.path().join("notes/invalid-tags.md").exists());
}

#[test]
fn new_keeps_the_created_note_when_the_editor_fails() {
    let root = TempDir::new().expect("failed to create temp dir");
    let output = Command::new(env!("CARGO_BIN_EXE_ztk"))
        .args(["new", "Editor Failure"])
        .env("ZTK_NOTES_DIR", root.path().join("notes"))
        .env_remove("ZTK_CONFIG")
        .env_remove("VISUAL")
        .env("EDITOR", "false")
        .current_dir(root.path())
        .output()
        .expect("failed to execute ztk new");

    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("Editor exited with a non-zero status"));
    assert!(root.path().join("notes/editor-failure.md").is_file());
}

#[test]
fn view_renders_an_existing_note_body() {
    let root = TempDir::new().expect("failed to create temp dir");
    write_note(&root, "view-me", "# Visible Heading\n\nVisible body text.");

    let output = ztk(&root, &["view", "view-me"]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let output_text = stdout(&output);
    assert!(output_text.contains("Viewing: notes/view-me.md"));
    assert!(output_text.contains("Visible Heading"));
    assert!(output_text.contains("Visible body text."));
}

#[test]
fn view_reports_missing_and_invalid_slugs() {
    let root = TempDir::new().expect("failed to create temp dir");

    let missing = ztk(&root, &["view", "missing"]);
    assert!(!missing.status.success());
    assert!(stderr(&missing).contains("Note not found"));

    let invalid = ztk(&root, &["view", "../outside"]);
    assert!(!invalid.status.success());
    assert!(stderr(&invalid).contains("Invalid slug"));
}

#[test]
fn update_changes_structured_fields_without_renaming_the_note() {
    let root = TempDir::new().expect("failed to create temp dir");
    write_note(&root, "stable-slug", "Original body.");

    let output = ztk(
        &root,
        &[
            "update",
            "stable-slug",
            "--title",
            "New title",
            "--tags",
            "rust,Rust,cli",
            "--body",
            "New body.",
        ],
    );

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(stdout(&output).contains("note updated: notes/stable-slug.md"));
    assert!(!root.path().join("notes/new-title.md").exists());
    let content = fs::read_to_string(root.path().join("notes/stable-slug.md")).unwrap();
    assert!(content.contains("title: New title"));
    assert!(content.contains("- rust"));
    assert!(content.contains("- cli"));
    assert_eq!(content.matches("- rust").count(), 1);
    assert!(content.contains("New body."));
}

#[test]
fn update_can_clear_tags_and_body() {
    let root = TempDir::new().expect("failed to create temp dir");
    write_note(&root, "clear-me", "Original body.");

    let output = ztk(&root, &["update", "clear-me", "--tags=", "--body="]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let content = fs::read_to_string(root.path().join("notes/clear-me.md")).unwrap();
    assert!(content.contains("tags: []"));
    assert!(!content.contains("Original body."));
}

#[test]
fn invalid_or_no_op_updates_do_not_rewrite_the_note() {
    let root = TempDir::new().expect("failed to create temp dir");
    write_note(&root, "unchanged", "Original body.");
    let path = root.path().join("notes/unchanged.md");
    let before = fs::read_to_string(&path).unwrap();

    let invalid_title = ztk(&root, &["update", "unchanged", "--title", "   "]);
    assert!(!invalid_title.status.success());
    assert!(stderr(&invalid_title).contains("Title cannot be empty"));
    assert_eq!(fs::read_to_string(&path).unwrap(), before);

    let invalid_tags = ztk(&root, &["update", "unchanged", "--tags", "bad!"]);
    assert!(!invalid_tags.status.success());
    assert!(stderr(&invalid_tags).contains("Invalid tags"));
    assert_eq!(fs::read_to_string(&path).unwrap(), before);

    let no_op = ztk(&root, &["update", "unchanged"]);
    assert!(no_op.status.success(), "stderr: {}", stderr(&no_op));
    assert!(stdout(&no_op).contains("note unchanged"));
    assert_eq!(fs::read_to_string(&path).unwrap(), before);
}

#[test]
fn edit_succeeds_when_editor_exits_zero() {
    let root = TempDir::new().expect("failed to create temp dir");
    write_note(&root, "edit-me", "Body before editor.");

    let output = ztk_with_editor(&root, "edit-me", "true");

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

    let output = ztk_with_editor(&root, "edit-me", "false");

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

    let output = ztk_with_editor(&root, "missing", "true");

    assert!(!output.status.success());
    assert!(stderr(&output).contains("Note not found"));
}

#[test]
fn visual_takes_precedence_over_editor() {
    let root = TempDir::new().expect("failed to create temp dir");
    write_note(&root, "edit-me", "Original body.");

    let output = Command::new(env!("CARGO_BIN_EXE_ztk"))
        .args(["edit", "edit-me"])
        .env("ZTK_NOTES_DIR", root.path().join("notes"))
        .env_remove("ZTK_CONFIG")
        .env("VISUAL", "true")
        .env("EDITOR", "false")
        .current_dir(root.path())
        .output()
        .expect("failed to execute ztk edit");

    assert!(output.status.success(), "stderr: {}", stderr(&output));
}

#[test]
fn edit_reports_empty_and_malformed_editor_commands() {
    let root = TempDir::new().expect("failed to create temp dir");
    write_note(&root, "edit-me", "Original body.");

    let empty = ztk_with_editor(&root, "edit-me", "");
    assert!(!empty.status.success());
    assert!(stderr(&empty).contains("$EDITOR is set but empty"));

    let malformed = ztk_with_editor(&root, "edit-me", "editor 'unclosed");
    assert!(!malformed.status.success());
    assert!(stderr(&malformed).contains("Could not parse $EDITOR"));
}

#[test]
fn edit_reports_a_missing_editor_executable() {
    let root = TempDir::new().expect("failed to create temp dir");
    write_note(&root, "edit-me", "Original body.");

    let output = ztk_with_editor(&root, "edit-me", "ztk-editor-that-does-not-exist");

    assert!(!output.status.success());
    assert!(stderr(&output).contains("Failed to launch editor"));
    assert!(stderr(&output).contains("ztk-editor-that-does-not-exist"));
}

#[cfg(unix)]
#[test]
fn edit_supports_quoted_executable_paths_and_flags() {
    use std::os::unix::fs::PermissionsExt;

    let root = TempDir::new().expect("failed to create temp dir");
    write_note(&root, "edit-me", "Original body.");
    let editor_path = root.path().join("editor with spaces");
    let argument_log = root.path().join("editor-arguments.txt");

    fs::write(
        &editor_path,
        "#!/bin/sh\nprintf '%s\\n' \"$@\" > \"$EDITOR_ARGUMENT_LOG\"\n",
    )
    .expect("failed to write fake editor");
    let mut permissions = fs::metadata(&editor_path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&editor_path, permissions).expect("failed to make fake editor executable");

    let editor_command = format!("'{}' --wait --reuse-window", editor_path.display());
    let output = Command::new(env!("CARGO_BIN_EXE_ztk"))
        .args(["edit", "edit-me"])
        .env("ZTK_NOTES_DIR", root.path().join("notes"))
        .env_remove("ZTK_CONFIG")
        .env_remove("VISUAL")
        .env("EDITOR", editor_command)
        .env("EDITOR_ARGUMENT_LOG", &argument_log)
        .current_dir(root.path())
        .output()
        .expect("failed to execute ztk edit");

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let arguments = fs::read_to_string(argument_log).expect("fake editor should log arguments");
    let arguments = arguments.lines().collect::<Vec<_>>();
    assert_eq!(&arguments[..2], ["--wait", "--reuse-window"]);
    assert!(arguments[2].ends_with(".md"));
}

#[cfg(unix)]
#[test]
fn invalid_editor_output_does_not_overwrite_the_original_note() {
    use std::os::unix::fs::PermissionsExt;

    let root = TempDir::new().expect("failed to create temp dir");
    write_note(&root, "edit-me", "Original body.");
    let note_path = root.path().join("notes/edit-me.md");
    let before = fs::read_to_string(&note_path).unwrap();
    let editor_path = root.path().join("invalid-editor");
    fs::write(&editor_path, "#!/bin/sh\nprintf 'invalid' > \"$1\"\n").unwrap();
    let mut permissions = fs::metadata(&editor_path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&editor_path, permissions).unwrap();

    let output = ztk_with_editor(&root, "edit-me", editor_path.to_str().unwrap());

    assert!(!output.status.success());
    assert_eq!(fs::read_to_string(note_path).unwrap(), before);
}
