use crate::{cli::output, errors::AppError};
use ztk::core::repository::NoteRepository;
use ztk::core::usecases::lint::{LintIssue, lint_note_by_slug};
use ztk::storage::local_repo::LocalMarkdownRepo;

use std::path::Path;

pub fn lint_notes(notes_dir: &Path, fix: bool) -> Result<(), AppError> {
    let repo = LocalMarkdownRepo::new(notes_dir);
    let slugs = repo.list_note_slugs()?;
    let mut total = 0;
    let mut fixed = 0;
    let mut failed = 0;

    let slug_width = slugs
        .iter()
        .map(|slug| slug.len() + ".md".len())
        .max()
        .unwrap_or_default();

    println!(
        "{}  {}\n",
        output::accent("Linting notes"),
        output::muted(format!("{} total", slugs.len()))
    );

    for slug in slugs {
        let result = lint_note_by_slug(&repo, &slug, fix);

        match result {
            Ok(issues) if issues.is_empty() => {
                print_result(&slug, slug_width, output::success("ok"));
            }
            Ok(issues) => {
                if fix {
                    let post = lint_note_by_slug(&repo, &slug, false)?;
                    if post.is_empty() {
                        print_result(&slug, slug_width, output::accent("fixed"));
                        fixed += 1;
                    } else {
                        print_failed(&post, &slug, slug_width);
                        failed += 1;
                    }
                } else {
                    print_failed(&issues, &slug, slug_width);
                    failed += 1;
                }
            }
            Err(err) => {
                print_result(&slug, slug_width, output::danger("failed"));
                println!("  {}", output::muted(format!("{err}")));
                failed += 1;
            }
        }

        total += 1;
    }

    println!();
    println!(
        "{} {} files, {} fixed, {} failed",
        output::accent("Done:"),
        output::strong(total),
        output::strong(fixed),
        if failed == 0 {
            output::strong(failed)
        } else {
            output::danger(failed)
        }
    );

    if failed > 0 {
        return Err(AppError::LintFailed(failed));
    }

    Ok(())
}

fn print_result(slug: &str, width: usize, status: impl std::fmt::Display) {
    println!(
        "  {}  {status}",
        output::strong(format!("{:<width$}", format!("{slug}.md")))
    );
}

fn print_failed(issues: &[LintIssue], slug: &str, width: usize) {
    let details = issues
        .iter()
        .map(|i| i.message.to_string())
        .collect::<Vec<_>>()
        .join("; ");

    let issue_slug = issues.first().map(|i| i.slug.as_str()).unwrap_or(slug);

    print_result(issue_slug, width, output::danger("failed"));
    println!("  {}", output::muted(details));
}
