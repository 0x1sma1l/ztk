use crate::core::errors::CoreError;
use crate::core::note::Note;
use crate::core::repository::NoteRepository;
use crate::core::validators::validate_slug;

pub fn read_note<R: NoteRepository>(repo: &R, slug: &str) -> Result<Note, CoreError> {
    let slug = validate_slug(slug)?;
    repo.read_note(slug)
}

#[cfg(test)]
mod tests {
    use crate::core::errors::CoreError;
    use crate::core::usecases::test_support::{InMemoryNoteRepository, note};

    use super::read_note;

    #[test]
    fn read_returns_an_existing_note() {
        let repo = InMemoryNoteRepository::default();
        repo.insert(note("existing"));

        let loaded = read_note(&repo, "existing").expect("existing note should load");

        assert_eq!(loaded.slug, "existing");
        assert_eq!(repo.read_calls(), 1);
    }

    #[test]
    fn read_rejects_an_invalid_slug_before_calling_repository() {
        let repo = InMemoryNoteRepository::default();

        let error = read_note(&repo, "../outside").unwrap_err();

        assert!(matches!(error, CoreError::InvalidSlug(_)));
        assert_eq!(repo.read_calls(), 0);
    }

    #[test]
    fn read_propagates_repository_errors() {
        let repo = InMemoryNoteRepository::default();
        repo.fail_reads();

        let error = read_note(&repo, "valid-slug").unwrap_err();

        assert!(matches!(error, CoreError::Io(_)));
        assert_eq!(repo.read_calls(), 1);
    }
}
