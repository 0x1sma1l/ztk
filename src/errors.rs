use thiserror::Error;

use zet::core::errors::CoreError;

#[derive(Debug, Error)]
pub enum AppError {
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

    #[error("Lint failed: {0} file(s) contain issues")]
    LintFailed(usize),

    #[error(
        "Deletion requires confirmation from an interactive terminal. Re-run with `--force` to delete non-interactively."
    )]
    DeleteConfirmationRequired,
}
