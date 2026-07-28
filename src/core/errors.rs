use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("Title cannot be empty")]
    EmptyTitle,

    #[error("Invalid tags: {0}")]
    InvalidTags(String),

    #[error("Invalid `{field}` date `{value}`. Expected YYYY-MM-DD")]
    InvalidDate { field: &'static str, value: String },

    #[error("Note not found: {0}")]
    NoteNotFound(String),

    #[error("No frontmatter found in note")]
    EmptyFrontmatter,

    #[error("Invalid frontmatter: {0}")]
    InvalidFrontmatter(String),

    #[error("Failed to serialize frontmatter: {0}")]
    FrontmatterSerialize(serde_yaml::Error),

    #[error("Failed to parse frontmatter: {0}")]
    FrontmatterParse(serde_yaml::Error),

    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("Invalid slug: {0}")]
    InvalidSlug(String),

    #[error("Invalid trash ID: {0}")]
    InvalidTrashId(String),

    #[error("Trash entry not found: {0}")]
    TrashEntryNotFound(String),

    #[error("Cannot restore `{slug}` because a note with that slug already exists")]
    RestoreCollision { slug: String },

    #[error("Repository does not support operation: {0}")]
    UnsupportedRepositoryOperation(&'static str),
}
