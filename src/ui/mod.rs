mod colors;
mod dialogs;
mod feed_list;
mod reader;
mod sidebar;
mod utils;

use crate::app::{App, Focus};
use ratatui::{layout::{Constraint, Layout}, Frame};

pub fn render(f: &mut Frame, app: &App) {
    if app.focus == Focus::Reader {
        reader::render(f, app);
        return;
    }

    let chunks = Layout::horizontal([Constraint::Length(28), Constraint::Min(0)]).split(f.area());

    sidebar::render(f, app, chunks[0]);
    feed_list::render(f, app, chunks[1]);

    if app.input_mode {
        dialogs::render_input_dialog(f, app);
    }

    if app.filter_mode {
        dialogs::render_filter_dialog(f, app);
    }
}
