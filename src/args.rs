use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "zet")]
#[command(
    about = "Local-first, terminal-first Markdown note manager",
    long_about = "Zet is a local-first, keyboard-driven note tool for creating, viewing, editing, linting, and managing Markdown notes from the terminal.",
    after_help = "Examples:\n  zet new \"Rust Ownership\" --tags rust,learning\n  zet list\n  zet view rust-ownership\n  zet edit rust-ownership\n  zet lint\n  zet lint --fix\n  zet delete rust-ownership\n  zet stats\n  zet tui"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Create a new note.
    New {
        /// Note title.
        title: String,

        /// Comma-separated tags (example: rust, zettelkasten).
        #[arg(short, long)]
        tags: Option<String>,
    },
    /// List all notes.
    List,
    /// Edit a note in your $EDITOR.
    Edit {
        /// Note slug (without .md).
        slug: String,
    },
    /// View a note as rendered Markdown.
    View {
        /// Note slug (without .md).
        slug: String,
    },
    /// Launch interactive fuzzy search UI
    // Search,
    /// Lint notes for data and formatting issues.
    Lint {
        /// Apply available automatic fixes.
        #[arg(long)]
        fix: bool,
    },
    /// Show note count summary.
    Stats,
    /// Delete a note.
    Delete {
        /// Note slug (without .md).
        slug: String,
    },

    /// Launch the full-screen terminal interface.
    Tui,
}
