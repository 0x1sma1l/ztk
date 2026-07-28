use std::fs;
use std::process::{Command, Output};

use tempfile::TempDir;

fn write_note(root: &TempDir, slug: &str) {
    let notes_dir = root.path().join("notes");
    fs::create_dir_all(&notes_dir).expect("failed to create notes directory");
    fs::write(
        notes_dir.join(format!("{slug}.md")),
        format!(
            "---\ntitle: {slug}\ndate: 2026-07-28\ntags: []\nupdated_at: 2026-07-28\n---\n\nBody\n"
        ),
    )
    .expect("failed to write note");
}

fn run_search_without_fzf(root: &TempDir, args: &[&str]) -> Output {
    let empty_path = root.path().join("empty-path");
    fs::create_dir_all(&empty_path).unwrap();
    Command::new(env!("CARGO_BIN_EXE_ztk"))
        .args(args)
        .env("ZTK_NOTES_DIR", root.path().join("notes"))
        .env_remove("ZTK_CONFIG")
        .env("PATH", empty_path)
        .current_dir(root.path())
        .output()
        .expect("failed to execute ztk search")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn search_accepts_no_query_and_reports_a_missing_fzf() {
    let root = TempDir::new().expect("failed to create temp dir");
    write_note(&root, "rust");

    let output = run_search_without_fzf(&root, &["search"]);

    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("requires `fzf`"));
    assert!(stderr(&output).contains("brew install fzf"));
}

#[test]
fn search_accepts_an_optional_initial_query() {
    let root = TempDir::new().expect("failed to create temp dir");
    write_note(&root, "rust");

    let output = run_search_without_fzf(&root, &["search", "ownership"]);

    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("requires `fzf`"));
}

#[test]
fn empty_repository_does_not_require_fzf() {
    let root = TempDir::new().expect("failed to create temp dir");

    let output = run_search_without_fzf(&root, &["search"]);

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(stdout(&output).contains("No readable notes available to search"));
}

#[test]
fn unreadable_notes_are_reported_before_search() {
    let root = TempDir::new().expect("failed to create temp dir");
    fs::create_dir_all(root.path().join("notes")).unwrap();
    fs::write(root.path().join("notes/broken.md"), "not frontmatter").unwrap();

    let output = run_search_without_fzf(&root, &["search"]);

    assert!(output.status.success());
    assert!(stderr(&output).contains("skipped broken.md"));
    assert!(stdout(&output).contains("No readable notes available to search"));
}
