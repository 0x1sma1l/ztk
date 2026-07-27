use crate::errors::AppError;
use std::path::Path;
use ztk::core::usecases::list as list_usecase;
use ztk::storage::local_repo::LocalMarkdownRepo;

pub fn get_stats(notes_dir: &Path) -> Result<(), AppError> {
    let repo = LocalMarkdownRepo::new(notes_dir);
    let collection = list_usecase::list_notes(&repo)?;
    let note_count = collection.notes.len();
    println!("Total notes: {}", note_count);

    if !collection.issues.is_empty() {
        eprintln!(
            "warning: skipped {} unreadable note(s); run `ztk lint` for details",
            collection.issues.len()
        );
    }

    Ok(())
}
