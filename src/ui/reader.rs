use crate::app::App;
use crate::ui::colors::{DIM, PRIMARY, SECONDARY, SUCCESS};
use crate::ui::utils::truncate;
use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render(f: &mut Frame, app: &App) {
    let chunks = Layout::vertical([Constraint::Min(0), Constraint::Length(3)]).split(f.area());

    if app.article_loading {
        render_loading(f, app, chunks[0]);
    } else if let Some(article) = &app.current_article {
        render_article(f, app, article, chunks[0]);
    } else {
        render_no_article(f, chunks[0]);
    }

    render_help(f, chunks[1]);
}

fn render_loading(f: &mut Frame, app: &App, area: Rect) {
    let loading = Paragraph::new(format!("{} Loading article...", app.spinner_char()))
        .style(Style::default().fg(PRIMARY))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .title(Span::styled(" Reader ", Style::default().fg(PRIMARY)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(PRIMARY)),
        );
    f.render_widget(loading, area);
}

fn render_article(f: &mut Frame, app: &App, article: &crate::reader::Article, area: Rect) {
    let content_width = area.width.saturating_sub(4) as usize;
    let wrapped_lines: Vec<Line> = article
        .content
        .lines()
        .flat_map(|line| {
            if line.is_empty() {
                vec![Line::from("")]
            } else {
                textwrap::wrap(line, content_width)
                    .into_iter()
                    .map(|s| Line::from(s.to_string()))
                    .collect::<Vec<_>>()
            }
        })
        .collect();

    let total_lines = wrapped_lines.len();
    let visible_height = area.height.saturating_sub(2) as usize;
    let max_scroll = total_lines.saturating_sub(visible_height);
    let scroll = (app.article_scroll as usize).min(max_scroll);

    let visible_lines: Vec<Line> = wrapped_lines
        .into_iter()
        .skip(scroll)
        .take(visible_height)
        .collect();

    let scroll_indicator = if total_lines > visible_height {
        format!(" [{}/{}] ", scroll + 1, max_scroll + 1)
    } else {
        String::new()
    };

    let title = format!(" {} {}", truncate(&article.title, 60), scroll_indicator);

    let content = Paragraph::new(visible_lines)
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .title(Span::styled(title, Style::default().fg(PRIMARY)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(PRIMARY)),
        );
    f.render_widget(content, area);
}

fn render_no_article(f: &mut Frame, area: Rect) {
    let empty = Paragraph::new("No article loaded")
        .style(Style::default().fg(DIM))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .title(Span::styled(" Reader ", Style::default().fg(SECONDARY)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(DIM)),
        );
    f.render_widget(empty, area);
}

fn render_help(f: &mut Frame, area: Rect) {
    let help = Paragraph::new(Line::from(vec![
        Span::styled("Esc", Style::default().fg(SUCCESS)),
        Span::raw("/"),
        Span::styled("q", Style::default().fg(SUCCESS)),
        Span::raw(" back  "),
        Span::styled("j/k", Style::default().fg(SUCCESS)),
        Span::raw(" scroll  "),
        Span::styled("Space/b", Style::default().fg(SUCCESS)),
        Span::raw(" page  "),
        Span::styled("o", Style::default().fg(SUCCESS)),
        Span::raw(" open in browser"),
    ]))
    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(DIM)));
    f.render_widget(help, area);
}
