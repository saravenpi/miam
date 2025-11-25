use crate::app::{App, Focus};
use crate::ui::colors::{PRIMARY, SECONDARY, SUCCESS};
use crate::ui::utils::centered_rect;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

pub fn render_input_dialog(f: &mut Frame, app: &App) {
    let area = centered_rect(60, 3, f.area());

    f.render_widget(Clear, area);

    let input = Paragraph::new(app.input.as_str())
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .title(Span::styled(" Add Feed URL ", Style::default().fg(PRIMARY)))
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
        Focus::Feeds => " Filter Feeds ",
        Focus::Items => " Filter Items ",
        Focus::Tags => " Filter Tags ",
        Focus::Reader => " Filter ",
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
    let height = 3 + (app.editing_tags.len() as u16 + 2) / 3;
    let area = centered_rect(70, height.min(20), f.area());

    f.render_widget(Clear, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
        ])
        .split(area);

    let tag_input = Paragraph::new(app.tag_input.as_str())
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .title(Span::styled(" Add Tag (Press Enter) ", Style::default().fg(SUCCESS)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(SUCCESS)),
        );
    f.render_widget(tag_input, chunks[0]);

    if !app.editing_tags.is_empty() {
        let mut spans = vec![Span::raw(" ")];
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

            spans.push(Span::styled(format!(" {} ", tag), tag_style));
            spans.push(Span::raw(" "));
        }

        let tags_display = Paragraph::new(Line::from(spans))
            .wrap(Wrap { trim: false })
            .block(
                Block::default()
                    .title(Span::styled(" Current Tags (Tab/Arrow to select, Del to remove) ", Style::default().fg(SUCCESS)))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(SUCCESS)),
            );
        f.render_widget(tags_display, chunks[1]);
    }

    f.set_cursor_position((chunks[0].x + app.tag_input.len() as u16 + 1, chunks[0].y + 1));
}
