use std::fs;

use tempfile::TempDir;

use zet::core::errors::CoreError;
use zet::core::note::Note;
use zet::core::repository::NoteRepository;
use zet::storage::local_repo::LocalMarkdownRepo;

#[test]
fn save_and_read_note_roundtrip() {
    let notes_dir = TempDir::new().expect("failed to create temp dir");
    let repo = LocalMarkdownRepo::new(notes_dir);

    let note = Note {
        slug: "roundtrip-note".to_string(),
        title: "Roundtrip Note".to_string(),
        date: "2026-04-04".parse().unwrap(),
        tags: vec!["rust".to_string(), "integration".to_string()],
        updated_at: "2026-04-04".parse().unwrap(),
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
}

#[test]
fn list_notes_returns_saved_notes() {
    let notes_dir = TempDir::new().expect("failed to create temp dir");
    let repo = LocalMarkdownRepo::new(notes_dir);

    let note_one = Note {
        slug: "alpha".to_string(),
        title: "Alpha".to_string(),
        date: "2026-04-04".parse().unwrap(),
        tags: vec![],
        updated_at: "2026-04-04".parse().unwrap(),
        body: "alpha body".to_string(),
    };
    let note_two = Note {
        slug: "beta".to_string(),
        title: "Beta".to_string(),
        date: "2026-04-04".parse().unwrap(),
        tags: vec!["tag".to_string()],
        updated_at: "2026-04-04".parse().unwrap(),
        body: "beta body".to_string(),
    };

    repo.save_note(&note_one)
        .expect("save note_one should succeed");
    repo.save_note(&note_two)
        .expect("save note_two should succeed");

    let collection = repo.list_notes().expect("list should succeed");
    let mut slugs: Vec<String> = collection.notes.into_iter().map(|n| n.slug).collect();
    slugs.sort();

    assert_eq!(slugs, vec!["alpha".to_string(), "beta".to_string()]);
    assert!(collection.issues.is_empty());
}

#[test]
fn read_missing_note_returns_not_found() {
    let notes_dir = TempDir::new().expect("failed to create temp dir");
    let repo = LocalMarkdownRepo::new(notes_dir);

    let err = repo.read_note("missing-note").unwrap_err();
    assert!(matches!(err, CoreError::NoteNotFound(_)));
}

#[test]
fn note_path_uses_slug_md_naming() {
    let notes_dir = TempDir::new().expect("failed to create temp dir");
    let repo = LocalMarkdownRepo::new(notes_dir);

    let path = repo
        .note_path("my-slug")
        .expect("valid slug should produce a path");
    let path_string = path.to_string_lossy();
    assert!(path_string.ends_with("my-slug.md"));
}

#[test]
fn save_note_rejects_path_traversal_slug() {
    let root = TempDir::new().expect("failed to create temp dir");
    let notes_dir = root.path().join("notes");
    let repo = LocalMarkdownRepo::new(&notes_dir);

    let note = Note {
        slug: "../outside".to_string(),
        title: "Outside".to_string(),
        date: "2026-07-27".parse().unwrap(),
        tags: vec![],
        updated_at: "2026-07-27".parse().unwrap(),
        body: "must remain inside notes".to_string(),
    };

    let result = repo.save_note(&note);

    assert!(matches!(result, Err(CoreError::InvalidSlug(_))));
    assert!(!root.path().join("outside.md").exists());
    assert!(!notes_dir.exists());
}

#[test]
fn repository_operations_reject_invalid_slugs() {
    let notes_dir = TempDir::new().expect("failed to create temp dir");
    let repo = LocalMarkdownRepo::new(notes_dir);

    for invalid_slug in ["../outside", "a/b", "/absolute", ".", "..", "a_b", "a\\b"] {
        assert!(matches!(
            repo.note_path(invalid_slug),
            Err(CoreError::InvalidSlug(_))
        ));
        assert!(matches!(
            repo.note_exists(invalid_slug),
            Err(CoreError::InvalidSlug(_))
        ));
        assert!(matches!(
            repo.read_note(invalid_slug),
            Err(CoreError::InvalidSlug(_))
        ));
        assert!(matches!(
            repo.ensure_note_exists(invalid_slug),
            Err(CoreError::InvalidSlug(_))
        ));
        assert!(matches!(
            repo.trash_note(invalid_slug).map(|_| ()),
            Err(CoreError::InvalidSlug(_))
        ));
        assert!(matches!(
            repo.read_raw_note(invalid_slug),
            Err(CoreError::InvalidSlug(_))
        ));
    }
}

#[test]
fn repository_reads_legacy_note_without_updated_at() {
    let notes_dir = TempDir::new().expect("failed to create temp dir");
    let repo = LocalMarkdownRepo::new(&notes_dir);
    let legacy_path = notes_dir.path().join("legacy-note.md");

    std::fs::write(
        legacy_path,
        "---\ntitle: Legacy Note\ndate: 2026-04-04\ntags: [legacy]\n---\n\nLegacy body.\n",
    )
    .expect("failed to write legacy note");

    let note = repo
        .read_note("legacy-note")
        .expect("legacy note should remain readable");

    assert_eq!(note.title, "Legacy Note");
    assert_eq!(note.date.to_string(), "2026-04-04");
    assert_eq!(note.updated_at.to_string(), "2026-04-04");
    assert_eq!(note.tags, vec!["legacy"]);
    assert_eq!(note.body.trim_start_matches('\n'), "Legacy body.\n");
}

#[test]
fn repository_rejects_invalid_updated_at_with_field_context() {
    let notes_dir = TempDir::new().unwrap();
    fs::write(
        notes_dir.path().join("invalid-updated.md"),
        "---\ntitle: Invalid Updated\ndate: 2026-07-27\ntags: []\nupdated_at: tomorrow\n---\n\nBody\n",
    )
    .unwrap();
    let repo = LocalMarkdownRepo::new(notes_dir.path());

    let error = repo.read_note("invalid-updated").unwrap_err();

    assert!(
        error
            .to_string()
            .contains("Invalid `updated_at` date `tomorrow`")
    );
}

#[test]
fn list_notes_returns_readable_notes_and_malformed_file_issues() {
    let notes_dir = TempDir::new().expect("failed to create temp dir");
    let repo = LocalMarkdownRepo::new(&notes_dir);

    let valid = Note {
        slug: "valid-note".to_string(),
        title: "Valid Note".to_string(),
        date: "2026-07-27".parse().unwrap(),
        tags: vec![],
        updated_at: "2026-07-27".parse().unwrap(),
        body: "Valid body".to_string(),
    };
    repo.save_note(&valid).expect("valid note should save");
    std::fs::write(notes_dir.path().join("broken-note.md"), "not frontmatter")
        .expect("failed to write malformed note");

    let collection = repo
        .list_notes()
        .expect("malformed notes should be collected as issues");

    assert_eq!(collection.notes.len(), 1);
    assert_eq!(collection.notes[0].slug, "valid-note");
    assert_eq!(collection.issues.len(), 1);
    assert_eq!(collection.issues[0].slug, "broken-note");
    assert!(collection.issues[0].message.contains("No frontmatter"));
}
