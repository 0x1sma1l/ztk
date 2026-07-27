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

#[cfg(test)]
mod tests {
    use crate::core::errors::CoreError;
    use crate::core::usecases::test_support::{InMemoryNoteRepository, note};

    use super::lint_note_by_slug;

    #[test]
    fn lint_reports_policy_issues_without_writing_when_fix_is_disabled() {
        let repo = InMemoryNoteRepository::default();
        let mut invalid = note("invalid");
        invalid.title = " ".to_string();
        invalid.date = "27-07-2026".to_string();
        invalid.tags = vec!["rust".to_string(), "Rust".to_string()];
        repo.insert(invalid);

        let issues = lint_note_by_slug(&repo, "invalid", false).expect("lint should complete");

        assert_eq!(issues.len(), 3);
        assert!(issues.iter().any(|issue| issue.message.contains("title")));
        assert!(issues.iter().any(|issue| issue.message.contains("date")));
        assert!(
            issues
                .iter()
                .any(|issue| issue.message.contains("Duplicate tags"))
        );
        assert_eq!(repo.save_calls(), 0);
    }

    #[test]
    fn lint_fix_deduplicates_tags_case_insensitively() {
        let repo = InMemoryNoteRepository::default();
        let mut duplicate = note("duplicate");
        duplicate.tags = vec!["rust".to_string(), "Rust".to_string()];
        repo.insert(duplicate);

        let issues = lint_note_by_slug(&repo, "duplicate", true).expect("fix should complete");

        assert_eq!(issues.len(), 1);
        assert_eq!(repo.get("duplicate").unwrap().tags, vec!["rust"]);
        assert_eq!(repo.save_calls(), 1);
    }

    #[test]
    fn lint_turns_a_read_failure_into_a_file_issue() {
        let repo = InMemoryNoteRepository::default();
        repo.fail_reads();

        let issues = lint_note_by_slug(&repo, "unreadable", false)
            .expect("per-file read failures should be reported as issues");

        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("injected read failure"));
        assert_eq!(repo.save_calls(), 0);
    }

    #[test]
    fn lint_propagates_save_failure_during_fix() {
        let repo = InMemoryNoteRepository::default();
        let mut duplicate = note("duplicate");
        duplicate.tags = vec!["rust".to_string(), "Rust".to_string()];
        repo.insert(duplicate);
        repo.fail_saves();

        let error = lint_note_by_slug(&repo, "duplicate", true).unwrap_err();

        assert!(matches!(error, CoreError::Io(_)));
        assert_eq!(repo.save_calls(), 1);
    }
}
