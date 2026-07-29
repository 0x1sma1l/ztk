use color_eyre::Result;
use ratatui::DefaultTerminal;
use std::path::PathBuf;
use std::time::Duration;

use ztk::core::note::Note;
use ztk::core::usecases::list as list_usecase;
use ztk::core::usecases::{create, delete};
use ztk::storage::local_repo::LocalMarkdownRepo;

use super::editor::EditorSession;
use super::events;
use super::ui;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UiMode {
    #[default]
    Browse,
    Read,
    Editor,
    Help,
    CreateTitle,
    Search,
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
    search_matches: Vec<String>,
    search_selected_index: Option<usize>,
    editor: Option<EditorSession>,
    editor_cols: u16,
    editor_rows: u16,
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new(notes_dir: impl AsRef<std::path::Path>) -> Self {
        let mut app = Self {
            notes_dir: notes_dir.as_ref().to_path_buf(),
            ..Self::default()
        };
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
        let mut redraw = true;
        while self.running {
            let size = terminal.size()?;
            let area = ratatui::layout::Rect::new(0, 0, size.width, size.height);
            let (max_scroll, page_size) = ui::preview_metrics(&self, area);
            self.update_preview_metrics(max_scroll, page_size);
            let (editor_cols, editor_rows) = ui::note_surface_size(area);
            self.update_editor_size(editor_cols, editor_rows);
            redraw |= self.poll_editor();
            redraw |= self.editor.as_ref().is_some_and(EditorSession::take_dirty);
            if redraw {
                terminal.draw(|frame| ui::render(frame, &self))?;
            }
            redraw = events::handle_crossterm_events(&mut self, Duration::from_millis(16))?;
        }
        Ok(())
    }

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        if self.editor.is_some() {
            self.mode = UiMode::Editor;
            self.set_status_message("close the editor with :q before quitting Ztk (F6 detaches)");
            return;
        }
        self.running = false;
    }

    pub fn notes(&self) -> &[Note] {
        &self.notes
    }

    pub fn notes_dir(&self) -> &std::path::Path {
        &self.notes_dir
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

    #[cfg(test)]
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
            UiMode::Browse
        };
    }

    pub fn mode(&self) -> UiMode {
        self.mode
    }

    pub fn input(&self) -> &str {
        &self.input
    }

    pub fn begin_create(&mut self) {
        self.input.clear();
        self.mode = UiMode::CreateTitle;
    }

    pub fn begin_delete(&mut self) {
        if self.selected_note().is_some() {
            self.mode = UiMode::ConfirmDelete;
        } else {
            self.set_status_message("no note selected");
        }
    }

    pub fn begin_read(&mut self) {
        if self.selected_note().is_some() {
            self.mode = UiMode::Read;
            self.reset_preview_scroll();
        }
    }

    pub fn return_to_browse(&mut self) {
        self.mode = UiMode::Browse;
        self.reset_preview_scroll();
    }

    pub fn begin_editor(&mut self) {
        if self.editor.is_some() {
            self.mode = UiMode::Editor;
            self.set_status_message("editor attached; F6 returns to read mode");
            return;
        }

        let Some(slug) = self.selected_note().map(|note| note.slug.clone()) else {
            self.set_status_message("no note selected");
            return;
        };
        let command = match crate::cli::edit::configured_editor_command() {
            Ok(command) => command,
            Err(error) => {
                self.set_status_message(format!("editor failed: {error}"));
                return;
            }
        };
        match EditorSession::start(
            &self.notes_dir,
            &slug,
            command,
            self.editor_cols.max(1),
            self.editor_rows.max(1),
        ) {
            Ok(editor) => {
                self.editor = Some(editor);
                self.mode = UiMode::Editor;
                self.set_status_message("editor attached; :wq saves and returns, F6 detaches");
            }
            Err(error) => self.set_status_message(format!("editor failed: {error}")),
        }
    }

    pub fn detach_editor(&mut self) {
        if self.editor.is_some() {
            self.mode = UiMode::Read;
            self.set_status_message("editor detached and still running; press e to reattach");
        }
    }

    pub fn editor(&self) -> Option<&EditorSession> {
        self.editor.as_ref()
    }

    pub fn send_editor_key(&mut self, key: crossterm::event::KeyEvent) {
        if let Some(editor) = self.editor.as_mut()
            && let Err(error) = editor.send_key(key)
        {
            self.set_status_message(format!("editor input failed: {error}"));
        }
    }

    pub fn send_editor_paste(&mut self, text: &str) {
        if let Some(editor) = self.editor.as_mut()
            && let Err(error) = editor.send_paste(text)
        {
            self.set_status_message(format!("editor paste failed: {error}"));
        }
    }

    fn update_editor_size(&mut self, cols: u16, rows: u16) {
        self.editor_cols = cols;
        self.editor_rows = rows;
        if let Some(editor) = self.editor.as_mut()
            && let Err(error) = editor.resize(cols, rows)
        {
            self.set_status_message(format!("editor resize failed: {error}"));
        }
    }

    fn poll_editor(&mut self) -> bool {
        let outcome = self
            .editor
            .as_mut()
            .map(EditorSession::has_exited)
            .transpose();
        match outcome {
            Ok(Some(Some(success))) => {
                let editor = self.editor.take().expect("polled editor exists");
                self.mode = UiMode::Read;
                if success {
                    match editor.commit() {
                        Ok(slug) => {
                            self.refresh_notes_selecting(Some(&slug));
                            self.mode = UiMode::Read;
                            self.set_status_message(format!("saved {slug}"));
                        }
                        Err(error) => self
                            .set_status_message(format!("could not save editor changes: {error}")),
                    }
                } else {
                    self.set_status_message("editor exited unsuccessfully; changes were not saved");
                }
                true
            }
            Ok(Some(None) | None) => false,
            Err(error) => {
                self.set_status_message(format!("editor failed: {error}"));
                true
            }
        }
    }

    pub fn begin_search(&mut self) {
        self.input.clear();
        self.mode = UiMode::Search;
        self.update_search_matches();
    }

    pub fn search_matches(&self) -> &[String] {
        &self.search_matches
    }

    pub fn search_selected_index(&self) -> Option<usize> {
        self.search_selected_index
    }

    pub fn search_selected_note(&self) -> Option<&Note> {
        let slug = self
            .search_selected_index
            .and_then(|index| self.search_matches.get(index))?;
        self.notes.iter().find(|note| note.slug == *slug)
    }

    pub fn select_next_search_match(&mut self) {
        if self.search_matches.is_empty() {
            self.search_selected_index = None;
        } else {
            self.search_selected_index = Some(match self.search_selected_index {
                Some(index) if index + 1 < self.search_matches.len() => index + 1,
                _ => 0,
            });
        }
    }

    pub fn select_previous_search_match(&mut self) {
        if self.search_matches.is_empty() {
            self.search_selected_index = None;
        } else {
            self.search_selected_index = Some(match self.search_selected_index {
                Some(index) if index > 0 => index - 1,
                _ => self.search_matches.len() - 1,
            });
        }
    }

    pub fn submit_search(&mut self) {
        let Some(slug) = self.search_selected_note().map(|note| note.slug.clone()) else {
            return;
        };
        self.selected_index = self.notes.iter().position(|note| note.slug == slug);
        self.cancel_mode();
        self.set_status_message(format!("selected {slug}"));
        self.reset_preview_scroll();
    }

    fn update_search_matches(&mut self) {
        match crate::fuzzy::filter_notes(&self.notes, &self.input) {
            Ok(matches) => {
                self.search_matches = matches;
                self.search_selected_index = if self.search_matches.is_empty() {
                    None
                } else {
                    Some(0)
                };
            }
            Err(error) => {
                self.mode = UiMode::Browse;
                self.search_matches.clear();
                self.search_selected_index = None;
                self.set_status_message(format!("search failed: {error}"));
            }
        }
    }

    pub fn push_input(&mut self, character: char) {
        self.input.push(character);
        if self.mode == UiMode::Search {
            self.update_search_matches();
        }
    }

    pub fn pop_input(&mut self) {
        self.input.pop();
        if self.mode == UiMode::Search {
            self.update_search_matches();
        }
    }

    pub fn clear_input(&mut self) {
        self.input.clear();
        if self.mode == UiMode::Search {
            self.update_search_matches();
        }
    }

    pub fn cancel_mode(&mut self) {
        self.input.clear();
        self.search_matches.clear();
        self.search_selected_index = None;
        self.mode = UiMode::Browse;
    }

    pub fn submit_input(&mut self) {
        if self.mode != UiMode::CreateTitle {
            return;
        }

        if let Err(error) = self.submit_create() {
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
            Ok(entry) => {
                self.cancel_mode();
                self.refresh_notes();
                self.set_status_message(format!("moved {slug} to trash ({})", entry.id));
            }
            Err(error) => self.set_status_message(format!("error: {error}")),
        }
    }

    fn submit_create(&mut self) -> Result<(), ztk::core::errors::CoreError> {
        let repo = self.repository();
        let note = create::create_note(&repo, &self.input, None)?;
        let slug = note.slug.clone();
        self.cancel_mode();
        self.refresh_notes_selecting(Some(&slug));
        self.set_status_message(format!("created {slug}"));
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
                    self.set_status_message("no notes found (use `ztk new <title>`)");
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
    use ztk::core::note::Note;

    fn sample_note(title: &str) -> Note {
        Note {
            slug: title.to_lowercase().replace(' ', "-"),
            title: title.to_string(),
            date: "2026-04-07".parse().unwrap(),
            tags: vec!["test".to_string()],
            updated_at: "2026-04-07".parse().unwrap(),
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
        assert_eq!(app.mode(), UiMode::Browse);

        app.toggle_help();
        assert!(app.show_help());
        assert_eq!(app.mode(), UiMode::Help);

        app.toggle_help();
        assert!(!app.show_help());
        assert_eq!(app.mode(), UiMode::Browse);
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
    fn tui_actions_create_search_and_delete_through_core_use_cases() {
        let root = TempDir::new().unwrap();
        let mut app = App::with_notes_dir(root.path().join("notes"));

        app.begin_create();
        app.input = "Rust Ownership".to_string();
        app.submit_input();
        assert_eq!(app.mode(), UiMode::Browse);
        assert_eq!(
            app.selected_note().map(|note| note.slug.as_str()),
            Some("rust-ownership")
        );
        assert!(root.path().join("notes/rust-ownership.md").exists());

        app.begin_delete();
        app.confirm_delete();
        assert!(app.notes().is_empty());
        assert!(!root.path().join("notes/rust-ownership.md").exists());
    }

    #[test]
    fn validation_error_preserves_mode_and_typed_input() {
        let root = TempDir::new().unwrap();
        let mut app = App::with_notes_dir(root.path().join("notes"));
        app.begin_create();
        app.input = "   ".to_string();

        app.submit_input();

        assert_eq!(app.mode(), UiMode::CreateTitle);
        assert_eq!(app.input(), "   ");
        assert!(app.status_message().contains("Title cannot be empty"));
        assert!(app.notes().is_empty());
    }
}
