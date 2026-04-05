use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use zet::core::errors::CoreError;
use zet::core::note::Note;
use zet::core::repository::NoteRepository;
use zet::storage::local_repo::LocalMarkdownRepo;

fn test_notes_dir() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();

    std::env::temp_dir().join(format!(
        "zet-local-repo-integration-{}-{}",
        std::process::id(),
        nanos
    ))
}

fn cleanup(dir: &Path) {
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn save_and_read_note_roundtrip() {
    let notes_dir = test_notes_dir();
    let repo = LocalMarkdownRepo::new(&notes_dir);

    let note = Note {
        slug: "roundtrip-note".to_string(),
        title: "Roundtrip Note".to_string(),
        date: "2026-04-04".to_string(),
        tags: vec!["rust".to_string(), "integration".to_string()],
        updated_at: "2026-04-04".to_string(),
        body: "\n\n# Roundtrip\n\nBody content.\n".to_string(),
    };

    repo.save_note(&note).expect("save should succeed");
    let loaded = repo.read_note(&note.slug).expect("read should succeed");

    assert_eq!(loaded.slug, note.slug);
    assert_eq!(loaded.title, note.title);
    assert_eq!(loaded.date, note.date);
    assert_eq!(loaded.tags, note.tags);
    assert_eq!(loaded.updated_at, note.updated_at);
    assert_eq!(loaded.body, note.body);

    cleanup(&notes_dir);
}

#[test]
fn list_notes_returns_saved_notes() {
    let notes_dir = test_notes_dir();
    let repo = LocalMarkdownRepo::new(&notes_dir);

    let note_one = Note {
        slug: "alpha".to_string(),
        title: "Alpha".to_string(),
        date: "2026-04-04".to_string(),
        tags: vec![],
        updated_at: "2026-04-04".to_string(),
        body: "alpha body".to_string(),
    };
    let note_two = Note {
        slug: "beta".to_string(),
        title: "Beta".to_string(),
        date: "2026-04-04".to_string(),
        tags: vec!["tag".to_string()],
        updated_at: "2026-04-04".to_string(),
        body: "beta body".to_string(),
    };

    repo.save_note(&note_one)
        .expect("save note_one should succeed");
    repo.save_note(&note_two)
        .expect("save note_two should succeed");

    let notes = repo.list_notes().expect("list should succeed");
    let mut slugs: Vec<String> = notes.into_iter().map(|n| n.slug).collect();
    slugs.sort();

    assert_eq!(slugs, vec!["alpha".to_string(), "beta".to_string()]);

    cleanup(&notes_dir);
}

#[test]
fn read_missing_note_returns_not_found() {
    let notes_dir = test_notes_dir();
    let repo = LocalMarkdownRepo::new(&notes_dir);

    let err = repo.read_note("missing-note").unwrap_err();
    assert!(matches!(err, CoreError::NoteNotFound(_)));

    cleanup(&notes_dir);
}

#[test]
fn note_path_uses_slug_md_naming() {
    let notes_dir = test_notes_dir();
    let repo = LocalMarkdownRepo::new(&notes_dir);

    let path = repo.note_path("my-slug");
    let path_string = path.to_string_lossy();
    assert!(path_string.ends_with("my-slug.md"));

    cleanup(&notes_dir);
}
