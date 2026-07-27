use crate::errors::AppError;
use std::path::Path;
use zet::core::usecases::create as create_usecase;
use zet::storage::local_repo::LocalMarkdownRepo;

pub fn create_note(notes_dir: &Path, title: &str, tags: Option<&str>) -> Result<(), AppError> {
    let repo = LocalMarkdownRepo::new(notes_dir);
    let note = create_usecase::create_note(&repo, title, tags)?;
    println!("note created: notes/{}.md", note.slug);
    Ok(())
}
