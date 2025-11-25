use crate::app::{App, Focus};
use crate::ui::colors::{DIM, PRIMARY, SECONDARY, SELECTED_BG, SUCCESS};
use crate::ui::utils::{feed_icon, time_ago, truncate};
use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let is_loading = app.loading || app.background_loading || app.article_loading;
    let show_status = app.show_tooltips || is_loading;

    let chunks = if show_status {
        Layout::vertical([Constraint::Min(0), Constraint::Length(3)]).split(area)
    } else {
        Layout::vertical([Constraint::Min(0)]).split(area)
    };

    render_feed_items(f, app, chunks[0]);
    if show_status {
        render_status(f, app, chunks[1]);
    }
}

fn render_feed_items(f: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focus == Focus::Items;

    let feed_title = if app.loading {
        format!(" Feed {} ", app.spinner_char())
    } else if app.background_loading {
        format!(" Feed {} updating... ", app.spinner_char())
    } else {
        " Feed ".to_string()
    };

    if app.loading {
        render_loading(f, app, area, feed_title);
    } else if app.items.is_empty() {
        render_empty(f, area, feed_title, is_focused);
    } else {
        render_items_list(f, app, area, feed_title, is_focused);
    }
}

fn render_loading(f: &mut Frame, app: &App, area: Rect, feed_title: String) {
    let loading = Paragraph::new(format!("{} {}", app.spinner_char(), app.status))
        .style(Style::default().fg(PRIMARY))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .title(Span::styled(feed_title, Style::default().fg(PRIMARY)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(PRIMARY)),
        );
    f.render_widget(loading, area);
}

fn render_empty(f: &mut Frame, area: Rect, feed_title: String, is_focused: bool) {
    let empty = Paragraph::new("No items. Add a feed with 'a' or press 'r' to refresh.")
        .style(Style::default().fg(DIM))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .title(Span::styled(feed_title, Style::default().fg(SECONDARY)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(if is_focused { PRIMARY } else { DIM })),
        );
    f.render_widget(empty, area);
}

fn render_items_list(f: &mut Frame, app: &App, area: Rect, feed_title: String, is_focused: bool) {
    let filtered_items = app.get_filtered_items();
    let items: Vec<ListItem> = filtered_items
        .iter()
        .enumerate()
        .map(|(display_idx, (_, item))| {
            let selected = display_idx == app.item_index;
            let date = item.date.format("%m/%d").to_string();
            let relative = time_ago(&item.date);
            let style = if selected {
                Style::default().fg(Color::White).bg(SELECTED_BG)
            } else {
                Style::default().fg(Color::White)
            };

            let icon = item.link.as_ref().map(|l| feed_icon(l)).unwrap_or('\u{f15c}');
            let available_width = area.width.saturating_sub(6) as usize;
            let title = truncate(&item.title, available_width);

            let first_line = Line::from(vec![
                Span::raw("  "),
                Span::styled(format!("{} ", icon), Style::default().fg(SECONDARY)),
                Span::raw(title),
            ]);

            let video_type = if item.is_youtube_short {
                " • Short"
            } else if item.link.as_ref().map(|l| l.contains("youtube.com") || l.contains("youtu.be")).unwrap_or(false) {
                " • Video"
            } else {
                ""
            };

            // Calculate available width for source name
            let fixed_parts = format!("    {} • {}{} • ", date, relative, video_type);
            let fixed_width = fixed_parts.len();
            let available_width = area.width.saturating_sub(6) as usize;
            let source_max_width = available_width.saturating_sub(fixed_width);

            let source_display = if source_max_width > 10 {
                truncate(&item.source_name, source_max_width)
            } else {
                truncate(&item.source_name, 15)
            };

            let metadata = format!("{}{}", fixed_parts, source_display);
            let second_line = Line::from(vec![
                Span::styled(metadata, Style::default().fg(DIM)),
            ]);

            ListItem::new(vec![first_line, second_line])
                .style(style)
        })
        .collect();

    let items_block = Block::default()
        .title(Span::styled(feed_title, Style::default().fg(SECONDARY)))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if is_focused { PRIMARY } else { DIM }));

    let items_list = List::new(items)
        .block(items_block)
        .highlight_style(Style::default());
    f.render_stateful_widget(items_list, area, &mut app.item_list_state.clone());
}

fn render_status(f: &mut Frame, app: &App, area: Rect) {
    let is_loading = app.loading || app.background_loading || app.article_loading;
    let status_text = if is_loading {
        format!("{} {}", app.spinner_char(), app.status)
    } else {
        app.status.clone()
    };

    let content = if app.show_tooltips {
        Line::from(vec![
            Span::styled("Tab", Style::default().fg(SUCCESS)),
            Span::raw(" switch "),
            Span::styled("o", Style::default().fg(SUCCESS)),
            Span::raw(" open "),
            Span::styled("q", Style::default().fg(SUCCESS)),
            Span::raw(" quit"),
            Span::raw("  "),
            Span::styled(status_text, Style::default().fg(if is_loading { PRIMARY } else { DIM })),
        ])
    } else {
        Line::from(vec![
            Span::styled(status_text, Style::default().fg(PRIMARY)),
        ])
    };

    let status = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(DIM)));
    f.render_widget(status, area);
}
