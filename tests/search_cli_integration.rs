use std::fs;
use std::process::{Command, Output};

use tempfile::TempDir;

fn write_note(root: &TempDir, slug: &str, title: &str, tags: &str) {
    let notes_dir = root.path().join("notes");
    fs::create_dir_all(&notes_dir).expect("failed to create notes directory");
    fs::write(
        notes_dir.join(format!("{slug}.md")),
        format!(
            "---\ntitle: {title}\ndate: 2026-07-27\ntags: {tags}\nupdated_at: 2026-07-27\n---\n\nBody\n"
        ),
    )
    .expect("failed to write note");
}

fn run_search(root: &TempDir, query: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ztk"))
        .args(["search", query])
        .env("ZTK_NOTES_DIR", root.path().join("notes"))
        .env_remove("ZTK_CONFIG")
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
fn search_returns_ranked_slug_title_and_tag_matches() {
    let root = TempDir::new().expect("failed to create temp dir");
    write_note(&root, "rust", "Rust", "[language]");
    write_note(&root, "rusty-tools", "Rusty Tools", "[rust, tooling]");
    write_note(&root, "garden", "Garden Notes", "[plants]");

    let output = run_search(&root, "rust");

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let output = stdout(&output);
    let exact_position = output.find("| rust | Rust |").unwrap();
    let fuzzy_position = output.find("| rusty-tools | Rusty Tools |").unwrap();
    assert!(exact_position < fuzzy_position);
    assert!(!output.contains("garden"));
}

#[test]
fn search_reports_no_matches_without_failing() {
    let root = TempDir::new().expect("failed to create temp dir");
    write_note(&root, "garden", "Garden Notes", "[plants]");

    let output = run_search(&root, "zzzzzz");

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(stdout(&output).contains("No notes matched `zzzzzz`."));
}

#[test]
fn search_rejects_an_empty_query() {
    let root = TempDir::new().expect("failed to create temp dir");

    let output = run_search(&root, "   ");

    assert!(!output.status.success());
    assert!(stderr(&output).contains("Search query cannot be empty"));
}

#[test]
fn search_returns_matches_and_warns_about_unreadable_notes() {
    let root = TempDir::new().expect("failed to create temp dir");
    write_note(&root, "readable", "Readable Note", "[]");
    fs::write(root.path().join("notes/broken.md"), "not frontmatter")
        .expect("failed to write broken note");

    let output = run_search(&root, "readable");

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert!(stdout(&output).contains("| readable | Readable Note |"));
    assert!(stderr(&output).contains("skipped broken.md"));
}
