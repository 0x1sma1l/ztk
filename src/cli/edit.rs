use std::{env, process::Command};

use crate::errors::AppError;
use zet::core::repository::NoteRepository;
use zet::core::usecases::edit::update_note_content;
use zet::core::validators::validate_slug;
use zet::storage::local_repo::LocalMarkdownRepo;

pub fn edit_note(slug: &str) -> Result<(), AppError> {
    let slug = validate_slug(slug)?;
    let repo = LocalMarkdownRepo::default();
    repo.ensure_note_exists(slug)?;

    let note_path = repo.note_path(slug)?;

    let editor = env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
    let status = Command::new(editor).arg(&note_path).status()?;

    if !status.success() {
        return Err(AppError::EditorExitedWithError);
    }

    update_note_content(&repo, slug)?;

    Ok(())
}
