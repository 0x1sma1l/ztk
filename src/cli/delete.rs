use crate::core::usecases::delete::delete_note as delete_note_usecase;
use crate::{errors::AppError, storage::local_repo::LocalMarkdownRepo};

pub fn delete_note(slug: &str) -> Result<(), AppError> {
    let repo = LocalMarkdownRepo::default();
    delete_note_usecase(&repo, slug)?;

    println!("note deleted: notes/{}.md", slug);

    Ok(())
}
