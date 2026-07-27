use crate::errors::AppError;
use std::path::Path;
use zet::core::repository::NoteLoadIssue;
use zet::core::usecases::list as list_usecase;
use zet::storage::local_repo::LocalMarkdownRepo;

pub fn list_notes(notes_dir: &Path) -> Result<(), AppError> {
    let repo = LocalMarkdownRepo::new(notes_dir);
    let collection = list_usecase::list_notes(&repo)?;

    if collection.notes.is_empty() {
        if collection.issues.is_empty() {
            println!("No notes found. Try creating one first with `zet new <title>`.");
        } else {
            println!("No readable notes found.");
        }
    } else {
        println!("Notes (slug | date | tags):\n");
        for (i, note) in collection.notes.iter().enumerate() {
            let tags_display = if note.tags.is_empty() {
                String::new()
            } else {
                format!(" | tags: {}", note.tags.join(", "))
            };

            println!("{}. {} | {}{}", i + 1, note.slug, note.date, tags_display);
        }
    }

    print_load_warnings(&collection.issues);

    Ok(())
}

fn print_load_warnings(issues: &[NoteLoadIssue]) {
    for issue in issues {
        eprintln!("warning: skipped {}.md: {}", issue.slug, issue.message);
    }

    if !issues.is_empty() {
        eprintln!(
            "warning: skipped {} unreadable note(s); run `zet lint` for a complete integrity check",
            issues.len()
        );
    }
}
