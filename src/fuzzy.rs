use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use crate::errors::AppError;
use ztk::core::repository::{NoteLoadIssue, NoteRepository};
use ztk::storage::local_repo::LocalMarkdownRepo;

pub struct FuzzySelection {
    pub slug: Option<String>,
    pub issues: Vec<NoteLoadIssue>,
    pub candidate_count: usize,
}

pub fn select_note(
    notes_dir: &Path,
    initial_query: Option<&str>,
) -> Result<FuzzySelection, AppError> {
    let collection = LocalMarkdownRepo::new(notes_dir).list_notes()?;
    if collection.notes.is_empty() {
        return Ok(FuzzySelection {
            slug: None,
            issues: collection.issues,
            candidate_count: 0,
        });
    }

    let mut command = Command::new("fzf");
    command.args([
        "--delimiter=\t",
        "--with-nth=2..",
        "--no-multi",
        "--prompt=Notes> ",
    ]);
    if let Some(query) = initial_query.filter(|query| !query.is_empty()) {
        command.arg("--query").arg(query);
    }
    command.stdin(Stdio::piped()).stdout(Stdio::piped());

    let mut child = command.spawn().map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            AppError::FzfNotInstalled
        } else {
            AppError::Io(error)
        }
    })?;

    {
        let mut input = child.stdin.take().expect("fzf stdin is piped");
        for note in &collection.notes {
            writeln!(
                input,
                "{}\t{}\t{}",
                note.slug,
                sanitize(&note.title),
                sanitize(&note.tags.join(", "))
            )?;
        }
    }

    let output = child.wait_with_output()?;
    let slug = if output.status.success() {
        parse_slug(&output.stdout)
    } else if matches!(output.status.code(), Some(1 | 130)) {
        None
    } else {
        return Err(AppError::FzfFailed(output.status.code()));
    };

    let candidate_count = collection.notes.len();

    Ok(FuzzySelection {
        slug,
        issues: collection.issues,
        candidate_count,
    })
}

fn sanitize(value: &str) -> String {
    value
        .chars()
        .map(|character| match character {
            '\t' | '\r' | '\n' => ' ',
            other => other,
        })
        .collect()
}

fn parse_slug(output: &[u8]) -> Option<String> {
    let line = std::str::from_utf8(output).ok()?.lines().next()?;
    let slug = line.split('\t').next()?.trim();
    (!slug.is_empty()).then(|| slug.to_string())
}

#[cfg(test)]
mod tests {
    use super::{parse_slug, sanitize};

    #[test]
    fn selection_extracts_only_the_slug_column() {
        assert_eq!(
            parse_slug(b"rust-ownership\tRust Ownership\trust, notes\n"),
            Some("rust-ownership".to_string())
        );
        assert_eq!(parse_slug(b""), None);
        assert_eq!(parse_slug(&[0xff]), None);
    }

    #[test]
    fn candidate_fields_cannot_inject_rows_or_columns() {
        assert_eq!(sanitize("title\tpart\nnext"), "title part next");
    }
}
