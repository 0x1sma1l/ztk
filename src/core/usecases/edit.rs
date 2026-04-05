use crate::core::errors::CoreError;
use crate::core::repository::NoteRepository;
use chrono::Local;

pub fn update_note_content<R: NoteRepository>(repo: &R, slug: &str) -> Result<(), CoreError> {
    let mut note = repo.read_note(slug)?;
    note.updated_at = Local::now().format("%Y-%m-%d").to_string();
    repo.save_note(&note)?;

    Ok(())
}
