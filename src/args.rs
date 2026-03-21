use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "zet")]
#[command(about = "A simple note-taking CLI tool in Rust", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    New {
        /// The title of the note
        title: String,

        /// Comma separated tags (e.g rust, zettelkasten)
        #[arg(short, long)]
        tags: Option<String>,
    },
    /// List all notes
    List,
    /// Edit an existing note in your $EDITOR
    Edit {
        /// The slug of the note (without .md)
        slug: String,
    },
    /// View the selected notes as Markdown
    View {
        /// the slug of the note (without .md)
        slug: String,
    },
    /// Launch interactive fuzzy search UI
    // Search,
    /// Check for Errors in notes
    Lint {
        /// Fix issues automatically
        #[arg(long)]
        fix: bool,
    },
    /// Get Note count
    Stats,
    /// Delete a note
    Delete {
        /// The slug of the note (without .md)
        slug: String,
    },

    /// Launch the full-screen interactive interface
    Tui,
}
