use std::{env, ffi::OsString, fs, path::Path, process::Command};

use crate::errors::AppError;
use zet::core::repository::NoteRepository;
use zet::core::usecases::edit::{UpdateNoteRequest, update_note};
use zet::core::validators::validate_slug;
use zet::storage::frontmatter::parse_frontmatter_and_body;
use zet::storage::local_repo::LocalMarkdownRepo;

pub fn edit_note(notes_dir: &Path, slug: &str) -> Result<(), AppError> {
    let slug = validate_slug(slug)?;
    let repo = LocalMarkdownRepo::new(notes_dir);
    repo.ensure_note_exists(slug)?;
    let original = repo.read_raw_note(slug)?;
    let temporary = tempfile::Builder::new()
        .prefix(&format!("zet-{slug}-"))
        .suffix(".md")
        .tempfile()?;
    fs::write(temporary.path(), original)?;

    let visual = env::var_os("VISUAL");
    let editor = env::var_os("EDITOR");
    let (source, raw_command) = select_editor_command(visual.as_deref(), editor.as_deref())?;
    let mut command_parts = parse_editor_command(source, &raw_command)?;
    let executable = command_parts.remove(0);
    let status = Command::new(&executable)
        .args(command_parts)
        .arg(temporary.path())
        .status()
        .map_err(|source| AppError::EditorLaunch {
            editor: executable,
            source,
        })?;

    if !status.success() {
        return Err(AppError::EditorExitedWithError);
    }

    let edited = fs::read_to_string(temporary.path())?;
    let (frontmatter, body) = parse_frontmatter_and_body(&edited)?;
    update_note(
        &repo,
        slug,
        UpdateNoteRequest {
            title: Some(frontmatter.title),
            tags: Some(frontmatter.tags),
            body: Some(body),
        },
    )?;

    Ok(())
}

fn select_editor_command(
    visual: Option<&std::ffi::OsStr>,
    editor: Option<&std::ffi::OsStr>,
) -> Result<(&'static str, OsString), AppError> {
    if let Some(command) = visual {
        if command.is_empty() {
            return Err(AppError::EmptyEditorCommand("$VISUAL"));
        }
        return Ok(("$VISUAL", command.to_os_string()));
    }

    if let Some(command) = editor {
        if command.is_empty() {
            return Err(AppError::EmptyEditorCommand("$EDITOR"));
        }
        return Ok(("$EDITOR", command.to_os_string()));
    }

    Ok(("default editor", OsString::from("vi")))
}

fn parse_editor_command(
    source: &'static str,
    command: &std::ffi::OsStr,
) -> Result<Vec<String>, AppError> {
    let command = command
        .to_str()
        .ok_or(AppError::InvalidEditorCommand(source))?;
    let parts = shlex::split(command).ok_or(AppError::InvalidEditorCommand(source))?;

    if parts.is_empty() {
        return Err(AppError::EmptyEditorCommand(source));
    }

    Ok(parts)
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;

    use crate::errors::AppError;

    use super::{parse_editor_command, select_editor_command};

    #[test]
    fn visual_takes_precedence_over_editor() {
        let (source, command) =
            select_editor_command(Some(OsStr::new("code --wait")), Some(OsStr::new("vim")))
                .unwrap();

        assert_eq!(source, "$VISUAL");
        assert_eq!(command, "code --wait");
    }

    #[test]
    fn missing_editor_variables_fall_back_to_vi() {
        let (source, command) = select_editor_command(None, None).unwrap();

        assert_eq!(source, "default editor");
        assert_eq!(command, "vi");
    }

    #[test]
    fn empty_configured_editor_is_actionable_error() {
        let error = select_editor_command(None, Some(OsStr::new(""))).unwrap_err();

        assert!(matches!(error, AppError::EmptyEditorCommand("$EDITOR")));
    }

    #[test]
    fn parser_preserves_quoted_executable_and_arguments() {
        let parts = parse_editor_command(
            "$EDITOR",
            OsStr::new("'/Applications/Editor App/bin/editor' --wait --reuse-window"),
        )
        .unwrap();

        assert_eq!(
            parts,
            vec![
                "/Applications/Editor App/bin/editor",
                "--wait",
                "--reuse-window"
            ]
        );
    }

    #[test]
    fn parser_rejects_unclosed_quotes() {
        let error = parse_editor_command("$EDITOR", OsStr::new("editor 'unclosed")).unwrap_err();

        assert!(matches!(error, AppError::InvalidEditorCommand("$EDITOR")));
    }
}
