use crate::fuzzy;
use crate::{cli::output, errors::AppError};
use std::path::Path;

pub fn search_notes(notes_dir: &Path, query: Option<&str>) -> Result<(), AppError> {
    let selection = fuzzy::select_note(notes_dir, query)?;
    for issue in &selection.issues {
        output::warning(format_args!("skipped {}.md: {}", issue.slug, issue.message));
    }
    if !selection.issues.is_empty() {
        output::warning(format_args!(
            "skipped {} unreadable note(s); run `ztk lint` for a complete integrity check",
            selection.issues.len()
        ));
    }

    if selection.candidate_count == 0 {
        println!(
            "{}",
            output::accent("No readable notes available to search.")
        );
        return Ok(());
    }

    match selection.slug {
        Some(slug) => crate::cli::edit::edit_note(notes_dir, &slug),
        None => Ok(()),
    }
}
