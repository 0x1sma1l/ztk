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

const FALLBACK_EDITORS: [&str; 4] = ["nvim", "vim", "vi", "nano"];

pub fn edit_note(notes_dir: &Path, slug: &str) -> Result<(), AppError> {
    let edit = EditBuffer::prepare(notes_dir, slug)?;
    let mut command_parts = configured_editor_command()?;
    let executable = command_parts.remove(0);
    let mut command = Command::new(&executable);
    command.args(command_parts);
    command.args(vim_number_arguments(OsStr::new(&executable)));
    let status = command
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
    select_editor_command_with(visual, editor, executable_on_path)
}

fn select_editor_command_with(
    visual: Option<&OsStr>,
    editor: Option<&OsStr>,
    mut is_available: impl FnMut(&str) -> bool,
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

    FALLBACK_EDITORS
        .into_iter()
        .find(|editor| is_available(editor))
        .map(|editor| ("detected editor", OsString::from(editor)))
        .ok_or(AppError::NoEditorFound)
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

pub(crate) fn is_vim_family(executable: &OsStr) -> bool {
    executable_name(executable).is_some_and(|name| matches!(name.as_str(), "vi" | "vim" | "nvim"))
}

pub(crate) fn is_neovim(executable: &OsStr) -> bool {
    executable_name(executable).is_some_and(|name| name == "nvim")
}

pub(crate) fn vim_number_arguments(executable: &OsStr) -> &'static [&'static str] {
    if executable_name(executable)
        .is_some_and(|name| matches!(name.as_str(), "vi" | "vim" | "nvim"))
    {
        &["-c", "set number relativenumber"]
    } else {
        &[]
    }
}

fn executable_name(executable: &OsStr) -> Option<String> {
    Path::new(executable)
        .file_stem()
        .and_then(OsStr::to_str)
        .map(str::to_ascii_lowercase)
}

fn executable_on_path(command: &str) -> bool {
    let Some(path) = env::var_os("PATH") else {
        return false;
    };
    let executable = format!("{command}{}", env::consts::EXE_SUFFIX);

    env::split_paths(&path).any(|directory| is_executable_file(&directory.join(&executable)))
}

#[cfg(unix)]
fn is_executable_file(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;

    std::fs::metadata(path)
        .is_ok_and(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
}

#[cfg(not(unix))]
fn is_executable_file(path: &Path) -> bool {
    path.is_file()
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;

    use crate::errors::AppError;

    use super::{
        is_neovim, is_vim_family, parse_editor_command, select_editor_command_with,
        vim_number_arguments,
    };

    #[test]
    fn visual_takes_precedence_over_editor() {
        let (source, command) = select_editor_command_with(
            Some(OsStr::new("code --wait")),
            Some(OsStr::new("vim")),
            |_| false,
        )
        .unwrap();

        assert_eq!(source, "$VISUAL");
        assert_eq!(command, "code --wait");
    }

    #[test]
    fn missing_editor_variables_use_the_first_available_terminal_editor() {
        let (source, command) =
            select_editor_command_with(None, None, |editor| matches!(editor, "vim" | "vi"))
                .unwrap();

        assert_eq!(source, "detected editor");
        assert_eq!(command, "vim");
    }

    #[test]
    fn missing_editor_variables_report_when_no_terminal_editor_is_available() {
        let error = select_editor_command_with(None, None, |_| false).unwrap_err();

        assert!(matches!(error, AppError::NoEditorFound));
    }

    #[test]
    fn empty_configured_editor_is_actionable_error() {
        let error = select_editor_command_with(None, Some(OsStr::new("")), |_| true).unwrap_err();

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

    #[test]
    fn vi_vim_and_neovim_enable_absolute_and_relative_line_numbers() {
        let expected = &["-c", "set number relativenumber"];

        assert_eq!(
            vim_number_arguments(OsStr::new("/usr/local/bin/nvim")),
            expected
        );
        assert_eq!(vim_number_arguments(OsStr::new("vim")), expected);
        assert_eq!(vim_number_arguments(OsStr::new("vi")), expected);
        assert!(vim_number_arguments(OsStr::new("code")).is_empty());
    }

    #[test]
    fn vim_family_detection_supports_paths_and_ignores_other_editors() {
        assert!(is_vim_family(OsStr::new("/usr/local/bin/nvim")));
        assert!(is_vim_family(OsStr::new("vim")));
        assert!(is_vim_family(OsStr::new("vi")));
        assert!(is_neovim(OsStr::new("nvim")));
        assert!(!is_neovim(OsStr::new("vim")));
        assert!(!is_vim_family(OsStr::new("code")));
    }
}
