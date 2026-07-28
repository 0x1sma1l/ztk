use super::app::{App, UiMode};
use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
/// Reads the crossterm events and updates the state of [`App`].
///
/// If your application needs to perform work in between handling events, you can use the
/// [`event::poll`] function to check if there are any events available with a timeout.
pub fn handle_crossterm_events(app: &mut App) -> Result<()> {
    match event::read()? {
        // it's important to check KeyEventKind::Press to avoid handling key release events
        Event::Key(key) if key.kind == KeyEventKind::Press => on_key_event(app, key),
        Event::Mouse(_) => {}
        Event::Resize(cols, rows) => app.handle_resize(cols, rows),
        _ => {}
    }
    Ok(())
}

/// Handles the key events and updates the state of [`App`].
fn on_key_event(app: &mut App, key: KeyEvent) {
    if app.show_help() {
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('h' | '?')) => app.toggle_help(),
            (_, KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => app.quit(),
            _ => {}
        }
        return;
    }

    if app.mode() == UiMode::ConfirmDelete {
        match key.code {
            KeyCode::Char('y' | 'Y') => app.confirm_delete(),
            KeyCode::Char('n' | 'N') | KeyCode::Esc => {
                app.cancel_mode();
                app.set_status_message("delete cancelled");
            }
            _ => {}
        }
        return;
    }

    if matches!(
        app.mode(),
        UiMode::CreateTitle | UiMode::EditTitle | UiMode::EditTags | UiMode::EditBody
    ) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc) => app.cancel_mode(),
            (_, KeyCode::Enter) => app.submit_input(),
            (_, KeyCode::Backspace) => app.pop_input(),
            (KeyModifiers::CONTROL, KeyCode::Char('u' | 'U')) => app.clear_input(),
            (KeyModifiers::NONE | KeyModifiers::SHIFT, KeyCode::Char(character)) => {
                app.push_input(character)
            }
            _ => {}
        }
        return;
    }

    match (key.modifiers, key.code) {
        (_, KeyCode::Esc | KeyCode::Char('q'))
        | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => app.quit(),
        (_, KeyCode::Char('h' | '?')) => app.toggle_help(),
        (_, KeyCode::Down | KeyCode::Char('j')) => app.select_next(),
        (_, KeyCode::Up | KeyCode::Char('k')) => app.select_previous(),
        (_, KeyCode::Home | KeyCode::Char('g')) => app.select_first(),
        (_, KeyCode::End | KeyCode::Char('G')) => app.select_last(),
        (_, KeyCode::Char(']')) => app.scroll_preview_down(1),
        (_, KeyCode::Char('[')) => app.scroll_preview_up(1),
        (_, KeyCode::PageDown) => app.scroll_preview_page_down(),
        (_, KeyCode::PageUp) => app.scroll_preview_page_up(),
        (_, KeyCode::Char('r')) => app.refresh_notes(),
        (_, KeyCode::Char('/')) => app.request_search(),
        (_, KeyCode::Char('n')) => app.begin_input(UiMode::CreateTitle),
        (_, KeyCode::Char('e')) => app.begin_input(UiMode::EditTitle),
        (_, KeyCode::Char('t')) => app.begin_input(UiMode::EditTags),
        (_, KeyCode::Char('b')) => app.begin_input(UiMode::EditBody),
        (_, KeyCode::Char('d')) => app.begin_delete(),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    use super::on_key_event;
    use crate::tui::app::App;
    use ztk::core::note::Note;

    fn note(title: &str) -> Note {
        Note {
            slug: title.to_lowercase(),
            title: title.to_string(),
            date: "2026-07-27".parse().unwrap(),
            tags: vec![],
            updated_at: "2026-07-27".parse().unwrap(),
            body: String::new(),
        }
    }

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn navigation_keys_update_selection() {
        let mut app = App::default();
        app.set_notes(vec![note("First"), note("Second"), note("Third")]);

        on_key_event(&mut app, key(KeyCode::Char('j')));
        assert_eq!(app.selected_index(), Some(1));

        on_key_event(&mut app, key(KeyCode::Down));
        assert_eq!(app.selected_index(), Some(2));

        on_key_event(&mut app, key(KeyCode::Char('k')));
        assert_eq!(app.selected_index(), Some(1));

        on_key_event(&mut app, key(KeyCode::Home));
        assert_eq!(app.selected_index(), Some(0));

        on_key_event(&mut app, key(KeyCode::Char('G')));
        assert_eq!(app.selected_index(), Some(2));
    }

    #[test]
    fn help_keys_toggle_help_and_escape_closes_it() {
        let mut app = App::default();

        on_key_event(&mut app, key(KeyCode::Char('?')));
        assert!(app.show_help());

        on_key_event(&mut app, key(KeyCode::Esc));
        assert!(!app.show_help());

        on_key_event(&mut app, key(KeyCode::Char('h')));
        assert!(app.show_help());
    }

    #[test]
    fn help_mode_blocks_navigation_and_scrolling() {
        let mut app = App::default();
        app.set_notes(vec![note("First"), note("Second")]);
        app.update_preview_metrics(10, 4);
        on_key_event(&mut app, key(KeyCode::Char('?')));

        on_key_event(&mut app, key(KeyCode::Char('j')));
        on_key_event(&mut app, key(KeyCode::PageDown));

        assert_eq!(app.selected_index(), Some(0));
        assert_eq!(app.preview_scroll(), 0);
    }

    #[test]
    fn preview_scroll_keys_respect_bounds() {
        let mut app = App::default();
        app.update_preview_metrics(10, 4);

        on_key_event(&mut app, key(KeyCode::Char(']')));
        assert_eq!(app.preview_scroll(), 1);
        on_key_event(&mut app, key(KeyCode::PageDown));
        assert_eq!(app.preview_scroll(), 5);
        on_key_event(&mut app, key(KeyCode::PageDown));
        on_key_event(&mut app, key(KeyCode::PageDown));
        assert_eq!(app.preview_scroll(), 10);
        on_key_event(&mut app, key(KeyCode::PageUp));
        assert_eq!(app.preview_scroll(), 6);
        on_key_event(&mut app, key(KeyCode::Char('[')));
        assert_eq!(app.preview_scroll(), 5);
    }

    #[test]
    fn input_modes_capture_text_and_escape_cancels() {
        let mut app = App::default();

        on_key_event(&mut app, key(KeyCode::Char('n')));
        assert_eq!(app.mode(), crate::tui::app::UiMode::CreateTitle);
        on_key_event(&mut app, key(KeyCode::Char('r')));
        on_key_event(&mut app, key(KeyCode::Char('u')));
        on_key_event(&mut app, key(KeyCode::Char('s')));
        on_key_event(&mut app, key(KeyCode::Char('t')));
        assert_eq!(app.input(), "rust");
        on_key_event(&mut app, key(KeyCode::Backspace));
        assert_eq!(app.input(), "rus");
        on_key_event(&mut app, key(KeyCode::Esc));
        assert_eq!(app.mode(), crate::tui::app::UiMode::Normal);
        assert!(app.input().is_empty());
    }

    #[test]
    fn slash_requests_external_fuzzy_search() {
        let mut app = App::default();

        on_key_event(&mut app, key(KeyCode::Char('/')));

        assert!(app.search_requested());
        assert_eq!(app.mode(), crate::tui::app::UiMode::Normal);
    }

    #[test]
    fn deletion_requires_explicit_confirmation() {
        let mut app = App::default();
        app.set_notes(vec![note("First")]);

        on_key_event(&mut app, key(KeyCode::Char('d')));
        assert_eq!(app.mode(), crate::tui::app::UiMode::ConfirmDelete);
        on_key_event(&mut app, key(KeyCode::Char('n')));

        assert_eq!(app.mode(), crate::tui::app::UiMode::Normal);
        assert_eq!(app.notes().len(), 1);
        assert_eq!(app.status_message(), "delete cancelled");
    }
}
