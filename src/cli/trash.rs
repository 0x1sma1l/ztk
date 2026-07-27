use std::path::Path;

use crate::errors::AppError;
use zet::core::usecases::trash;
use zet::storage::local_repo::LocalMarkdownRepo;

pub fn list(notes_dir: &Path) -> Result<(), AppError> {
    let collection = trash::list_trash(&LocalMarkdownRepo::new(notes_dir))?;
    if collection.entries.is_empty() {
        println!("Trash is empty.");
    } else {
        println!("Trash (id | original slug | deleted at):");
        for entry in collection.entries {
            println!(
                "{} | {} | {}",
                entry.id, entry.original_slug, entry.deleted_at
            );
        }
    }
    for issue in collection.issues {
        eprintln!(
            "warning: skipped trash metadata {}: {}",
            issue.slug, issue.message
        );
    }
    Ok(())
}

pub fn restore(notes_dir: &Path, id: &str) -> Result<(), AppError> {
    let note = trash::restore_trash(&LocalMarkdownRepo::new(notes_dir), id)?;
    println!("note restored: {}.md", note.slug);
    Ok(())
}

pub fn purge(notes_dir: &Path, id: &str, force: bool) -> Result<(), AppError> {
    if !force {
        return Err(AppError::PurgeConfirmationRequired);
    }
    trash::purge_trash(&LocalMarkdownRepo::new(notes_dir), id)?;
    println!("trash entry permanently purged: {id}");
    Ok(())
}
