use color_eyre::Result;
use ratatui::DefaultTerminal;

use crate::core::note::Note;
use crate::core::usecases::list as list_usecase;
use crate::storage::local_repo::LocalMarkdownRepo;

use super::events;
use super::ui;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UiMode {
    #[default]
    Normal,
    Help,
}

/// The main application which holds the state and logic of the application.
#[derive(Debug, Default)]
pub struct App {
    /// Is the application running?
    running: bool,
    notes: Vec<Note>,
    selected_index: Option<usize>,
    show_help: bool,
    status_message: String,
    mode: UiMode,
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new() -> Self {
        let mut app = Self::default();
        app.refresh_notes();
        app
    }

    /// Run the application's main loop.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.running = true;
        while self.running {
            terminal.draw(|frame| ui::render(frame, &self))?;
            events::handle_crossterm_events(&mut self)?;
        }
        Ok(())
    }

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn notes(&self) -> &[Note] {
        &self.notes
    }

    pub fn set_notes(&mut self, notes: Vec<Note>) {
        self.notes = notes;
        self.selected_index = if self.notes.is_empty() { None } else { Some(0) };
    }

    pub fn selected_index(&self) -> Option<usize> {
        self.selected_index
    }

    pub fn selected_note(&self) -> Option<&Note> {
        self.selected_index.and_then(|index| self.notes.get(index))
    }

    pub fn select_next(&mut self) {
        if self.notes.is_empty() {
            self.selected_index = None;
            return;
        }

        self.selected_index = Some(match self.selected_index {
            Some(index) if index + 1 < self.notes.len() => index + 1,
            Some(_) => 0,
            None => 0,
        });
    }

    pub fn select_previous(&mut self) {
        if self.notes.is_empty() {
            self.selected_index = None;
            return;
        }

        self.selected_index = Some(match self.selected_index {
            Some(index) if index > 0 => index - 1,
            Some(_) => self.notes.len() - 1,
            None => 0,
        });
    }

    pub fn select_first(&mut self) {
        self.selected_index = if self.notes.is_empty() { None } else { Some(0) };
    }

    pub fn select_last(&mut self) {
        self.selected_index = if self.notes.is_empty() {
            None
        } else {
            Some(self.notes.len() - 1)
        };
    }

    pub fn show_help(&self) -> bool {
        self.show_help
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
        self.mode = if self.show_help {
            UiMode::Help
        } else {
            UiMode::Normal
        };
    }

    pub fn mode(&self) -> UiMode {
        self.mode
    }

    pub fn status_message(&self) -> &str {
        &self.status_message
    }

    pub fn set_status_message<T: Into<String>>(&mut self, status: T) {
        self.status_message = status.into();
    }

    pub fn refresh_notes(&mut self) {
        let repo = LocalMarkdownRepo::default();
        match list_usecase::list_notes(&repo) {
            Ok(collection) => {
                let note_count = collection.notes.len();
                let skipped = collection.issues.len();
                self.set_notes(collection.notes);

                if skipped > 0 {
                    self.set_status_message(format!(
                        "loaded {note_count} note(s), skipped {skipped} invalid file(s)"
                    ));
                } else if note_count == 0 {
                    self.set_status_message("no notes found (use `zet new <title>`)");
                } else {
                    self.set_status_message(format!("loaded {note_count} note(s)"));
                }
            }
            Err(error) => {
                self.set_notes(Vec::new());
                self.set_status_message(format!("failed to list notes: {error}"));
            }
        }
    }

    pub fn handle_resize(&mut self, cols: u16, rows: u16) {
        if self.status_message.is_empty() || self.status_message.starts_with("resized to ") {
            self.set_status_message(format!("resized to {cols}x{rows}"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{App, UiMode};
    use crate::core::note::Note;

    fn sample_note(title: &str) -> Note {
        Note {
            slug: title.to_lowercase().replace(' ', "-"),
            title: title.to_string(),
            date: "2026-04-07".to_string(),
            tags: vec!["test".to_string()],
            updated_at: "2026-04-07".to_string(),
            body: format!("Body for {title}"),
        }
    }

    #[test]
    fn set_notes_selects_first_note_by_default() {
        let mut app = App::default();
        app.set_notes(vec![sample_note("First"), sample_note("Second")]);

        assert_eq!(app.selected_index(), Some(0));
        assert_eq!(app.selected_note().map(|n| n.title.as_str()), Some("First"));
    }

    #[test]
    fn set_notes_empty_clears_selection() {
        let mut app = App::default();
        app.set_notes(vec![sample_note("First")]);
        app.set_notes(Vec::new());

        assert_eq!(app.selected_index(), None);
        assert!(app.selected_note().is_none());
    }

    #[test]
    fn selection_wraps_around_in_both_directions() {
        let mut app = App::default();
        app.set_notes(vec![
            sample_note("First"),
            sample_note("Second"),
            sample_note("Third"),
        ]);

        app.select_last();
        assert_eq!(app.selected_index(), Some(2));

        app.select_next();
        assert_eq!(app.selected_index(), Some(0));

        app.select_previous();
        assert_eq!(app.selected_index(), Some(2));
    }

    #[test]
    fn help_toggle_updates_mode_consistently() {
        let mut app = App::default();
        assert!(!app.show_help());
        assert_eq!(app.mode(), UiMode::Normal);

        app.toggle_help();
        assert!(app.show_help());
        assert_eq!(app.mode(), UiMode::Help);

        app.toggle_help();
        assert!(!app.show_help());
        assert_eq!(app.mode(), UiMode::Normal);
    }

    #[test]
    fn resize_updates_status_message() {
        let mut app = App::default();
        app.handle_resize(120, 40);
        assert_eq!(app.status_message(), "resized to 120x40");
    }

    #[test]
    fn resize_does_not_overwrite_meaningful_status_message() {
        let mut app = App::default();
        app.set_status_message("loaded 3 note(s), skipped 1 invalid file(s)");

        app.handle_resize(120, 40);

        assert_eq!(
            app.status_message(),
            "loaded 3 note(s), skipped 1 invalid file(s)"
        );
    }
}
