use crate::{cli::output, errors::AppError};
use std::path::Path;
use ztk::core::usecases::edit::{self, UpdateNoteRequest};
use ztk::storage::local_repo::LocalMarkdownRepo;

pub fn update_note(
    notes_dir: &Path,
    slug: &str,
    title: Option<&str>,
    tags: Option<&str>,
    body: Option<&str>,
) -> Result<(), AppError> {
    let repo = LocalMarkdownRepo::new(notes_dir);
    let result = edit::update_note(
        &repo,
        slug,
        UpdateNoteRequest {
            title: title.map(ToOwned::to_owned),
            tags: tags.map(|raw| raw.split(',').map(ToOwned::to_owned).collect()),
            body: body.map(ToOwned::to_owned),
        },
    )?;

    if result.changed {
        println!(
            "{} {}",
            output::accent("note updated:"),
            output::strong(format!("notes/{}.md", result.note.slug))
        );
    } else {
        println!(
            "{} {}",
            output::muted("note unchanged:"),
            output::strong(format!("notes/{}.md", result.note.slug))
        );
    }
    Ok(())
}
