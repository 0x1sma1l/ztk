use crate::core::{errors::CoreError, repository::NoteRepository, validators::validate_slug};

pub fn delete_note<R: NoteRepository>(repo: &R, slug: &str) -> Result<(), CoreError> {
    let slug = validate_slug(slug)?;
    repo.delete_note(slug)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::core::errors::CoreError;
    use crate::core::note::Note;
    use crate::core::repository::NoteRepository;
    use crate::storage::local_repo::LocalMarkdownRepo;

    use super::delete_note;

    fn test_notes_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("zet-delete-tests-{}-{}", std::process::id(), nanos))
    }

    #[test]
    fn delete_note_deletes_existing_note() {
        let notes_dir = test_notes_dir();
        let repo = LocalMarkdownRepo::new(&notes_dir);

        let note = Note {
            slug: "delete-me".to_string(),
            title: "Delete Me".to_string(),
            date: "2026-03-21".to_string(),
            tags: vec![],
            updated_at: "2026-03-21".to_string(),
            body: "body".to_string(),
        };

        repo.save_note(&note).expect("failed to save setup note");
        assert!(repo.note_path("delete-me").exists());

        let result = delete_note(&repo, "delete-me");

        assert!(result.is_ok());
        assert!(!repo.note_path("delete-me").exists());
        let _ = fs::remove_dir_all(&notes_dir);
    }

    #[test]
    fn delete_note_returns_not_found_when_missing() {
        let notes_dir = test_notes_dir();
        let repo = LocalMarkdownRepo::new(&notes_dir);

        let err = delete_note(&repo, "missing-note").unwrap_err();

        match err {
            CoreError::NoteNotFound(_) => {}
            other => panic!("expected NoteNotFound, got {other:?}"),
        }
        let _ = fs::remove_dir_all(&notes_dir);
    }
}
