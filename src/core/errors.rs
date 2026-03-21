use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("title cannot be empty")]
    EmptyTitle,

    #[error("invalid tags: {0}")]
    InvalidTags(String),

    #[error("note not found: {0}")]
    NoteNotFound(String),

    #[error("invalid frontmatter for `{0}`: {1}")]
    InvalidFrontmatter(String, String),

    #[error("failed to serialize frontmatter: {0}")]
    FrontmatterSerialize(#[from] serde_yaml::Error),

    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("invalid slug: {0}")]
    InvalidSlug(String),
}
