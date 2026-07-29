use std::{
    env,
    ffi::{OsStr, OsString},
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use crate::errors::AppError;
use ztk::core::repository::NoteRepository;
use ztk::core::usecases::edit::{UpdateNoteRequest, update_note};
use ztk::core::validators::validate_slug;
use ztk::storage::frontmatter::parse_frontmatter_and_body;
use ztk::storage::local_repo::LocalMarkdownRepo;

pub fn edit_note(notes_dir: &Path, slug: &str) -> Result<(), AppError> {
    let edit = EditBuffer::prepare(notes_dir, slug)?;
    let mut command_parts = configured_editor_command()?;
    let executable = command_parts.remove(0);
    let status = Command::new(&executable)
        .args(command_parts)
        .arg(edit.path())
        .status()
        .map_err(|source| AppError::EditorLaunch {
            editor: executable,
            source,
        })?;

    if !status.success() {
        return Err(AppError::EditorExitedWithError);
    }

    edit.commit()?;
    Ok(())
}

pub(crate) struct EditBuffer {
    notes_dir: PathBuf,
    slug: String,
    temporary: tempfile::NamedTempFile,
}

impl EditBuffer {
    pub(crate) fn prepare(notes_dir: &Path, slug: &str) -> Result<Self, AppError> {
        let slug = validate_slug(slug)?;
        let repo = LocalMarkdownRepo::new(notes_dir);
        repo.ensure_note_exists(slug)?;
        let original = repo.read_raw_note(slug)?;
        let temporary = tempfile::Builder::new()
            .prefix(&format!("ztk-{slug}-"))
            .suffix(".md")
            .tempfile()?;
        fs::write(temporary.path(), original)?;

        Ok(Self {
            notes_dir: notes_dir.to_path_buf(),
            slug: slug.to_string(),
            temporary,
        })
    }

    pub(crate) fn path(&self) -> &Path {
        self.temporary.path()
    }

    pub(crate) fn slug(&self) -> &str {
        &self.slug
    }

    pub(crate) fn commit(self) -> Result<(), AppError> {
        let edited = fs::read_to_string(self.temporary.path())?;
        let (frontmatter, body) = parse_frontmatter_and_body(&edited)?;
        update_note(
            &LocalMarkdownRepo::new(&self.notes_dir),
            &self.slug,
            UpdateNoteRequest {
                title: Some(frontmatter.title),
                tags: Some(frontmatter.tags),
                body: Some(body),
            },
        )?;
        Ok(())
    }
}

pub(crate) fn configured_editor_command() -> Result<Vec<String>, AppError> {
    let visual = env::var_os("VISUAL");
    let editor = env::var_os("EDITOR");
    let (source, raw_command) = select_editor_command(visual.as_deref(), editor.as_deref())?;
    parse_editor_command(source, &raw_command)
}

fn select_editor_command(
    visual: Option<&OsStr>,
    editor: Option<&OsStr>,
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

fn parse_editor_command(source: &'static str, command: &OsStr) -> Result<Vec<String>, AppError> {
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
