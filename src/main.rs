mod args;
mod cli;
mod errors;
mod tui;

use clap::Parser;

use args::{Cli, Command};

use cli::delete::delete_note;
use cli::edit::edit_note;
use cli::lint::lint_notes;
use cli::list::list_notes;
use cli::new::create_note;
use cli::search::search_notes;
use cli::stats::get_stats;
use cli::tui::run_tui;
use cli::view::view_note;

fn main() {
    let cli = Cli::parse();

    let result = match &cli.command {
        Command::New { title, tags } => {
            let transformed_title = title.trim_matches(|c| c == '"' || c == '\'');
            create_note(transformed_title, tags.as_deref())
        }
        Command::List => list_notes(),
        Command::Edit { slug } => edit_note(slug),
        Command::View { slug } => view_note(slug),
        Command::Search { query } => search_notes(query),
        Command::Lint { fix } => lint_notes(*fix),
        Command::Stats => get_stats(),
        Command::Delete { slug, force } => delete_note(slug, *force),
        Command::Tui => {
            if let Err(err) = run_tui() {
                eprintln!("TUI error: {}", err);
                std::process::exit(1);
            }

            Ok(())
        }
    };

    if let Err(err) = result {
        eprintln!("{}", err);
        std::process::exit(1);
    }
}
