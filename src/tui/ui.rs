use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    prelude::{Color, Modifier, Style},
    text::{Line, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};

use super::app::App;

pub fn render(frame: &mut Frame, app: &App) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(4),
        ])
        .split(frame.area());

    render_header(frame, layout[0], app);
    render_body(frame, layout[1], app);
    render_footer(frame, layout[2], app);
    render_help_overlay(frame, app);
}

fn render_header(frame: &mut Frame, area: Rect, app: &App) {
    let mode = match app.mode() {
        super::app::UiMode::Normal => "NORMAL",
        super::app::UiMode::Help => "HELP",
    };
    let title = Line::from(" Zet ").centered().style(theme::TITLE_STYLE);
    let right = format!("Mode: {mode} | Notes: {}", app.notes().len());
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
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(area);

    render_list_pane(frame, columns[0], app);
    render_preview_pane(frame, columns[1], app);
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
    let preview = if let Some(note) = app.selected_note() {
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
    };

    frame.render_widget(
        Paragraph::new(preview)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(Line::from(" Preview ").style(theme::PANE_TITLE_STYLE))
                    .style(theme::PANE_BLOCK_STYLE),
            )
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn render_footer(frame: &mut Frame, area: Rect, app: &App) {
    let status = if app.status_message().is_empty() {
        "ready"
    } else {
        app.status_message()
    };
    let text = Text::from(vec![
        Line::from(format!(" {status}")).style(theme::STATUS_TEXT_STYLE),
        Line::from(" q quit | h/? help | j/k move | g/G jump | r refresh ")
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
        Line::from(""),
        Line::from("Data").style(theme::HELP_SECTION_STYLE),
        Line::from("  r             Refresh notes from storage").style(theme::HELP_TEXT_STYLE),
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
    use ratatui::{Terminal, backend::TestBackend};

    use super::render;
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

    #[test]
    fn footer_renders_current_status_message() {
        let mut app = App::default();
        app.set_status_message("loaded 3 note(s), skipped 1 invalid file(s)");

        let output = rendered_text(&app, 80, 20);

        assert!(output.contains("loaded 3 note(s), skipped 1 invalid file(s)"));
        assert!(output.contains("q quit | h/? help"));
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
        app.set_status_message("a status message longer than the available terminal width");

        let output = rendered_text(&app, 24, 12);

        assert!(output.contains("Status"));
    }
}
