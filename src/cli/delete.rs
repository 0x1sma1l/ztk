use std::io::{self, BufRead, IsTerminal, Write};
use std::path::Path;

use crate::errors::AppError;
use zet::core::repository::NoteRepository;
use zet::core::usecases::delete::delete_note as delete_note_usecase;
use zet::storage::local_repo::LocalMarkdownRepo;

pub fn delete_note(notes_dir: &Path, slug: &str, force: bool) -> Result<(), AppError> {
    let repo = LocalMarkdownRepo::new(notes_dir);
    repo.ensure_note_exists(slug)?;

    if !force {
        let stdin = io::stdin();
        if !stdin.is_terminal() {
            return Err(AppError::DeleteConfirmationRequired);
        }

        let mut reader = stdin.lock();
        let mut stdout = io::stdout().lock();
        if !confirm_delete(&mut reader, &mut stdout, slug)? {
            writeln!(stdout, "delete cancelled: notes/{slug}.md")?;
            return Ok(());
        }
    }

    delete_note_usecase(&repo, slug)?;

    println!("note deleted: notes/{}.md", slug);

    Ok(())
}

fn confirm_delete<R: BufRead, W: Write>(
    reader: &mut R,
    writer: &mut W,
    slug: &str,
) -> io::Result<bool> {
    loop {
        write!(writer, "Permanently delete notes/{slug}.md? [y/N]: ")?;
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

    use super::confirm_delete;

    #[test]
    fn confirmation_accepts_yes_case_insensitively() {
        for answer in ["y\n", "yes\n", "Y\n", "YES\n"] {
            let mut reader = Cursor::new(answer.as_bytes());
            let mut output = Vec::new();

            assert!(confirm_delete(&mut reader, &mut output, "example").unwrap());
        }
    }

    #[test]
    fn confirmation_defaults_to_no() {
        for answer in ["\n", "n\n", "no\n", "N\n", "NO\n", ""] {
            let mut reader = Cursor::new(answer.as_bytes());
            let mut output = Vec::new();

            assert!(!confirm_delete(&mut reader, &mut output, "example").unwrap());
        }
    }

    #[test]
    fn confirmation_reprompts_after_invalid_input() {
        let mut reader = Cursor::new(b"maybe\nyes\n");
        let mut output = Vec::new();

        assert!(confirm_delete(&mut reader, &mut output, "example").unwrap());
        assert!(
            String::from_utf8(output)
                .unwrap()
                .contains("Please answer `y`/`yes` or `n`/`no`.")
        );
    }
}
