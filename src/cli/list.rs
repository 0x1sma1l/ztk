use crate::{cli::output, errors::AppError};
use std::path::Path;
use ztk::core::repository::NoteLoadIssue;
use ztk::core::usecases::list as list_usecase;
use ztk::storage::local_repo::LocalMarkdownRepo;

pub fn list_notes(notes_dir: &Path) -> Result<(), AppError> {
    let repo = LocalMarkdownRepo::new(notes_dir);
    let collection = list_usecase::list_notes(&repo)?;

    if collection.notes.is_empty() {
        if collection.issues.is_empty() {
            println!(
                "{} {}",
                output::accent("No notes found."),
                output::muted("Create one with `ztk new <title>`.")
            );
        } else {
            println!("{}", output::accent("No readable notes found."));
        }
    } else {
        let number_width = collection.notes.len().to_string().len().max(1);
        let slug_width = collection
            .notes
            .iter()
            .map(|note| note.slug.len())
            .max()
            .unwrap_or_default()
            .max("SLUG".len());

        println!(
            "{}  {}\n",
            output::accent("Notes"),
            output::muted(format!("{} total", collection.notes.len()))
        );
        println!(
            "  {}  {}  {}  {}",
            output::muted(format!("{:>number_width$}", "#")),
            output::muted(format!("{:<slug_width$}", "SLUG")),
            output::muted("DATE"),
            output::muted("TAGS")
        );
        for (i, note) in collection.notes.iter().enumerate() {
            let tags = if note.tags.is_empty() {
                "—".to_string()
            } else {
                note.tags.join(", ")
            };

            println!(
                "  {}  {}  {}  {}",
                output::muted(format!("{:>number_width$}", i + 1)),
                output::strong(format!("{:<slug_width$}", note.slug)),
                output::muted(note.date),
                output::muted(tags)
            );
        }
    }

    print_load_warnings(&collection.issues);

    Ok(())
}

fn print_load_warnings(issues: &[NoteLoadIssue]) {
    for issue in issues {
        output::warning(format_args!("skipped {}.md: {}", issue.slug, issue.message));
    }

    if !issues.is_empty() {
        output::warning(format_args!(
            "skipped {} unreadable note(s); run `ztk lint` for a complete integrity check",
            issues.len()
        ));
    }
}
