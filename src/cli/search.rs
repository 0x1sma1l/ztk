use crate::errors::AppError;
use crate::fuzzy;
use std::path::Path;

pub fn search_notes(notes_dir: &Path, query: Option<&str>) -> Result<(), AppError> {
    let selection = fuzzy::select_note(notes_dir, query)?;
    for issue in &selection.issues {
        eprintln!("warning: skipped {}.md: {}", issue.slug, issue.message);
    }
    if !selection.issues.is_empty() {
        eprintln!(
            "warning: skipped {} unreadable note(s); run `ztk lint` for a complete integrity check",
            selection.issues.len()
        );
    }

    if selection.candidate_count == 0 {
        println!("No readable notes available to search.");
        return Ok(());
    }

    match selection.slug {
        Some(slug) => crate::cli::edit::edit_note(notes_dir, &slug),
        None => Ok(()),
    }
}
