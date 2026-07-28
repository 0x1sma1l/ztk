use std::io::{self, BufRead, IsTerminal, Write};
use std::path::Path;

use crate::errors::AppError;
use ztk::core::usecases::trash;
use ztk::storage::local_repo::LocalMarkdownRepo;

pub fn list(notes_dir: &Path) -> Result<(), AppError> {
    let collection = trash::list_trash(&LocalMarkdownRepo::new(notes_dir))?;
    if collection.entries.is_empty() {
        println!("Trash is empty.");
    } else {
        println!("Trash (id | original slug | deleted at):");
        for entry in collection.entries {
            println!(
                "{} | {} | {}",
                entry.id, entry.original_slug, entry.deleted_at
            );
        }
    }
    for issue in collection.issues {
        eprintln!(
            "warning: skipped trash metadata {}: {}",
            issue.slug, issue.message
        );
    }
    Ok(())
}

pub fn restore(notes_dir: &Path, id: &str) -> Result<(), AppError> {
    let note = trash::restore_trash(&LocalMarkdownRepo::new(notes_dir), id)?;
    println!("note restored: {}.md", note.slug);
    Ok(())
}

pub fn purge(notes_dir: &Path, id: Option<&str>, all: bool) -> Result<(), AppError> {
    let repo = LocalMarkdownRepo::new(notes_dir);
    let stdin = io::stdin();
    if !stdin.is_terminal() {
        return Err(AppError::PurgeConfirmationRequired);
    }

    let target = if all {
        "all trash".to_string()
    } else {
        format!(
            "trash entry `{}`",
            id.expect("clap requires a trash ID or --all")
        )
    };
    let mut reader = stdin.lock();
    let mut stdout = io::stdout().lock();
    if !confirm_purge(&mut reader, &mut stdout, &target)? {
        writeln!(stdout, "purge cancelled")?;
        return Ok(());
    }

    if all {
        let count = trash::purge_all_trash(&repo)?;
        writeln!(stdout, "trash permanently purged: {count} note(s)")?;
        return Ok(());
    }

    let id = id.expect("clap requires a trash ID or --all");
    trash::purge_trash(&repo, id)?;
    println!("trash entry permanently purged: {id}");
    Ok(())
}

fn confirm_purge<R: BufRead, W: Write>(
    reader: &mut R,
    writer: &mut W,
    target: &str,
) -> io::Result<bool> {
    loop {
        write!(
            writer,
            "Permanently purge {target}? This cannot be undone. [y/N]: "
        )?;
        writer.flush()?;

        let mut answer = String::new();
        if reader.read_line(&mut answer)? == 0 {
            return Ok(false);
        }

        match answer.trim().to_ascii_lowercase().as_str() {
            "y" | "yes" => return Ok(true),
            "" | "n" | "no" => return Ok(false),
            _ => writeln!(writer, "Please answer `y`/`yes` or `n`/`no`.")?,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::confirm_purge;

    #[test]
    fn purge_confirmation_accepts_yes() {
        for answer in ["y\n", "yes\n", "Y\n", "YES\n"] {
            let mut reader = Cursor::new(answer.as_bytes());
            let mut output = Vec::new();

            assert!(confirm_purge(&mut reader, &mut output, "all trash").unwrap());
        }
    }

    #[test]
    fn purge_confirmation_defaults_to_no() {
        for answer in ["\n", "n\n", "no\n", "N\n", "NO\n", ""] {
            let mut reader = Cursor::new(answer.as_bytes());
            let mut output = Vec::new();

            assert!(!confirm_purge(&mut reader, &mut output, "trash entry `id`").unwrap());
        }
    }

    #[test]
    fn purge_confirmation_reprompts() {
        let mut reader = Cursor::new(b"maybe\nyes\n");
        let mut output = Vec::new();

        assert!(confirm_purge(&mut reader, &mut output, "all trash").unwrap());
        assert!(
            String::from_utf8(output)
                .unwrap()
                .contains("Please answer `y`/`yes` or `n`/`no`.")
        );
    }
}
