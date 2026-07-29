mod args;
mod cli;
mod config;
mod errors;
mod fuzzy;
mod tui;

use clap::Parser;

use args::{Cli, Command, TrashAction};

use cli::delete::delete_note;
use cli::edit::edit_note;
use cli::lint::lint_notes;
use cli::list::list_notes;
use cli::new::create_note;
use cli::search::search_notes;
use cli::stats::get_stats;
use cli::trash;
use cli::tui::run_tui;
use cli::update::update_note;
use cli::view::view_note;

fn main() {
    let cli = Cli::parse();
    let notes_dir = match config::resolve_notes_dir(cli.notes_dir.clone()) {
        Ok(path) => path,
        Err(error) => {
            cli::output::error(error);
            std::process::exit(1);
        }
    };
    if let Err(error) = config::ensure_notes_dir(&notes_dir) {
        cli::output::error(error);
        std::process::exit(1);
    }

    let result = match &cli.command {
        Command::New { title, tags } => {
            let transformed_title = title.trim_matches(|c| c == '"' || c == '\'');
            create_note(&notes_dir, transformed_title, tags.as_deref())
                .and_then(|slug| edit_note(&notes_dir, &slug))
        }
        Command::List => list_notes(&notes_dir),
        Command::Edit { slug } => edit_note(&notes_dir, slug),
        Command::Update {
            slug,
            title,
            tags,
            body,
        } => update_note(
            &notes_dir,
            slug,
            title.as_deref(),
            tags.as_deref(),
            body.as_deref(),
        ),
        Command::View { slug } => view_note(&notes_dir, slug),
        Command::Search { query } => search_notes(&notes_dir, query.as_deref()),
        Command::Lint { fix } => lint_notes(&notes_dir, *fix),
        Command::Stats => get_stats(&notes_dir),
        Command::Delete { slug, force } => delete_note(&notes_dir, slug, *force),
        Command::Purge { id, all } => trash::purge(&notes_dir, id.as_deref(), *all),
        Command::Trash { action } => match action {
            TrashAction::List => trash::list(&notes_dir),
            TrashAction::Restore { id } => trash::restore(&notes_dir, id),
        },
        Command::Tui => {
            if let Err(err) = run_tui(&notes_dir) {
                cli::output::error(format!("TUI: {err}"));
                std::process::exit(1);
            }

            Ok(())
        }
    };

    if let Err(err) = result {
        cli::output::error(err);
        std::process::exit(1);
    }
}
