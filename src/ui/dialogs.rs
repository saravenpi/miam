use crate::app::{App, Focus};
use crate::ui::colors::{PRIMARY, SECONDARY, SUCCESS};
use crate::ui::utils::centered_rect;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub fn render_input_dialog(f: &mut Frame, app: &App) {
    let area = centered_rect(60, 3, f.area());

    f.render_widget(Clear, area);

    let input = Paragraph::new(app.input.as_str())
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .title(Span::styled(" ➕ Add Feed URL ", Style::default().fg(PRIMARY)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(PRIMARY)),
        );
    f.render_widget(input, area);

    f.set_cursor_position((area.x + app.input.len() as u16 + 1, area.y + 1));
}

pub fn render_filter_dialog(f: &mut Frame, app: &App) {
    let area = centered_rect(60, 3, f.area());

    f.render_widget(Clear, area);

    let title = match app.focus {
        Focus::Feeds => " 🔍 Filter Feeds ",
        Focus::Items => " 🔍 Filter Items ",
        Focus::Tags => " 🔍 Filter Tags ",
        Focus::Reader => " 🔍 Filter ",
    };

    let filter = Paragraph::new(app.filter.as_str())
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .title(Span::styled(title, Style::default().fg(SECONDARY)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(SECONDARY)),
        );
    f.render_widget(filter, area);

    f.set_cursor_position((area.x + app.filter.len() as u16 + 1, area.y + 1));
}

pub fn render_tag_editor(f: &mut Frame, app: &App) {
    let tag_count = app.editing_tags.len() as u16;
    let tags_height = if tag_count > 0 {
        tag_count.div_ceil(3).max(3) + 2
    } else {
        0
    };
    let total_height = 3 + tags_height;

    let dialog_width = (f.area().width * 80 / 100).max(40);
    let dialog_height = total_height.min(25);
    let area = centered_rect(dialog_width, dialog_height, f.area());

    f.render_widget(Clear, area);

    let chunks = if app.editing_tags.is_empty() {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3)])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(4),
            ])
            .split(area)
    };

    let tag_input = Paragraph::new(app.tag_input.as_str())
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .title(Span::styled(" 🏷️  Add Tag (Press Enter) ", Style::default().fg(SUCCESS)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(SUCCESS)),
        );
    f.render_widget(tag_input, chunks[0]);

    if !app.editing_tags.is_empty() {
        let mut lines = Vec::new();
        let mut current_line_spans = Vec::new();
        let mut current_width = 0;
        let available_width = chunks[1].width.saturating_sub(4) as usize;

        for (i, tag) in app.editing_tags.iter().enumerate() {
            let is_selected = i == app.selected_tag_index;
            let tag_style = if is_selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(SUCCESS)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .fg(SUCCESS)
                    .bg(Color::DarkGray)
            };

            let tag_display = format!(" {} ", tag);
            let tag_width = tag_display.len() + 1;

            if current_width + tag_width > available_width && !current_line_spans.is_empty() {
                lines.push(Line::from(current_line_spans.clone()));
                current_line_spans.clear();
                current_width = 0;
            }

            current_line_spans.push(Span::styled(tag_display, tag_style));
            current_line_spans.push(Span::raw(" "));
            current_width += tag_width;
        }

        if !current_line_spans.is_empty() {
            lines.push(Line::from(current_line_spans));
        }

        let tags_display = Paragraph::new(lines)
            .block(
                Block::default()
                    .title(Span::styled(" 🏷️  Current Tags (Tab/Arrows to select, Del to remove, Enter to save) ", Style::default().fg(SUCCESS)))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(SUCCESS)),
            );
        f.render_widget(tags_display, chunks[1]);
    }

    f.set_cursor_position((chunks[0].x + app.tag_input.len() as u16 + 1, chunks[0].y + 1));
}
