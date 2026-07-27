use crate::errors::AppError;
use std::path::Path;
use ztk::core::usecases::search as search_usecase;
use ztk::storage::local_repo::LocalMarkdownRepo;

pub fn search_notes(notes_dir: &Path, query: &str) -> Result<(), AppError> {
    let repo = LocalMarkdownRepo::new(notes_dir);
    let results = search_usecase::search_notes(&repo, query)?;

    if results.matches.is_empty() {
        println!("No notes matched `{}`.", query.trim());
    } else {
        println!("Search results (score | slug | title | tags):\n");
        for result in results.matches {
            let tags = if result.tags.is_empty() {
                "-".to_string()
            } else {
                result.tags.join(", ")
            };
            println!(
                "{} | {} | {} | {}",
                result.score, result.slug, result.title, tags
            );
        }
    }

    for issue in &results.issues {
        eprintln!("warning: skipped {}.md: {}", issue.slug, issue.message);
    }
    if !results.issues.is_empty() {
        eprintln!(
            "warning: skipped {} unreadable note(s); run `ztk lint` for a complete integrity check",
            results.issues.len()
        );
    }

    Ok(())
}
