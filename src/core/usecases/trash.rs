use crate::core::errors::CoreError;
use crate::core::note::Note;
use crate::core::repository::{NoteRepository, TrashCollection};

pub fn list_trash<R: NoteRepository>(repo: &R) -> Result<TrashCollection, CoreError> {
    repo.list_trash()
}

pub fn restore_trash<R: NoteRepository>(repo: &R, id: &str) -> Result<Note, CoreError> {
    repo.restore_trash(id)
}

pub fn purge_trash<R: NoteRepository>(repo: &R, id: &str) -> Result<(), CoreError> {
    repo.purge_trash(id)
}
