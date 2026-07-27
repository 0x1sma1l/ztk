use crate::core::errors::CoreError;
use crate::core::note::{Note, NoteDate};
use crate::core::repository::NoteRepository;
use crate::core::validators::{slugify, validate_tags};

pub fn create_note<R: NoteRepository>(
    repo: &R,
    title: &str,
    raw_tags: Option<&str>,
) -> Result<Note, CoreError> {
    if title.trim().is_empty() {
        return Err(CoreError::EmptyTitle);
    }

    let tags = match raw_tags {
        Some(raw) => validate_tags(raw)?,
        None => Vec::new(),
    };

    let base_slug = slugify(title);
    let slug = unique_slug(repo, &base_slug)?;
    let date = NoteDate::today_local();
    let updated_at = date;

    let note = Note {
        slug,
        title: title.to_string(),
        date,
        tags,
        updated_at,
        body: format!("# {}\n\n<!-- Start writing your note below -->\n", title),
    };

    repo.save_note(&note)?;

    Ok(note)
}

fn unique_slug<R: NoteRepository>(repo: &R, base: &str) -> Result<String, CoreError> {
    let root = if base.is_empty() {
        "note".to_string()
    } else {
        base.to_string()
    };
    let mut candidate = root.clone();

    if !repo.note_exists(&candidate)? {
        return Ok(candidate);
    }

    let mut idx = 1;
    loop {
        candidate = format!("{}-{}", root, idx);
        if !repo.note_exists(&candidate)? {
            return Ok(candidate);
        }
        idx += 1;
    }
}

#[cfg(test)]
mod tests {
    use crate::core::errors::CoreError;
    use crate::core::usecases::test_support::{InMemoryNoteRepository, note};

    use super::create_note;

    #[test]
    fn create_validates_builds_and_saves_a_note() {
        let repo = InMemoryNoteRepository::default();

        let created = create_note(&repo, "Rust Ownership", Some("rust, learning"))
            .expect("valid note should be created");

        assert_eq!(created.slug, "rust-ownership");
        assert_eq!(created.tags, vec!["rust", "learning"]);
        assert_eq!(created.updated_at, created.date);
        assert!(created.body.contains("# Rust Ownership"));
        assert_eq!(repo.save_calls(), 1);
        assert!(repo.get("rust-ownership").is_some());
    }

    #[test]
    fn create_uses_the_first_available_slug_suffix() {
        let repo = InMemoryNoteRepository::default();
        repo.insert(note("same-title"));
        repo.insert(note("same-title-1"));

        let created = create_note(&repo, "Same Title", None).expect("collision should be resolved");

        assert_eq!(created.slug, "same-title-2");
    }

    #[test]
    fn create_rejects_an_empty_title_before_saving() {
        let repo = InMemoryNoteRepository::default();

        let error = create_note(&repo, "   ", None).unwrap_err();

        assert!(matches!(error, CoreError::EmptyTitle));
        assert_eq!(repo.note_count(), 0);
        assert_eq!(repo.save_calls(), 0);
    }

    #[test]
    fn create_propagates_repository_errors() {
        let repo = InMemoryNoteRepository::default();
        repo.fail_exists();

        let error = create_note(&repo, "Repository Failure", None).unwrap_err();

        assert!(matches!(error, CoreError::Io(_)));
        assert_eq!(repo.save_calls(), 0);
    }
}
