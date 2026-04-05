use chrono::NaiveDate;

use crate::core::errors::CoreError;
use crate::core::repository::NoteRepository;
use crate::core::validators::{dedup_tags, has_duplicate_tags};

#[derive(Debug, Clone)]
pub struct LintIssue {
    pub slug: String,
    pub message: String,
}

pub fn lint_note_by_slug<R: NoteRepository>(
    repo: &R,
    slug: &str,
    fix: bool,
) -> Result<Vec<LintIssue>, CoreError> {
    let mut issues = Vec::new();

    let mut note = match repo.read_note(slug) {
        Ok(note) => note,
        Err(err) => {
            issues.push(issue(slug, &err.to_string()));
            return Ok(issues);
        }
    };

    let mut changed = false;

    if note.title.trim().is_empty() {
        issues.push(issue(slug, "Missing required field `title`"))
    }

    if note.date.trim().is_empty() {
        issues.push(issue(slug, "Missing required field `date`"))
    } else if invalid_date(&note.date) {
        issues.push(issue(
            &note.slug,
            "Invalid `date` format. Expected YYYY-MM-DD",
        ))
    }

    if has_duplicate_tags(&note.tags) {
        issues.push(issue(slug, "Duplicate tags found"));

        if fix {
            dedup_tags(&mut note.tags);
            changed = true;
        }
    }

    if fix && changed {
        repo.save_note(&note)?;
    }

    Ok(issues)
}

fn issue(slug: &str, message: &str) -> LintIssue {
    LintIssue {
        slug: slug.to_string(),
        message: message.to_string(),
    }
}

fn invalid_date(date: &str) -> bool {
    NaiveDate::parse_from_str(date, "%Y-%m-%d").is_err()
}
