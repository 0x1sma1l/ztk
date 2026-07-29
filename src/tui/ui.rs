use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Margin, Rect},
    prelude::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};
use unicode_width::UnicodeWidthStr;

use super::app::{App, UiMode};

pub fn render(frame: &mut Frame, app: &App) {
    let layout = screen_layout(frame.area());

    render_header(frame, layout[0], app);
    render_body(frame, layout[1], app);
    render_footer(frame, layout[2], app);
    render_help_overlay(frame, app);
    render_action_overlay(frame, app);
    render_search_overlay(frame, app);
}

fn screen_layout(area: Rect) -> std::rc::Rc<[Rect]> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(8),
            Constraint::Length(2),
        ])
        .split(area)
}

fn render_header(frame: &mut Frame, area: Rect, app: &App) {
    let mode = match app.mode() {
        super::app::UiMode::Browse => "BROWSE",
        super::app::UiMode::Read => "READ",
        super::app::UiMode::Editor => "EDITOR",
        super::app::UiMode::Help => "HELP",
        super::app::UiMode::CreateTitle => "CREATE",
        super::app::UiMode::Search => "SEARCH",
        super::app::UiMode::ConfirmDelete => "CONFIRM DELETE",
    };
    let columns = Layout::horizontal([Constraint::Length(8), Constraint::Min(0)]).split(area);
    frame.render_widget(
        Paragraph::new(Line::from(" Ztk ").style(theme::TITLE_STYLE)),
        columns[0],
    );
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(app.notes_dir().display().to_string(), theme::MUTED_STYLE),
            Span::styled("  ·  ", theme::MUTED_STYLE),
            Span::styled(mode, theme::MODE_STYLE),
            Span::styled(
                format!("  ·  {} notes ", app.notes().len()),
                theme::MUTED_STYLE,
            ),
        ]))
        .right_aligned(),
        columns[1],
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
            [Constraint::Percentage(28), Constraint::Percentage(72)],
        )
    };

    Layout::default()
        .direction(direction)
        .constraints(constraints)
        .split(area)
}

pub fn preview_metrics(app: &App, area: Rect) -> (u16, u16) {
    let (width, height) = note_surface_size(area);
    let page_size = height.max(1);
    if width == 0 || height == 0 {
        return (0, page_size);
    }

    let line_count = wrapped_line_count(&preview_content(app), width);
    let max_scroll = line_count.saturating_sub(height as usize);
    (u16::try_from(max_scroll).unwrap_or(u16::MAX), page_size)
}

pub fn note_surface_size(area: Rect) -> (u16, u16) {
    let screen = screen_layout(area);
    let body = body_layout(screen[1]);
    let inner = Block::default().borders(Borders::TOP).inner(body[1]);
    (inner.width.max(1), inner.height.max(1))
}

fn render_list_pane(frame: &mut Frame, area: Rect, app: &App) {
    if app.notes().is_empty() {
        frame.render_widget(
            Paragraph::new(Text::from(vec![
                Line::from("No notes loaded yet.").style(theme::MUTED_STYLE),
                Line::from(""),
                Line::from("Create one with `ztk new <title>`, or press `r` to refresh.")
                    .style(theme::MUTED_STYLE),
            ]))
            .block(
                Block::default()
                    .borders(Borders::TOP | Borders::RIGHT)
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
                note.date.to_string()
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
                .borders(Borders::TOP | Borders::RIGHT)
                .title(Line::from(" Notes ").style(theme::PANE_TITLE_STYLE))
                .style(theme::PANE_BLOCK_STYLE),
        )
        .highlight_style(theme::SELECTED_ROW_STYLE)
        .highlight_symbol("› ");

    let mut list_state = ListState::default().with_selected(app.selected_index());
    frame.render_stateful_widget(list, area, &mut list_state);
}

fn render_preview_pane(frame: &mut Frame, area: Rect, app: &App) {
    let title = match app.mode() {
        super::app::UiMode::Read => " Note · read ",
        super::app::UiMode::Editor => " Note · editor ",
        _ => " Note ",
    };
    let block = Block::default()
        .borders(Borders::TOP)
        .title(Line::from(title).style(theme::PANE_TITLE_STYLE))
        .style(theme::PANE_BLOCK_STYLE);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.mode() == super::app::UiMode::Editor {
        if let Some(editor) = app.editor() {
            editor.render(frame, inner);
            return;
        }
    }

    frame.render_widget(preview_paragraph(app), inner);
}

fn preview_paragraph(app: &App) -> Paragraph<'_> {
    let preview = preview_content(app);
    Paragraph::new(preview)
        .style(theme::BODY_TEXT_STYLE)
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
    let keys = footer_keys(app.mode());
    let text = Text::from(vec![
        Line::from(vec![
            Span::styled(" ● ", theme::STATUS_DOT_STYLE),
            Span::styled(status, theme::STATUS_TEXT_STYLE),
        ]),
        Line::from(keys).style(theme::FOOTER_TEXT_STYLE),
    ]);

    frame.render_widget(Paragraph::new(text).style(theme::FOOTER_BLOCK_STYLE), area);
}

fn footer_keys(mode: UiMode) -> &'static str {
    match mode {
        UiMode::Read => " j/k scroll  PgUp/PgDn page  e edit  Esc notes ",
        UiMode::Editor => " editor owns keys  :wq save + apply  :q apply saved  F6 detach only ",
        _ => " n new  Enter read  e edit  / search  h/? help  q/Esc exit ",
    }
}

fn render_help_overlay(frame: &mut Frame, app: &App) {
    if !app.show_help() {
        return;
    }

    let popup_area = centered_rect(70, 60, frame.area());
    let help_text = Text::from(vec![
        Line::from(" Ztk Help ")
            .style(theme::HELP_TITLE_STYLE)
            .centered(),
        Line::from(""),
        Line::from("Navigation").style(theme::HELP_SECTION_STYLE),
        Line::from("  j / Down      Move selection down").style(theme::HELP_TEXT_STYLE),
        Line::from("  k / Up        Move selection up").style(theme::HELP_TEXT_STYLE),
        Line::from("  g / Home      Jump to first note").style(theme::HELP_TEXT_STYLE),
        Line::from("  G / End       Jump to last note").style(theme::HELP_TEXT_STYLE),
        Line::from("  Enter         Focus the note for reading").style(theme::HELP_TEXT_STYLE),
        Line::from("  [ / ]         Scroll preview one line").style(theme::HELP_TEXT_STYLE),
        Line::from("  PgUp / PgDn   Scroll preview one page").style(theme::HELP_TEXT_STYLE),
        Line::from(""),
        Line::from("Data").style(theme::HELP_SECTION_STYLE),
        Line::from("  r             Refresh notes from storage").style(theme::HELP_TEXT_STYLE),
        Line::from("  /             Search notes").style(theme::HELP_TEXT_STYLE),
        Line::from("  n             Create a note").style(theme::HELP_TEXT_STYLE),
        Line::from("  e             Edit in the configured or detected terminal editor")
            .style(theme::HELP_TEXT_STYLE),
        Line::from("  d             Move selected note to trash with confirmation")
            .style(theme::HELP_TEXT_STYLE),
        Line::from(""),
        Line::from("General").style(theme::HELP_SECTION_STYLE),
        Line::from("  h / ?         Toggle this help panel").style(theme::HELP_TEXT_STYLE),
        Line::from("  q / Esc       Quit Ztk TUI").style(theme::HELP_TEXT_STYLE),
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
    let (title, prompt) = match app.mode() {
        UiMode::CreateTitle => (" Create note ", "Title"),
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
                .style(theme::BODY_TEXT_STYLE)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(Line::from(" Confirm delete ").style(theme::PANE_TITLE_STYLE))
                        .style(theme::OVERLAY_BLOCK_STYLE),
                )
                .wrap(Wrap { trim: true }),
                area,
            );
            return;
        }
        UiMode::Browse | UiMode::Read | UiMode::Editor | UiMode::Help | UiMode::Search => return,
    };

    let area = centered_rect(80, 30, frame.area());
    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(format!(
            "{prompt}:\n{}\n\nEnter submit | Esc cancel | Ctrl-U clear",
            app.input()
        ))
        .style(theme::BODY_TEXT_STYLE)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Line::from(title).style(theme::PANE_TITLE_STYLE))
                .style(theme::OVERLAY_BLOCK_STYLE),
        )
        .wrap(Wrap { trim: false }),
        area,
    );
}

fn render_search_overlay(frame: &mut Frame, app: &App) {
    if app.mode() != super::app::UiMode::Search {
        return;
    }

    let area = centered_rect(80, 70, frame.area());
    frame.render_widget(Clear, area);
    frame.render_widget(
        Block::default()
            .borders(Borders::ALL)
            .title(Line::from(" Search notes ").style(theme::PANE_TITLE_STYLE))
            .style(theme::SEARCH_BLOCK_STYLE),
        area,
    );

    let inner = area.inner(Margin::new(1, 1));
    let rows = Layout::vertical([
        Constraint::Length(2),
        Constraint::Min(3),
        Constraint::Length(1),
    ])
    .split(inner);
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" ❯ ", theme::SEARCH_POINTER_STYLE),
            Span::styled(app.input(), theme::BODY_TEXT_STYLE),
        ]))
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .style(theme::SEARCH_BLOCK_STYLE),
        ),
        rows[0],
    );

    let panes = if rows[1].width < 70 {
        Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)]).split(rows[1])
    } else {
        Layout::horizontal([Constraint::Percentage(45), Constraint::Percentage(55)]).split(rows[1])
    };

    let items = app
        .search_matches()
        .iter()
        .filter_map(|slug| app.notes().iter().find(|note| note.slug == *slug))
        .map(|note| {
            ListItem::new(Line::from(vec![
                Span::styled(note.title.as_str(), theme::ROW_TITLE_STYLE),
                Span::styled(
                    if note.tags.is_empty() {
                        String::new()
                    } else {
                        format!("  [{}]", note.tags.join(", "))
                    },
                    theme::ROW_META_STYLE,
                ),
            ]))
        })
        .collect::<Vec<_>>();
    let results_border = if rows[1].width < 70 {
        Borders::BOTTOM
    } else {
        Borders::RIGHT
    };
    let results = List::new(items)
        .block(
            Block::default()
                .borders(results_border)
                .title(
                    Line::from(format!(" Results ({}) ", app.search_matches().len()))
                        .style(theme::PANE_TITLE_STYLE),
                )
                .style(theme::SEARCH_BLOCK_STYLE),
        )
        .highlight_style(theme::SELECTED_ROW_STYLE)
        .highlight_symbol("› ");
    let mut state = ListState::default().with_selected(app.search_selected_index());
    frame.render_stateful_widget(results, panes[0], &mut state);

    let preview = app
        .search_selected_note()
        .map(note_preview_content)
        .unwrap_or_else(|| "No matching notes".to_string());
    frame.render_widget(
        Paragraph::new(preview)
            .style(theme::BODY_TEXT_STYLE)
            .block(
                Block::default()
                    .title(Line::from(" Preview ").style(theme::PANE_TITLE_STYLE))
                    .style(theme::SEARCH_BLOCK_STYLE),
            )
            .wrap(Wrap { trim: false }),
        panes[1],
    );
    frame.render_widget(
        Paragraph::new(" ↑/↓ navigate  Enter select  Esc close  Ctrl-U clear ")
            .style(theme::MUTED_STYLE),
        rows[2],
    );
}

fn note_preview_content(note: &ztk::core::note::Note) -> String {
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

    // Preserve the terminal/fzf background and use Vesper's peach-orange accent
    // throughout. The selected surface matches Vesper's active list background.
    const ACCENT: Color = Color::Rgb(255, 199, 153);
    const ACCENT_TEXT: Color = ACCENT;
    const PRIMARY_TEXT: Color = Color::Indexed(252);
    const SECONDARY_TEXT: Color = Color::Indexed(245);
    const BORDER: Color = Color::Indexed(239);
    const SELECTED_SURFACE: Color = Color::Rgb(35, 35, 35);

    pub const TITLE_STYLE: Style = Style::new()
        .fg(ACCENT)
        .bg(Color::Reset)
        .add_modifier(Modifier::BOLD);

    pub const MODE_STYLE: Style = Style::new().fg(ACCENT).add_modifier(Modifier::BOLD);
    pub const BODY_TEXT_STYLE: Style = Style::new().fg(PRIMARY_TEXT).bg(Color::Reset);
    pub const MUTED_STYLE: Style = Style::new().fg(SECONDARY_TEXT).bg(Color::Reset);
    pub const PANE_BLOCK_STYLE: Style = Style::new().fg(BORDER).bg(Color::Reset);
    pub const PANE_TITLE_STYLE: Style = Style::new().fg(ACCENT).add_modifier(Modifier::BOLD);
    pub const ROW_TITLE_STYLE: Style = Style::new().fg(PRIMARY_TEXT);
    pub const ROW_META_STYLE: Style = Style::new().fg(SECONDARY_TEXT);
    pub const SELECTED_ROW_STYLE: Style = Style::new()
        .fg(ACCENT_TEXT)
        .bg(SELECTED_SURFACE)
        .add_modifier(Modifier::BOLD);
    pub const FOOTER_BLOCK_STYLE: Style = Style::new().bg(Color::Reset);
    pub const STATUS_DOT_STYLE: Style = Style::new().fg(ACCENT);
    pub const STATUS_TEXT_STYLE: Style = Style::new().fg(PRIMARY_TEXT);
    pub const FOOTER_TEXT_STYLE: Style = Style::new().fg(SECONDARY_TEXT);
    pub const OVERLAY_BACKDROP_STYLE: Style = Style::new().bg(Color::Reset);
    pub const OVERLAY_BLOCK_STYLE: Style = Style::new().fg(BORDER).bg(Color::Reset);
    pub const HELP_BLOCK_STYLE: Style = Style::new().fg(BORDER).bg(Color::Reset);
    pub const SEARCH_BLOCK_STYLE: Style = Style::new().fg(BORDER).bg(Color::Reset);
    pub const SEARCH_POINTER_STYLE: Style = Style::new().fg(ACCENT).add_modifier(Modifier::BOLD);
    pub const HELP_TITLE_STYLE: Style = Style::new().fg(ACCENT_TEXT).add_modifier(Modifier::BOLD);
    pub const HELP_SECTION_STYLE: Style = Style::new().fg(ACCENT).add_modifier(Modifier::BOLD);
    pub const HELP_TEXT_STYLE: Style = Style::new().fg(PRIMARY_TEXT);
}

#[cfg(test)]
mod tests {
    use ratatui::{Terminal, backend::TestBackend, layout::Rect, prelude::Color};
    use ztk::core::note::Note;

    use super::{footer_keys, preview_metrics, render, theme};
    use crate::tui::app::{App, UiMode};

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
            date: "2026-07-27".parse().unwrap(),
            tags: vec!["unicode".to_string()],
            updated_at: "2026-07-27".parse().unwrap(),
            body: body.to_string(),
        }
    }

    #[test]
    fn theme_uses_one_accent_and_preserves_the_terminal_background() {
        for style in [
            theme::TITLE_STYLE,
            theme::MODE_STYLE,
            theme::PANE_TITLE_STYLE,
            theme::STATUS_DOT_STYLE,
            theme::SEARCH_POINTER_STYLE,
            theme::HELP_SECTION_STYLE,
        ] {
            assert_eq!(style.fg, Some(Color::Rgb(255, 199, 153)));
        }
        assert_eq!(theme::BODY_TEXT_STYLE.bg, Some(Color::Reset));
        assert_eq!(
            theme::SELECTED_ROW_STYLE.fg,
            Some(Color::Rgb(255, 199, 153))
        );
        assert_eq!(theme::SELECTED_ROW_STYLE.bg, Some(Color::Rgb(35, 35, 35)));
    }

    #[test]
    fn footer_renders_current_status_message() {
        let mut app = App::default();
        app.set_status_message("loaded 3 note(s), skipped 1 invalid file(s)");

        let output = rendered_text(&app, 80, 20);

        assert!(output.contains("loaded 3 note(s), skipped 1 invalid file(s)"));
        assert!(output.contains("n new  Enter read  e edit"));
    }

    #[test]
    fn footer_renders_ready_when_status_is_empty() {
        let app = App::default();

        let output = rendered_text(&app, 50, 16);

        assert!(output.contains("ready"));
    }

    #[test]
    fn editor_footer_distinguishes_applying_changes_from_detaching() {
        let keys = footer_keys(UiMode::Editor);

        assert!(keys.contains(":wq save + apply"));
        assert!(keys.contains(":q apply saved"));
        assert!(keys.contains("F6 detach only"));
    }

    #[test]
    fn narrow_terminal_render_does_not_panic() {
        let mut app = App::default();
        app.set_notes(vec![note_with_body("Preview body")]);
        app.set_status_message("a status message longer than the available terminal width");

        let output = rendered_text(&app, 40, 18);

        assert!(output.contains("n new"));
        assert!(output.contains("Notes"));
        assert!(output.contains("Note"));
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
        app.begin_delete();
        let delete = rendered_text(&app, 80, 24);
        assert!(delete.contains("Move scroll-test to recoverable trash?"));
    }
}
