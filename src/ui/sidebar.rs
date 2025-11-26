use crate::app::{App, Focus};
use crate::ui::colors::{DIM, PRIMARY, SECONDARY, SELECTED_BG, SUCCESS};
use crate::ui::utils::{feed_icon, truncate};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let chunks = if app.show_tooltips {
        Layout::vertical([
            Constraint::Length(4),
            Constraint::Percentage(50),
            Constraint::Percentage(50),
            Constraint::Length(3),
        ])
        .split(area)
    } else {
        Layout::vertical([
            Constraint::Length(4),
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(area)
    };

    render_logo(f, chunks[0]);
    render_feeds_list(f, app, chunks[1]);
    render_tags_list(f, app, chunks[2]);
    if app.show_tooltips {
        render_help(f, app, chunks[3]);
    }
}

fn render_logo(f: &mut Frame, area: Rect) {
    let logo = Paragraph::new(vec![
        Line::from(Span::styled("  miam", Style::default().fg(PRIMARY).add_modifier(Modifier::BOLD))),
        Line::from(Span::styled("  RSS Reader", Style::default().fg(DIM))),
    ])
    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(DIM)));
    f.render_widget(logo, area);
}

fn render_feeds_list(f: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focus == Focus::Feeds;
    let mut items: Vec<ListItem> = Vec::new();

    let mut offset = 0;

    if app.show_all && app.filter.is_empty() {
        let selected = app.feed_index == offset;
        let style = if selected {
            Style::default().fg(SECONDARY).bg(SELECTED_BG)
        } else {
            Style::default().fg(SECONDARY)
        };
        items.push(ListItem::new("  ★ All").style(style));
        offset += 1;
    }

    if app.filter.is_empty() {
        let selected = app.feed_index == offset;
        let style = if selected {
            Style::default().fg(Color::Red).bg(SELECTED_BG)
        } else {
            Style::default().fg(Color::Red)
        };
        items.push(ListItem::new("  ❤ Liked").style(style));
        offset += 1;

        let selected_articles = app.feed_index == offset;
        let article_style = if selected_articles {
            Style::default().fg(Color::Rgb(100, 149, 237)).bg(SELECTED_BG)
        } else {
            Style::default().fg(Color::Rgb(100, 149, 237))
        };
        items.push(ListItem::new("  \u{f15c} Articles").style(article_style));
        offset += 1;

        let selected_videos = app.feed_index == offset;
        let video_style = if selected_videos {
            Style::default().fg(Color::Rgb(255, 99, 71)).bg(SELECTED_BG)
        } else {
            Style::default().fg(Color::Rgb(255, 99, 71))
        };
        items.push(ListItem::new("  \u{f03d} Videos").style(video_style));
        offset += 1;
    }

    let filtered_sources = app.get_filtered_sources();
    for (display_idx, (_, source)) in filtered_sources.iter().enumerate() {
        let idx = display_idx + offset;
        let selected = idx == app.feed_index;
        let icon = feed_icon(&source.url);
        let style = if selected {
            Style::default().fg(Color::White).bg(SELECTED_BG)
        } else {
            Style::default().fg(Color::White)
        };
        items.push(ListItem::new(format!("  {} {}", icon, truncate(&source.name, 20))).style(style));
    }

    let feeds_block = Block::default()
        .title(Span::styled(" \u{f09e} Feeds ", Style::default().fg(SECONDARY)))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if is_focused { PRIMARY } else { DIM }));

    let mut list_state = app.feed_list_state.clone();

    let feeds_list = List::new(items)
        .block(feeds_block)
        .highlight_style(Style::default());
    f.render_stateful_widget(feeds_list, area, &mut list_state);
}

fn render_tags_list(f: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focus == Focus::Tags;
    let tags = app.get_all_tags();
    let mut items: Vec<ListItem> = Vec::new();

    for (idx, tag) in tags.iter().enumerate() {
        let selected = idx == app.tag_index && is_focused;
        let feed_count = app.get_feeds_by_tag(tag).len();
        let style = if selected {
            Style::default().fg(Color::White).bg(SELECTED_BG)
        } else {
            Style::default().fg(Color::White)
        };
        items.push(ListItem::new(format!("  # {} ({})", truncate(tag, 15), feed_count)).style(style));
    }

    let tags_block = Block::default()
        .title(Span::styled(" \u{f02b} Tags ", Style::default().fg(SECONDARY)))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if is_focused { PRIMARY } else { DIM }));

    let mut list_state = app.tag_list_state.clone();

    let tags_list = List::new(items)
        .block(tags_block)
        .highlight_style(Style::default());
    f.render_stateful_widget(tags_list, area, &mut list_state);
}

fn render_help(f: &mut Frame, app: &App, area: Rect) {
    if !app.show_tooltips {
        return;
    }
    let help = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("a", Style::default().fg(SUCCESS)),
            Span::raw(" add "),
            Span::styled("t", Style::default().fg(SUCCESS)),
            Span::raw(" tag "),
            Span::styled("r", Style::default().fg(SUCCESS)),
            Span::raw(" refresh "),
            Span::styled("l", Style::default().fg(SUCCESS)),
            Span::raw(" like"),
        ]),
    ])
    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(DIM)));
    f.render_widget(help, area);
}
