use color_eyre::Result;
use ratatui::DefaultTerminal;
use std::path::PathBuf;

use zet::core::note::Note;
use zet::core::repository::NoteRepository;
use zet::core::usecases::list as list_usecase;
use zet::core::usecases::{create, delete, edit, search};
use zet::storage::local_repo::LocalMarkdownRepo;

use super::events;
use super::ui;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UiMode {
    #[default]
    Normal,
    Help,
    Search,
    CreateTitle,
    EditTitle,
    EditTags,
    EditBody,
    ConfirmDelete,
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
    preview_scroll: u16,
    preview_max_scroll: u16,
    preview_page_size: u16,
    input: String,
    notes_dir: PathBuf,
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new() -> Self {
        let mut app = Self::default();
        app.refresh_notes();
        app
    }

    #[cfg(test)]
    pub fn with_notes_dir(path: impl AsRef<std::path::Path>) -> Self {
        Self {
            notes_dir: path.as_ref().to_path_buf(),
            ..Self::default()
        }
    }

    fn repository(&self) -> LocalMarkdownRepo {
        if self.notes_dir.as_os_str().is_empty() {
            LocalMarkdownRepo::default()
        } else {
            LocalMarkdownRepo::new(&self.notes_dir)
        }
    }

    /// Run the application's main loop.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.running = true;
        while self.running {
            let size = terminal.size()?;
            let area = ratatui::layout::Rect::new(0, 0, size.width, size.height);
            let (max_scroll, page_size) = ui::preview_metrics(&self, area);
            self.update_preview_metrics(max_scroll, page_size);
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
        self.reset_preview_scroll();
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
        self.reset_preview_scroll();
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
        self.reset_preview_scroll();
    }

    pub fn select_first(&mut self) {
        self.selected_index = if self.notes.is_empty() { None } else { Some(0) };
        self.reset_preview_scroll();
    }

    pub fn select_last(&mut self) {
        self.selected_index = if self.notes.is_empty() {
            None
        } else {
            Some(self.notes.len() - 1)
        };
        self.reset_preview_scroll();
    }

    pub fn preview_scroll(&self) -> u16 {
        self.preview_scroll
    }

    pub fn preview_max_scroll(&self) -> u16 {
        self.preview_max_scroll
    }

    pub fn update_preview_metrics(&mut self, max_scroll: u16, page_size: u16) {
        self.preview_max_scroll = max_scroll;
        self.preview_page_size = page_size.max(1);
        self.preview_scroll = self.preview_scroll.min(max_scroll);
    }

    pub fn scroll_preview_down(&mut self, amount: u16) {
        self.preview_scroll = self
            .preview_scroll
            .saturating_add(amount)
            .min(self.preview_max_scroll);
    }

    pub fn scroll_preview_up(&mut self, amount: u16) {
        self.preview_scroll = self.preview_scroll.saturating_sub(amount);
    }

    pub fn scroll_preview_page_down(&mut self) {
        self.scroll_preview_down(self.preview_page_size);
    }

    pub fn scroll_preview_page_up(&mut self) {
        self.scroll_preview_up(self.preview_page_size);
    }

    fn reset_preview_scroll(&mut self) {
        self.preview_scroll = 0;
        self.preview_max_scroll = 0;
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

    pub fn input(&self) -> &str {
        &self.input
    }

    pub fn begin_input(&mut self, mode: UiMode) {
        self.input = match mode {
            UiMode::EditTitle => self
                .selected_note()
                .map(|note| note.title.clone())
                .unwrap_or_default(),
            UiMode::EditTags => self
                .selected_note()
                .map(|note| note.tags.join(","))
                .unwrap_or_default(),
            UiMode::EditBody => self
                .selected_note()
                .map(|note| note.body.clone())
                .unwrap_or_default(),
            _ => String::new(),
        };
        self.mode = mode;
    }

    pub fn begin_delete(&mut self) {
        if self.selected_note().is_some() {
            self.mode = UiMode::ConfirmDelete;
        } else {
            self.set_status_message("no note selected");
        }
    }

    pub fn push_input(&mut self, character: char) {
        self.input.push(character);
    }

    pub fn pop_input(&mut self) {
        self.input.pop();
    }

    pub fn clear_input(&mut self) {
        self.input.clear();
    }

    pub fn cancel_mode(&mut self) {
        self.input.clear();
        self.mode = UiMode::Normal;
    }

    pub fn submit_input(&mut self) {
        let result = match self.mode {
            UiMode::Search => self.submit_search(),
            UiMode::CreateTitle => self.submit_create(),
            UiMode::EditTitle => self.submit_update(edit::UpdateNoteRequest {
                title: Some(self.input.clone()),
                ..Default::default()
            }),
            UiMode::EditTags => self.submit_update(edit::UpdateNoteRequest {
                tags: Some(self.input.split(',').map(ToOwned::to_owned).collect()),
                ..Default::default()
            }),
            UiMode::EditBody => self.submit_update(edit::UpdateNoteRequest {
                body: Some(self.input.clone()),
                ..Default::default()
            }),
            _ => return,
        };

        if let Err(error) = result {
            self.set_status_message(format!("error: {error}"));
        }
    }

    pub fn confirm_delete(&mut self) {
        let Some(slug) = self.selected_note().map(|note| note.slug.clone()) else {
            self.cancel_mode();
            self.set_status_message("no note selected");
            return;
        };
        let repo = self.repository();
        match delete::delete_note(&repo, &slug) {
            Ok(()) => {
                self.cancel_mode();
                self.refresh_notes();
                self.set_status_message(format!("deleted {slug}"));
            }
            Err(error) => self.set_status_message(format!("error: {error}")),
        }
    }

    fn submit_create(&mut self) -> Result<(), zet::core::errors::CoreError> {
        let repo = self.repository();
        let note = create::create_note(&repo, &self.input, None)?;
        let slug = note.slug.clone();
        self.cancel_mode();
        self.refresh_notes_selecting(Some(&slug));
        self.set_status_message(format!("created {slug}"));
        Ok(())
    }

    fn submit_update(
        &mut self,
        request: edit::UpdateNoteRequest,
    ) -> Result<(), zet::core::errors::CoreError> {
        let slug = self
            .selected_note()
            .map(|note| note.slug.clone())
            .ok_or_else(|| zet::core::errors::CoreError::NoteNotFound(String::new()))?;
        let repo = self.repository();
        let result = edit::update_note(&repo, &slug, request)?;
        self.cancel_mode();
        self.refresh_notes_selecting(Some(&slug));
        self.set_status_message(if result.changed {
            format!("updated {slug}")
        } else {
            format!("unchanged {slug}")
        });
        Ok(())
    }

    fn submit_search(&mut self) -> Result<(), zet::core::errors::CoreError> {
        let repo = self.repository();
        let results = search::search_notes(&repo, &self.input)?;
        let notes = results
            .matches
            .iter()
            .map(|result| repo.read_note(&result.slug))
            .collect::<Result<Vec<_>, _>>()?;
        let count = notes.len();
        let skipped = results.issues.len();
        self.set_notes(notes);
        self.cancel_mode();
        self.set_status_message(format!("{count} search result(s), {skipped} skipped"));
        Ok(())
    }

    pub fn status_message(&self) -> &str {
        &self.status_message
    }

    pub fn set_status_message<T: Into<String>>(&mut self, status: T) {
        self.status_message = status.into();
    }

    pub fn refresh_notes(&mut self) {
        self.refresh_notes_selecting(None);
    }

    fn refresh_notes_selecting(&mut self, preferred_slug: Option<&str>) {
        let repo = self.repository();
        match list_usecase::list_notes(&repo) {
            Ok(collection) => {
                let note_count = collection.notes.len();
                let skipped = collection.issues.len();
                self.set_notes(collection.notes);
                if let Some(index) = preferred_slug
                    .and_then(|slug| self.notes.iter().position(|note| note.slug == slug))
                {
                    self.selected_index = Some(index);
                }

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
    use tempfile::TempDir;
    use zet::core::note::Note;

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

    #[test]
    fn preview_scroll_is_bounded_and_page_aware() {
        let mut app = App::default();
        app.update_preview_metrics(12, 5);

        app.scroll_preview_down(3);
        assert_eq!(app.preview_scroll(), 3);
        app.scroll_preview_page_down();
        assert_eq!(app.preview_scroll(), 8);
        app.scroll_preview_down(u16::MAX);
        assert_eq!(app.preview_scroll(), 12);
        app.scroll_preview_page_up();
        assert_eq!(app.preview_scroll(), 7);
        app.scroll_preview_up(u16::MAX);
        assert_eq!(app.preview_scroll(), 0);
    }

    #[test]
    fn selection_change_resets_preview_scroll() {
        let mut app = App::default();
        app.set_notes(vec![sample_note("First"), sample_note("Second")]);
        app.update_preview_metrics(20, 5);
        app.scroll_preview_down(8);

        app.select_next();

        assert_eq!(app.preview_scroll(), 0);
        assert_eq!(app.preview_max_scroll(), 0);
    }

    #[test]
    fn resized_preview_clamps_existing_scroll() {
        let mut app = App::default();
        app.update_preview_metrics(20, 5);
        app.scroll_preview_down(18);

        app.update_preview_metrics(4, 10);

        assert_eq!(app.preview_scroll(), 4);
    }

    #[test]
    fn tui_actions_create_search_update_and_delete_through_core_use_cases() {
        let root = TempDir::new().unwrap();
        let mut app = App::with_notes_dir(root.path().join("notes"));

        app.begin_input(UiMode::CreateTitle);
        app.input = "Rust Ownership".to_string();
        app.submit_input();
        assert_eq!(app.mode(), UiMode::Normal);
        assert_eq!(
            app.selected_note().map(|note| note.slug.as_str()),
            Some("rust-ownership")
        );
        assert!(root.path().join("notes/rust-ownership.md").exists());

        app.begin_input(UiMode::EditTags);
        app.input = "rust,learning".to_string();
        app.submit_input();
        assert_eq!(app.selected_note().unwrap().tags, ["rust", "learning"]);

        app.begin_input(UiMode::EditTitle);
        app.clear_input();
        app.input = "Ownership Rules".to_string();
        app.submit_input();
        assert_eq!(app.selected_note().unwrap().title, "Ownership Rules");
        assert_eq!(app.selected_note().unwrap().slug, "rust-ownership");

        app.begin_input(UiMode::EditBody);
        app.clear_input();
        app.input = "Updated from the TUI".to_string();
        app.submit_input();
        assert_eq!(
            app.selected_note().unwrap().body.trim_start_matches('\n'),
            "Updated from the TUI"
        );

        app.begin_input(UiMode::Search);
        app.input = "learning".to_string();
        app.submit_input();
        assert_eq!(app.notes().len(), 1);
        assert!(app.status_message().contains("1 search result"));

        app.begin_delete();
        app.confirm_delete();
        assert!(app.notes().is_empty());
        assert!(!root.path().join("notes/rust-ownership.md").exists());
    }

    #[test]
    fn validation_error_preserves_mode_and_typed_input() {
        let root = TempDir::new().unwrap();
        let mut app = App::with_notes_dir(root.path().join("notes"));
        app.begin_input(UiMode::CreateTitle);
        app.input = "   ".to_string();

        app.submit_input();

        assert_eq!(app.mode(), UiMode::CreateTitle);
        assert_eq!(app.input(), "   ");
        assert!(app.status_message().contains("Title cannot be empty"));
        assert!(app.notes().is_empty());
    }
}
