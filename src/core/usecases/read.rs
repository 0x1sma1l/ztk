use crate::core::errors::CoreError;
use crate::core::note::Note;
use crate::core::repository::NoteRepository;
use crate::core::validators::validate_slug;

pub fn read_note<R: NoteRepository>(repo: &R, slug: &str) -> Result<Note, CoreError> {
    let slug = validate_slug(slug)?;
    repo.read_note(&slug)
}
