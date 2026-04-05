use chrono::Local;

use crate::core::errors::CoreError;
use crate::core::note::Note;
use crate::core::repository::NoteRepository;
use crate::core::validators::{slugify, validate_tags};

pub fn create_note<R: NoteRepository>(
    repo: &R,
    title: &str,
    raw_tags: Option<&str>,
) -> Result<Note, CoreError> {
    if title.trim().is_empty() {
        return Err(CoreError::EmptyTitle);
    }

    let tags = match raw_tags {
        Some(raw) => validate_tags(raw)?,
        None => Vec::new(),
    };

    let base_slug = slugify(title);
    let slug = unique_slug(repo, &base_slug)?;
    let date = Local::now().format("%Y-%m-%d").to_string();
    let updated_at = date.clone();

    let note = Note {
        slug,
        title: title.to_string(),
        date,
        tags,
        updated_at,
        body: format!("# {}\n\n<!-- Start writing your note below -->\n", title),
    };

    repo.save_note(&note)?;

    Ok(note)
}

fn unique_slug<R: NoteRepository>(repo: &R, base: &str) -> Result<String, CoreError> {
    let root = if base.is_empty() {
        "note".to_string()
    } else {
        base.to_string()
    };
    let mut candidate = root.clone();

    if !repo.note_exists(&candidate)? {
        return Ok(candidate);
    }

    let mut idx = 1;
    loop {
        candidate = format!("{}-{}", root, idx);
        if !repo.note_exists(&candidate)? {
            return Ok(candidate);
        }
        idx += 1;
    }
}
