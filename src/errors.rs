use thiserror::Error;

use crate::config::ConfigError;
use ztk::core::errors::CoreError;

#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    Config(#[from] ConfigError),
    #[error(transparent)]
    Core(#[from] CoreError),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Editor exited with a non-zero status. Check your editor configuration and try again.")]
    EditorExitedWithError,

    #[error("{0} is set but empty. Set it to an editor command or remove the variable.")]
    EmptyEditorCommand(&'static str),

    #[error("Could not parse {0}. Check its quotes and command syntax.")]
    InvalidEditorCommand(&'static str),

    #[error("Failed to launch editor `{editor}`: {source}")]
    EditorLaunch {
        editor: String,
        source: std::io::Error,
    },

    #[error("Embedded editor error: {0}")]
    EmbeddedEditor(String),

    #[error(
        "Interactive search requires `fzf`. Install it with `brew install fzf`, `sudo apt install fzf`, or `winget install junegunn.fzf`."
    )]
    FzfNotInstalled,

    #[error("fzf exited unsuccessfully (status: {0:?})")]
    FzfFailed(Option<i32>),

    #[error("Lint failed: {0} file(s) contain issues")]
    LintFailed(usize),

    #[error(
        "Deletion requires confirmation from an interactive terminal. Re-run with `--force` to delete non-interactively."
    )]
    DeleteConfirmationRequired,

    #[error("Permanent purge requires confirmation from an interactive terminal.")]
    PurgeConfirmationRequired,
}
