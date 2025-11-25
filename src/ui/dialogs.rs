use crate::app::{App, Focus};
use crate::ui::colors::{PRIMARY, SECONDARY};
use crate::ui::utils::centered_rect;
use ratatui::{
    style::{Color, Style},
    text::Span,
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
