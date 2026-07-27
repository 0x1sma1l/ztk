use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Margin, Rect},
    prelude::{Color, Modifier, Style},
    text::{Line, Text},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};
use unicode_width::UnicodeWidthStr;

use super::app::App;

pub fn render(frame: &mut Frame, app: &App) {
    let layout = screen_layout(frame.area());

    render_header(frame, layout[0], app);
    render_body(frame, layout[1], app);
    render_footer(frame, layout[2], app);
    render_help_overlay(frame, app);
    render_action_overlay(frame, app);
}

fn screen_layout(area: Rect) -> std::rc::Rc<[Rect]> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(4),
        ])
        .split(area)
}

fn render_header(frame: &mut Frame, area: Rect, app: &App) {
    let mode = match app.mode() {
        super::app::UiMode::Normal => "NORMAL",
        super::app::UiMode::Help => "HELP",
        super::app::UiMode::Search => "SEARCH",
        super::app::UiMode::CreateTitle => "CREATE",
        super::app::UiMode::EditTitle => "EDIT TITLE",
        super::app::UiMode::EditTags => "EDIT TAGS",
        super::app::UiMode::EditBody => "EDIT BODY",
        super::app::UiMode::ConfirmDelete => "CONFIRM DELETE",
    };
    let title = Line::from(" Zet ").centered().style(theme::TITLE_STYLE);
    let right = format!(
        "Repo: {} | Mode: {mode} | Notes: {}",
        app.notes_dir().display(),
        app.notes().len()
    );
    frame.render_widget(
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .title_bottom(Line::from(right).right_aligned().style(theme::MUTED_STYLE))
            .style(theme::HEADER_BLOCK_STYLE),
        area,
    );
}

fn render_body(frame: &mut Frame, area: Rect, app: &App) {
    let columns = body_layout(area);

    render_list_pane(frame, columns[0], app);
    render_preview_pane(frame, columns[1], app);
}

fn body_layout(area: Rect) -> std::rc::Rc<[Rect]> {
    let (direction, constraints) = if area.width < 72 {
        (
            Direction::Vertical,
            [Constraint::Percentage(40), Constraint::Percentage(60)],
        )
    } else {
        (
            Direction::Horizontal,
            [Constraint::Percentage(35), Constraint::Percentage(65)],
        )
    };

    Layout::default()
        .direction(direction)
        .constraints(constraints)
        .split(area)
}

pub fn preview_metrics(app: &App, area: Rect) -> (u16, u16) {
    let screen = screen_layout(area);
    let body = body_layout(screen[1]);
    let inner = body[1].inner(Margin::new(1, 1));
    let page_size = inner.height.max(1);
    if inner.width == 0 || inner.height == 0 {
        return (0, page_size);
    }

    let line_count = wrapped_line_count(&preview_content(app), inner.width);
    let max_scroll = line_count.saturating_sub(inner.height as usize);
    (u16::try_from(max_scroll).unwrap_or(u16::MAX), page_size)
}

fn render_list_pane(frame: &mut Frame, area: Rect, app: &App) {
    if app.notes().is_empty() {
        frame.render_widget(
            Paragraph::new(Text::from(vec![
                Line::from("No notes loaded yet.").style(theme::MUTED_STYLE),
                Line::from(""),
                Line::from("Create one with `zet new <title>`, or press `r` to refresh.")
                    .style(theme::MUTED_STYLE),
            ]))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(Line::from(" Notes ").style(theme::PANE_TITLE_STYLE))
                    .style(theme::PANE_BLOCK_STYLE),
            )
            .wrap(Wrap { trim: true }),
            area,
        );
        return;
    }

    let items = app
        .notes()
        .iter()
        .map(|note| {
            let meta = if note.tags.is_empty() {
                note.date.clone()
            } else {
                format!("{}  [{}]", note.date, note.tags.join(", "))
            };
            ListItem::new(Text::from(vec![
                Line::from(note.title.as_str()).style(theme::ROW_TITLE_STYLE),
                Line::from(meta).style(theme::ROW_META_STYLE),
            ]))
        })
        .collect::<Vec<_>>();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Line::from(" Notes ").style(theme::PANE_TITLE_STYLE))
                .style(theme::PANE_BLOCK_STYLE),
        )
        .highlight_style(theme::SELECTED_ROW_STYLE)
        .highlight_symbol(">> ");

    let mut list_state = ListState::default().with_selected(app.selected_index());
    frame.render_stateful_widget(list, area, &mut list_state);
}

fn render_preview_pane(frame: &mut Frame, area: Rect, app: &App) {
    frame.render_widget(preview_paragraph(app), area);
}

fn preview_paragraph(app: &App) -> Paragraph<'_> {
    let preview = preview_content(app);
    let title = format!(
        " Preview [{}/{}] ",
        app.preview_scroll(),
        app.preview_max_scroll()
    );
    Paragraph::new(preview)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Line::from(title).style(theme::PANE_TITLE_STYLE))
                .style(theme::PANE_BLOCK_STYLE),
        )
        .wrap(Wrap { trim: false })
        .scroll((app.preview_scroll(), 0))
}

fn preview_content(app: &App) -> String {
    if let Some(note) = app.selected_note() {
        format!(
            "Title: {}\nSlug: {}\nDate: {}\nUpdated: {}\nTags: {}\n\n{}\n",
            note.title,
            note.slug,
            note.date,
            note.updated_at,
            if note.tags.is_empty() {
                "-".to_string()
            } else {
                note.tags.join(", ")
            },
            note.body
        )
    } else {
        "Select a note to preview details here.".to_string()
    }
}

fn wrapped_line_count(content: &str, width: u16) -> usize {
    let width = usize::from(width.max(1));
    content
        .lines()
        .map(|line| wrapped_line_height(line, width))
        .sum::<usize>()
        .max(1)
}

fn wrapped_line_height(line: &str, width: usize) -> usize {
    if line.is_empty() {
        return 1;
    }

    let mut lines = 1;
    let mut used = 0;
    for word in line.split_whitespace() {
        let word_width = UnicodeWidthStr::width(word);
        let separator = usize::from(used > 0);
        if used + separator + word_width <= width {
            used += separator + word_width;
            continue;
        }

        if used > 0 {
            lines += 1;
        }
        lines += word_width.saturating_sub(1) / width;
        used = word_width % width;
        if used == 0 && word_width > 0 {
            used = width;
        }
    }
    let display_width = UnicodeWidthStr::width(line);
    lines.max(display_width.saturating_sub(1) / width + 1)
}

fn render_footer(frame: &mut Frame, area: Rect, app: &App) {
    let status = if app.status_message().is_empty() {
        "ready"
    } else {
        app.status_message()
    };
    let text = Text::from(vec![
        Line::from(format!(" {status}")).style(theme::STATUS_TEXT_STYLE),
        Line::from(" n new | e/t/b edit | / search | d delete | h/? help ")
            .style(theme::FOOTER_TEXT_STYLE),
    ]);

    frame.render_widget(
        Paragraph::new(text).block(
            Block::default()
                .borders(Borders::ALL)
                .title(Line::from(" Status ").style(theme::PANE_TITLE_STYLE))
                .style(theme::FOOTER_BLOCK_STYLE),
        ),
        area,
    );
}

fn render_help_overlay(frame: &mut Frame, app: &App) {
    if !app.show_help() {
        return;
    }

    let popup_area = centered_rect(70, 60, frame.area());
    let help_text = Text::from(vec![
        Line::from(" Zet Help ")
            .style(theme::HELP_TITLE_STYLE)
            .centered(),
        Line::from(""),
        Line::from("Navigation").style(theme::HELP_SECTION_STYLE),
        Line::from("  j / Down      Move selection down").style(theme::HELP_TEXT_STYLE),
        Line::from("  k / Up        Move selection up").style(theme::HELP_TEXT_STYLE),
        Line::from("  g / Home      Jump to first note").style(theme::HELP_TEXT_STYLE),
        Line::from("  G / End       Jump to last note").style(theme::HELP_TEXT_STYLE),
        Line::from("  [ / ]         Scroll preview one line").style(theme::HELP_TEXT_STYLE),
        Line::from("  PgUp / PgDn   Scroll preview one page").style(theme::HELP_TEXT_STYLE),
        Line::from(""),
        Line::from("Data").style(theme::HELP_SECTION_STYLE),
        Line::from("  r             Refresh notes from storage").style(theme::HELP_TEXT_STYLE),
        Line::from("  /             Search notes").style(theme::HELP_TEXT_STYLE),
        Line::from("  n             Create a note").style(theme::HELP_TEXT_STYLE),
        Line::from("  e / t / b     Edit title / tags / body").style(theme::HELP_TEXT_STYLE),
        Line::from("  d             Move selected note to trash with confirmation")
            .style(theme::HELP_TEXT_STYLE),
        Line::from(""),
        Line::from("General").style(theme::HELP_SECTION_STYLE),
        Line::from("  h / ?         Toggle this help panel").style(theme::HELP_TEXT_STYLE),
        Line::from("  q / Esc       Quit Zet TUI").style(theme::HELP_TEXT_STYLE),
        Line::from("  Ctrl-C        Force quit").style(theme::HELP_TEXT_STYLE),
    ]);

    frame.render_widget(
        Block::default().style(theme::OVERLAY_BACKDROP_STYLE),
        popup_area,
    );
    frame.render_widget(
        Paragraph::new(help_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(Line::from(" Keymap ").style(theme::HELP_SECTION_STYLE))
                    .style(theme::HELP_BLOCK_STYLE),
            )
            .wrap(Wrap { trim: true }),
        popup_area,
    );
}

fn render_action_overlay(frame: &mut Frame, app: &App) {
    use super::app::UiMode;

    let (title, prompt) = match app.mode() {
        UiMode::Search => (" Search ", "Query"),
        UiMode::CreateTitle => (" Create note ", "Title"),
        UiMode::EditTitle => (" Edit title ", "Title"),
        UiMode::EditTags => (" Edit tags ", "Comma-separated tags"),
        UiMode::EditBody => (" Edit body ", "Markdown body"),
        UiMode::ConfirmDelete => {
            let slug = app
                .selected_note()
                .map(|note| note.slug.as_str())
                .unwrap_or("note");
            let area = centered_rect(70, 24, frame.area());
            frame.render_widget(Clear, area);
            frame.render_widget(
                Paragraph::new(format!(
                    "Move {slug} to recoverable trash? Press y to confirm, n or Esc to cancel."
                ))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Confirm delete "),
                )
                .wrap(Wrap { trim: true }),
                area,
            );
            return;
        }
        UiMode::Normal | UiMode::Help => return,
    };

    let area = centered_rect(80, 30, frame.area());
    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(format!(
            "{prompt}:\n{}\n\nEnter submit | Esc cancel | Ctrl-U clear",
            app.input()
        ))
        .block(Block::default().borders(Borders::ALL).title(title))
        .wrap(Wrap { trim: false }),
        area,
    );
}

fn centered_rect(width_percent: u16, height_percent: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - height_percent) / 2),
            Constraint::Percentage(height_percent),
            Constraint::Percentage((100 - height_percent) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - width_percent) / 2),
            Constraint::Percentage(width_percent),
            Constraint::Percentage((100 - width_percent) / 2),
        ])
        .split(vertical[1])[1]
}

mod theme {
    use super::*;

    pub const TITLE_STYLE: Style = Style::new()
        .fg(Color::Rgb(246, 248, 255))
        .bg(Color::Rgb(25, 35, 60))
        .add_modifier(Modifier::BOLD);

    pub const MUTED_STYLE: Style = Style::new().fg(Color::Rgb(150, 162, 184));
    pub const HEADER_BLOCK_STYLE: Style = Style::new().bg(Color::Rgb(16, 20, 35));
    pub const PANE_BLOCK_STYLE: Style = Style::new().bg(Color::Rgb(10, 14, 24));
    pub const PANE_TITLE_STYLE: Style = Style::new()
        .fg(Color::Rgb(130, 185, 255))
        .add_modifier(Modifier::BOLD);
    pub const ROW_TITLE_STYLE: Style = Style::new().fg(Color::Rgb(208, 222, 248));
    pub const ROW_META_STYLE: Style = Style::new().fg(Color::Rgb(138, 154, 179));
    pub const SELECTED_ROW_STYLE: Style = Style::new()
        .fg(Color::Rgb(255, 244, 214))
        .bg(Color::Rgb(80, 52, 24))
        .add_modifier(Modifier::BOLD);
    pub const FOOTER_BLOCK_STYLE: Style = Style::new().bg(Color::Rgb(20, 25, 38));
    pub const STATUS_TEXT_STYLE: Style = Style::new().fg(Color::Rgb(255, 204, 128));
    pub const FOOTER_TEXT_STYLE: Style = Style::new().fg(Color::Rgb(200, 210, 230));
    pub const OVERLAY_BACKDROP_STYLE: Style = Style::new().bg(Color::Rgb(12, 16, 26));
    pub const HELP_BLOCK_STYLE: Style = Style::new().bg(Color::Rgb(18, 24, 40));
    pub const HELP_TITLE_STYLE: Style = Style::new()
        .fg(Color::Rgb(237, 243, 255))
        .add_modifier(Modifier::BOLD);
    pub const HELP_SECTION_STYLE: Style = Style::new()
        .fg(Color::Rgb(130, 185, 255))
        .add_modifier(Modifier::BOLD);
    pub const HELP_TEXT_STYLE: Style = Style::new().fg(Color::Rgb(210, 220, 240));
}

#[cfg(test)]
mod tests {
    use ratatui::{Terminal, backend::TestBackend, layout::Rect};
    use zet::core::note::Note;

    use super::{preview_metrics, render};
    use crate::tui::app::App;

    fn rendered_text(app: &App, width: u16, height: u16) -> String {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).expect("test terminal should initialize");

        terminal
            .draw(|frame| render(frame, app))
            .expect("TUI render should succeed");

        terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect::<Vec<_>>()
            .join("")
    }

    fn note_with_body(body: &str) -> Note {
        Note {
            slug: "scroll-test".to_string(),
            title: "Scroll Test".to_string(),
            date: "2026-07-27".to_string(),
            tags: vec!["unicode".to_string()],
            updated_at: "2026-07-27".to_string(),
            body: body.to_string(),
        }
    }

    #[test]
    fn footer_renders_current_status_message() {
        let mut app = App::default();
        app.set_status_message("loaded 3 note(s), skipped 1 invalid file(s)");

        let output = rendered_text(&app, 80, 20);

        assert!(output.contains("loaded 3 note(s), skipped 1 invalid file(s)"));
        assert!(output.contains("n new | e/t/b edit"));
    }

    #[test]
    fn footer_renders_ready_when_status_is_empty() {
        let app = App::default();

        let output = rendered_text(&app, 50, 16);

        assert!(output.contains("ready"));
    }

    #[test]
    fn narrow_terminal_render_does_not_panic() {
        let mut app = App::default();
        app.set_notes(vec![note_with_body("Preview body")]);
        app.set_status_message("a status message longer than the available terminal width");

        let output = rendered_text(&app, 40, 18);

        assert!(output.contains("Status"));
        assert!(output.contains("Notes"));
        assert!(output.contains("Preview"));
    }

    #[test]
    fn wrapped_unicode_preview_is_scrollable_to_the_end() {
        let mut app = App::default();
        let body = (0..20)
            .map(|index| format!("行 {index}: café 🚀 with wrapped words"))
            .collect::<Vec<_>>()
            .join("\n");
        app.set_notes(vec![note_with_body(&body)]);
        let area = Rect::new(0, 0, 48, 20);
        let (max_scroll, page_size) = preview_metrics(&app, area);
        app.update_preview_metrics(max_scroll, page_size);

        assert!(max_scroll > 0);
        app.scroll_preview_down(u16::MAX);
        let output = rendered_text(&app, area.width, area.height);

        assert!(output.contains("19: café"));
    }

    #[test]
    fn empty_and_short_previews_have_no_scroll_range() {
        for body in ["", "short"] {
            let mut app = App::default();
            app.set_notes(vec![note_with_body(body)]);

            let (max_scroll, _) = preview_metrics(&app, Rect::new(0, 0, 100, 30));

            assert_eq!(max_scroll, 0, "body: {body:?}");
        }
    }

    #[test]
    fn action_modes_render_prompts_and_confirmation() {
        let mut app = App::default();
        app.set_notes(vec![note_with_body("body")]);
        app.begin_input(crate::tui::app::UiMode::Search);
        app.push_input('r');
        let search = rendered_text(&app, 80, 24);
        assert!(search.contains("Query:"));
        assert!(search.contains("Enter submit"));

        app.cancel_mode();
        app.begin_delete();
        let delete = rendered_text(&app, 80, 24);
        assert!(delete.contains("Move scroll-test to recoverable trash?"));
    }
}
