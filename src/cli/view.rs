use std::path::Path;
use termimad::MadSkin;

use crate::errors::AppError;
use zet::core::usecases::read as read_usecase;
use zet::storage::local_repo::LocalMarkdownRepo;

pub fn view_note(notes_dir: &Path, slug: &str) -> Result<(), AppError> {
    let repo = LocalMarkdownRepo::new(notes_dir);
    let note = read_usecase::read_note(&repo, slug)?;

    let skin = MadSkin::default();
    println!("Viewing: notes/{}.md\n", slug);
    skin.print_text(&note.body);

    Ok(())
}
