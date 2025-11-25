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
        Layout::vertical([Constraint::Length(4), Constraint::Min(0), Constraint::Length(3)])
            .split(area)
    } else {
        Layout::vertical([Constraint::Length(4), Constraint::Min(0)])
            .split(area)
    };

    render_logo(f, chunks[0]);
    render_feeds_list(f, app, chunks[1]);
    if app.show_tooltips {
        render_help(f, app, chunks[2]);
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

    if app.show_all && app.filter.is_empty() {
        let selected = app.feed_index == 0;
        let style = if selected {
            Style::default().fg(SECONDARY).bg(SELECTED_BG)
        } else {
            Style::default().fg(SECONDARY)
        };
        items.push(ListItem::new("  ★ All").style(style));
    }

    let filtered_sources = app.get_filtered_sources();
    for (display_idx, (_, source)) in filtered_sources.iter().enumerate() {
        let idx = if app.show_all && app.filter.is_empty() { display_idx + 1 } else { display_idx };
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
        .title(Span::styled(" Feeds ", Style::default().fg(SECONDARY)))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if is_focused { PRIMARY } else { DIM }));

    let feeds_list = List::new(items)
        .block(feeds_block)
        .highlight_style(Style::default());
    f.render_stateful_widget(feeds_list, area, &mut app.feed_list_state.clone());
}

fn render_help(f: &mut Frame, app: &App, area: Rect) {
    if !app.show_tooltips {
        return;
    }
    let help = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("a", Style::default().fg(SUCCESS)),
            Span::raw(" add "),
            Span::styled("d", Style::default().fg(SUCCESS)),
            Span::raw(" del "),
            Span::styled("r", Style::default().fg(SUCCESS)),
            Span::raw(" refresh"),
        ]),
    ])
    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(DIM)));
    f.render_widget(help, area);
}
