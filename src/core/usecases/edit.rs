use crate::core::errors::CoreError;
use crate::core::note::{Note, NoteDate};
use crate::core::repository::NoteRepository;
use crate::core::validators::{dedup_tags, validate_tag_values};

#[derive(Debug, Clone, Default)]
pub struct UpdateNoteRequest {
    pub title: Option<String>,
    pub tags: Option<Vec<String>>,
    pub body: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UpdateNoteResult {
    pub note: Note,
    pub changed: bool,
}

pub fn update_note<R: NoteRepository>(
    repo: &R,
    slug: &str,
    request: UpdateNoteRequest,
) -> Result<UpdateNoteResult, CoreError> {
    update_note_at(repo, slug, request, NoteDate::today_local())
}

fn update_note_at<R: NoteRepository>(
    repo: &R,
    slug: &str,
    request: UpdateNoteRequest,
    updated_at: NoteDate,
) -> Result<UpdateNoteResult, CoreError> {
    let mut note = repo.read_note(slug)?;

    let title = request
        .title
        .map(|title| {
            let title = title.trim();
            if title.is_empty() {
                Err(CoreError::EmptyTitle)
            } else {
                Ok(title.to_string())
            }
        })
        .transpose()?;

    let tags = request
        .tags
        .map(|tags| {
            let mut tags = validate_tag_values(tags.iter().map(String::as_str))?;
            dedup_tags(&mut tags);
            Ok::<_, CoreError>(tags)
        })
        .transpose()?;

    let changed = title.as_ref().is_some_and(|title| title != &note.title)
        || tags.as_ref().is_some_and(|tags| tags != &note.tags)
        || request.body.as_ref().is_some_and(|body| body != &note.body);

    if !changed {
        return Ok(UpdateNoteResult { note, changed });
    }

    if let Some(title) = title {
        note.title = title;
    }
    if let Some(tags) = tags {
        note.tags = tags;
    }
    if let Some(body) = request.body {
        note.body = body;
    }
    note.updated_at = updated_at;
    repo.save_note(&note)?;

    Ok(UpdateNoteResult { note, changed })
}

#[cfg(test)]
mod tests {
    use crate::core::errors::CoreError;
    use crate::core::usecases::test_support::{InMemoryNoteRepository, note};

    use super::{UpdateNoteRequest, update_note_at};

    #[test]
    fn update_can_change_each_structured_field_together() {
        let repo = InMemoryNoteRepository::default();
        let mut existing = note("edit-me");
        existing.updated_at = "2020-01-01".parse().unwrap();
        repo.insert(existing);

        let result = update_note_at(
            &repo,
            "edit-me",
            UpdateNoteRequest {
                title: Some(" Updated title ".to_string()),
                tags: Some(vec![
                    "rust".to_string(),
                    "Rust".to_string(),
                    "cli".to_string(),
                ]),
                body: Some("Updated body".to_string()),
            },
            "2026-07-27".parse().unwrap(),
        )
        .expect("update should succeed");

        let saved = repo.get("edit-me").expect("updated note should be saved");
        assert!(result.changed);
        assert_eq!(saved.slug, "edit-me");
        assert_eq!(saved.title, "Updated title");
        assert_eq!(saved.tags, vec!["rust", "cli"]);
        assert_eq!(saved.body, "Updated body");
        assert_eq!(saved.updated_at.to_string(), "2026-07-27");
        assert_eq!(repo.read_calls(), 1);
        assert_eq!(repo.save_calls(), 1);
    }

    #[test]
    fn update_supports_each_field_independently() {
        for request in [
            UpdateNoteRequest {
                title: Some("New title".into()),
                ..Default::default()
            },
            UpdateNoteRequest {
                tags: Some(vec!["new-tag".into()]),
                ..Default::default()
            },
            UpdateNoteRequest {
                body: Some("New body".into()),
                ..Default::default()
            },
        ] {
            let repo = InMemoryNoteRepository::default();
            repo.insert(note("edit-me"));
            assert!(
                update_note_at(&repo, "edit-me", request, "2026-07-28".parse().unwrap())
                    .unwrap()
                    .changed
            );
            assert_eq!(repo.save_calls(), 1);
        }
    }

    #[test]
    fn no_op_does_not_save_or_change_updated_at() {
        let repo = InMemoryNoteRepository::default();
        let existing = note("edit-me");
        let original_updated_at = existing.updated_at;
        repo.insert(existing);

        let result = update_note_at(
            &repo,
            "edit-me",
            UpdateNoteRequest::default(),
            "2099-01-01".parse().unwrap(),
        )
        .unwrap();

        assert!(!result.changed);
        assert_eq!(result.note.updated_at, original_updated_at);
        assert_eq!(repo.save_calls(), 0);
    }

    #[test]
    fn invalid_changes_do_not_write() {
        for request in [
            UpdateNoteRequest {
                title: Some("   ".into()),
                ..Default::default()
            },
            UpdateNoteRequest {
                tags: Some(vec!["invalid tag!".into()]),
                ..Default::default()
            },
        ] {
            let repo = InMemoryNoteRepository::default();
            repo.insert(note("edit-me"));
            assert!(
                update_note_at(&repo, "edit-me", request, "2026-07-28".parse().unwrap()).is_err()
            );
            assert_eq!(repo.save_calls(), 0);
        }
    }

    #[test]
    fn update_does_not_save_when_read_fails() {
        let repo = InMemoryNoteRepository::default();
        repo.fail_reads();

        let error = update_note_at(
            &repo,
            "edit-me",
            UpdateNoteRequest::default(),
            "2026-07-28".parse().unwrap(),
        )
        .unwrap_err();

        assert!(matches!(error, CoreError::Io(_)));
        assert_eq!(repo.save_calls(), 0);
    }

    #[test]
    fn update_propagates_save_failure() {
        let repo = InMemoryNoteRepository::default();
        repo.insert(note("edit-me"));
        repo.fail_saves();

        let error = update_note_at(
            &repo,
            "edit-me",
            UpdateNoteRequest {
                body: Some("changed".into()),
                ..Default::default()
            },
            "2026-07-28".parse().unwrap(),
        )
        .unwrap_err();

        assert!(matches!(error, CoreError::Io(_)));
        assert_eq!(repo.save_calls(), 1);
    }
}
