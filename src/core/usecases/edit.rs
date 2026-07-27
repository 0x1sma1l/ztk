use crate::core::errors::CoreError;
use crate::core::repository::NoteRepository;
use chrono::Local;

pub fn update_note_content<R: NoteRepository>(repo: &R, slug: &str) -> Result<(), CoreError> {
    let mut note = repo.read_note(slug)?;
    note.updated_at = Local::now().format("%Y-%m-%d").to_string();
    repo.save_note(&note)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use chrono::Local;

    use crate::core::errors::CoreError;
    use crate::core::usecases::test_support::{InMemoryNoteRepository, note};

    use super::update_note_content;

    #[test]
    fn update_sets_current_date_and_persists_note() {
        let repo = InMemoryNoteRepository::default();
        let mut existing = note("edit-me");
        existing.updated_at = "2020-01-01".to_string();
        repo.insert(existing);

        update_note_content(&repo, "edit-me").expect("update should succeed");

        let saved = repo.get("edit-me").expect("updated note should be saved");
        assert_eq!(
            saved.updated_at,
            Local::now().format("%Y-%m-%d").to_string()
        );
        assert_eq!(repo.read_calls(), 1);
        assert_eq!(repo.save_calls(), 1);
    }

    #[test]
    fn update_does_not_save_when_read_fails() {
        let repo = InMemoryNoteRepository::default();
        repo.fail_reads();

        let error = update_note_content(&repo, "edit-me").unwrap_err();

        assert!(matches!(error, CoreError::Io(_)));
        assert_eq!(repo.save_calls(), 0);
    }

    #[test]
    fn update_propagates_save_failure() {
        let repo = InMemoryNoteRepository::default();
        repo.insert(note("edit-me"));
        repo.fail_saves();

        let error = update_note_content(&repo, "edit-me").unwrap_err();

        assert!(matches!(error, CoreError::Io(_)));
        assert_eq!(repo.save_calls(), 1);
    }
}
