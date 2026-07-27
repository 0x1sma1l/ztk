use crate::core::repository::NoteRepository;
use crate::core::usecases::lint::{LintIssue, lint_note_by_slug};
use crate::errors::AppError;
use crate::storage::local_repo::LocalMarkdownRepo;

use colored::Colorize;
use std::thread;
use std::time::Duration;

const DELAY_IN_MILLISECONDS: u64 = 2;

pub fn lint_notes(fix: bool) -> Result<(), AppError> {
    let repo = LocalMarkdownRepo::default();
    let slugs = repo.list_note_slugs()?;
    let mut total = 0;
    let mut fixed = 0;
    let mut failed = 0;

    println!("linting notes...");

    for slug in slugs {
        let result = lint_note_by_slug(&repo, &slug, fix);

        match result {
            Ok(issues) if issues.is_empty() => {
                println!("{slug}.md ... {}", "ok".green());
            }
            Ok(issues) => {
                if fix {
                    let post = lint_note_by_slug(&repo, &slug, false)?;
                    if post.is_empty() {
                        println!("{slug}.md ... {}", "fixed".yellow());
                        fixed += 1;
                    } else {
                        print_failed(&post, &slug);
                        failed += 1;
                    }
                } else {
                    print_failed(&issues, &slug);
                    failed += 1;
                }
            }
            Err(err) => {
                println!("{slug}.md ... {} (Error: {err})", "failed".red());
                failed += 1;
            }
        }

        total += 1;
        thread::sleep(Duration::from_millis(DELAY_IN_MILLISECONDS));
    }

    println!();
    println!("Done: {} files, {} fixed, {} failed", total, fixed, failed);

    if failed > 0 {
        return Err(AppError::LintFailed(failed));
    }

    Ok(())
}

fn print_failed(issues: &[LintIssue], slug: &str) {
    let details = issues
        .iter()
        .map(|i| i.message.to_string())
        .collect::<Vec<_>>()
        .join("; ");

    let issue_slug = issues.first().map(|i| i.slug.as_str()).unwrap_or(slug);

    println!(
        "{}.md ... {} (Error: {details})",
        issue_slug,
        "failed".red()
    );
}
