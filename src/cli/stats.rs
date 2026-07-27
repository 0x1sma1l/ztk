use crate::errors::AppError;
use zet::core::usecases::list as list_usecase;
use zet::storage::local_repo::LocalMarkdownRepo;

pub fn get_stats() -> Result<(), AppError> {
    let repo = LocalMarkdownRepo::default();
    let collection = list_usecase::list_notes(&repo)?;
    let note_count = collection.notes.len();
    println!("Total notes: {}", note_count);

    if !collection.issues.is_empty() {
        eprintln!(
            "warning: skipped {} unreadable note(s); run `zet lint` for details",
            collection.issues.len()
        );
    }

    Ok(())
}
