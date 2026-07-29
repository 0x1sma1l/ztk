use std::path::Path;
use termimad::MadSkin;

use crate::{cli::output, errors::AppError};
use ztk::core::usecases::read as read_usecase;
use ztk::storage::local_repo::LocalMarkdownRepo;

pub fn view_note(notes_dir: &Path, slug: &str) -> Result<(), AppError> {
    let repo = LocalMarkdownRepo::new(notes_dir);
    let note = read_usecase::read_note(&repo, slug)?;

    let skin = MadSkin::default();
    println!(
        "{} {}\n",
        output::accent("Viewing:"),
        output::strong(format!("notes/{slug}.md"))
    );
    skin.print_text(&note.body);

    Ok(())
}
