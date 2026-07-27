use crate::core::errors::CoreError;
use crate::core::repository::{NoteCollection, NoteRepository};

pub fn list_notes<R: NoteRepository>(repo: &R) -> Result<NoteCollection, CoreError> {
    repo.list_notes()
}
