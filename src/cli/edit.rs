use std::{env, process::Command};

use crate::core::usecases::read as read_usecase;
use crate::errors::AppError;
use crate::storage::local_repo::LocalMarkdownRepo;

pub fn edit_note(slug: &str) -> Result<(), AppError> {
    let repo = LocalMarkdownRepo::default();
    read_usecase::read_note(&repo, slug)?;

    let editor = env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
    let status = Command::new(editor).arg(repo.note_path(slug)).status()?;

    if !status.success() {
        return Err(AppError::EditorExitedWithError);
    }

    Ok(())
}
