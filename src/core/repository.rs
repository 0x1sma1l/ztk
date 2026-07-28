use crate::core::errors::CoreError;
use crate::core::note::Note;

#[derive(Debug, Clone, Default)]
pub struct NoteCollection {
    pub notes: Vec<Note>,
    pub issues: Vec<NoteLoadIssue>,
}

#[derive(Debug, Clone)]
pub struct NoteLoadIssue {
    pub slug: String,
    pub message: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TrashedNote {
    pub id: String,
    pub original_slug: String,
    pub deleted_at: String,
}

#[derive(Debug, Clone, Default)]
pub struct TrashCollection {
    pub entries: Vec<TrashedNote>,
    pub issues: Vec<NoteLoadIssue>,
}

pub trait NoteRepository {
    fn note_exists(&self, slug: &str) -> Result<bool, CoreError>;
    fn save_note(&self, note: &Note) -> Result<(), CoreError>;
    fn read_note(&self, slug: &str) -> Result<Note, CoreError>;
    fn ensure_note_exists(&self, slug: &str) -> Result<(), CoreError>;
    fn list_notes(&self) -> Result<NoteCollection, CoreError>;
    fn list_note_slugs(&self) -> Result<Vec<String>, CoreError>;
    fn trash_note(&self, slug: &str) -> Result<TrashedNote, CoreError> {
        let _ = slug;
        Err(CoreError::UnsupportedRepositoryOperation("trash"))
    }
    fn list_trash(&self) -> Result<TrashCollection, CoreError> {
        Err(CoreError::UnsupportedRepositoryOperation("list trash"))
    }
    fn restore_trash(&self, id: &str) -> Result<Note, CoreError> {
        let _ = id;
        Err(CoreError::UnsupportedRepositoryOperation("restore trash"))
    }
    fn purge_trash(&self, id: &str) -> Result<(), CoreError> {
        let _ = id;
        Err(CoreError::UnsupportedRepositoryOperation("purge trash"))
    }
    fn purge_all_trash(&self) -> Result<usize, CoreError> {
        Err(CoreError::UnsupportedRepositoryOperation("purge all trash"))
    }
    fn read_raw_note(&self, slug: &str) -> Result<String, CoreError>;
}
