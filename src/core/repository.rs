use crate::core::errors::CoreError;
use crate::core::note::Note;

pub trait NoteRepository {
    fn note_exists(&self, slug: &str) -> Result<bool, CoreError>;
    fn save_note(&self, note: &Note) -> Result<(), CoreError>;
    fn read_note(&self, slug: &str) -> Result<Note, CoreError>;
    fn list_notes(&self) -> Result<Vec<Note>, CoreError>;
    fn list_note_slugs(&self) -> Result<Vec<String>, CoreError>;
    fn delete_note(&self, slug: &str) -> Result<(), CoreError>;
    fn read_raw_note(&self, slug: &str) -> Result<String, CoreError>;
    fn write_raw_note(&self, slug: &str, content: &str) -> Result<(), CoreError>;
}
