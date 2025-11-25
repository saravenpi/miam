use chrono::{DateTime, Utc};
use ratatui::layout::Rect;

pub fn truncate(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        s.to_string()
    } else {
        format!("{}...", s.chars().take(max_len.saturating_sub(3)).collect::<String>())
    }
}

pub fn feed_icon(url: &str) -> char {
    if url.contains("youtube.com") || url.contains("youtu.be") {
        '\u{f16a}'
    } else {
        '\u{f15c}'
    }
}

pub fn time_ago(date: &DateTime<Utc>) -> String {
    let now = Utc::now();
    let duration = now.signed_duration_since(*date);

    let seconds = duration.num_seconds();
    if seconds < 0 {
        return "just now".to_string();
    }

    let minutes = duration.num_minutes();
    let hours = duration.num_hours();
    let days = duration.num_days();
    let weeks = days / 7;
    let months = days / 30;
    let years = days / 365;

    if seconds < 60 {
        "just now".to_string()
    } else if minutes == 1 {
        "1 min ago".to_string()
    } else if minutes < 60 {
        format!("{} mins ago", minutes)
    } else if hours == 1 {
        "1 hour ago".to_string()
    } else if hours < 24 {
        format!("{} hours ago", hours)
    } else if days == 1 {
        "1 day ago".to_string()
    } else if days < 7 {
        format!("{} days ago", days)
    } else if weeks == 1 {
        "1 week ago".to_string()
    } else if weeks < 4 {
        format!("{} weeks ago", weeks)
    } else if months == 1 {
        "1 month ago".to_string()
    } else if months < 12 {
        format!("{} months ago", months)
    } else if years == 1 {
        "1 year ago".to_string()
    } else {
        format!("{} years ago", years)
    }
}

pub fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width.min(area.width), height.min(area.height))
}
