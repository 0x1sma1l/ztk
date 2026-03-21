use crate::core::{errors::CoreError, repository::NoteRepository, validators::validate_slug};

pub fn delete_note<R: NoteRepository>(repo: &R, slug: &str) -> Result<(), CoreError> {
    let slug = validate_slug(slug)?;
    repo.delete_note(&slug)
}
