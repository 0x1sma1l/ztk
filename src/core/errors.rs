use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("Title cannot be empty")]
    EmptyTitle,

    #[error("Invalid tags: {0}")]
    InvalidTags(String),

    #[error("Note not found: {0}")]
    NoteNotFound(String),

    #[error("No frontmatter found in note")]
    EmptyFrontmatter,

    #[error("Failed to serialize frontmatter: {0}")]
    FrontmatterSerialize(serde_yaml::Error),

    #[error("Failed to parse frontmatter: {0}")]
    FrontmatterParse(serde_yaml::Error),

    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("Invalid slug: {0}")]
    InvalidSlug(String),
}
