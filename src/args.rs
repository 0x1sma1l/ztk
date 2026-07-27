use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "zet")]
#[command(
    about = "Local-first, terminal-first Markdown note manager",
    long_about = "Zet is a local-first, keyboard-driven note tool for creating, viewing, editing, linting, and managing Markdown notes from the terminal.",
    after_help = "Examples:\n  zet new \"Rust Ownership\" --tags rust,learning\n  zet list\n  zet view rust-ownership\n  zet edit rust-ownership\n  zet update rust-ownership --tags rust,learning\n  zet search ownership\n  zet lint\n  zet lint --fix\n  zet delete rust-ownership\n  zet stats\n  zet tui"
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
    /// Update note fields without opening an editor.
    Update {
        /// Note slug (without .md). Changing the title does not change the slug.
        slug: String,

        /// Replacement title.
        #[arg(long)]
        title: Option<String>,

        /// Replacement comma-separated tags; pass an empty value to clear tags.
        #[arg(long)]
        tags: Option<String>,

        /// Replacement Markdown body; pass an empty value to clear the body.
        #[arg(long)]
        body: Option<String>,
    },
    /// View a note as rendered Markdown.
    View {
        /// Note slug (without .md).
        slug: String,
    },
    /// Search note slugs, titles, and tags using fuzzy matching.
    Search {
        /// Search query.
        query: String,
    },
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

        /// Delete without interactive confirmation.
        #[arg(short, long)]
        force: bool,
    },

    /// Launch the full-screen terminal interface.
    Tui,
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use super::Cli;

    #[test]
    fn lint_help_describes_lint_only() {
        let command = Cli::command();
        let lint = command
            .find_subcommand("lint")
            .expect("lint command should be registered");

        assert_eq!(
            lint.get_about().map(ToString::to_string),
            Some("Lint notes for data and formatting issues".to_string())
        );
    }

    #[test]
    fn cli_v1_command_set_is_explicit() {
        let command = Cli::command();
        let names = command
            .get_subcommands()
            .map(|command| command.get_name())
            .collect::<Vec<_>>();

        assert_eq!(
            names,
            [
                "new", "list", "edit", "update", "view", "search", "lint", "stats", "delete", "tui"
            ]
        );
    }

    #[test]
    fn every_cli_v1_command_has_help_text() {
        let command = Cli::command();

        for subcommand in command
            .get_subcommands()
            .filter(|command| command.get_name() != "help")
        {
            assert!(
                subcommand.get_about().is_some(),
                "{} should have command help",
                subcommand.get_name()
            );
        }
    }
}
