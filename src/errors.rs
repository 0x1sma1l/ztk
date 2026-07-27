use thiserror::Error;

use zet::core::errors::CoreError;

#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    Core(#[from] CoreError),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error(
        "Editor exited with a non-zero status. Check your $EDITOR configuration and try again."
    )]
    EditorExitedWithError,

    #[error("Lint failed: {0} file(s) contain issues")]
    LintFailed(usize),

    #[error(
        "Deletion requires confirmation from an interactive terminal. Re-run with `--force` to delete non-interactively."
    )]
    DeleteConfirmationRequired,
}
