mod app;
mod cache;
mod config;
mod feed;
mod reader;
mod ui;

use anyhow::Result;
use app::App;
use clap::{Parser, Subcommand};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use feed::FeedItem;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::sync::mpsc;
use std::{io, thread, time::Duration};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser)]
#[command(name = "miam")]
#[command(about = "A minimalist RSS feed reader TUI", long_about = None)]
#[command(version = VERSION)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Upgrade,
}

enum LoadResult {
    Items(Vec<FeedItem>),
    BackgroundUpdate(Vec<FeedItem>),
    Article(reader::Article),
    ArticleError(String),
    FeedAdded(String, String),
    FeedAddError(String),
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Upgrade) => {
            return upgrade();
        }
        None => {}
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    app.load_config();

    let (tx, rx) = mpsc::channel::<LoadResult>();

    if !app.sources.is_empty() {
        app.loading = true;
        app.status = "Loading all feeds...".to_string();
        spawn_refresh_all(app.sources.clone(), tx.clone());
    }

    let res = run_app(&mut terminal, &mut app, rx, tx);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {err:?}");
    }

    Ok(())
}

fn upgrade() -> Result<()> {
    use std::process::Command;

    println!("Upgrading miam by rebuilding from source...");
    println!("This will fetch the latest version from GitHub and compile it.");

    let tmp_dir = std::env::temp_dir().join("miam-upgrade");

    if tmp_dir.exists() {
        println!("Cleaning up previous build directory...");
        std::fs::remove_dir_all(&tmp_dir)?;
    }

    println!("Cloning repository...");
    let clone_status = Command::new("git")
        .args(["clone", "--depth", "1", "https://github.com/saravenpi/miam.git"])
        .arg(&tmp_dir)
        .status()?;

    if !clone_status.success() {
        anyhow::bail!("Failed to clone repository");
    }

    println!("Building from source (this may take a few minutes)...");
    let build_status = Command::new("cargo")
        .args(["build", "--release"])
        .current_dir(&tmp_dir)
        .status()?;

    if !build_status.success() {
        anyhow::bail!("Build failed");
    }

    let current_exe = std::env::current_exe()?;
    let new_binary = tmp_dir.join("target/release/miam");

    println!("Installing new binary to {}...", current_exe.display());
    std::fs::copy(&new_binary, &current_exe)?;

    println!("Cleaning up...");
    std::fs::remove_dir_all(&tmp_dir)?;

    println!("Successfully upgraded miam!");
    println!("Restart the application to use the new version.");

    Ok(())
}

fn spawn_refresh_all_cached(sources: Vec<feed::FeedSource>, tx: mpsc::Sender<LoadResult>) {
    let cached = cache::load_all_cached();
    if !cached.is_empty() {
        let _ = tx.send(LoadResult::Items(cached));
    }

    thread::spawn(move || {
        for source in &sources {
            if let Ok(feed_items) = feed::fetch_feed(&source.url) {
                let items_for_source: Vec<_> = feed_items
                    .into_iter()
                    .map(|mut item| {
                        item.source_name = source.name.clone();
                        item
                    })
                    .collect();
                cache::merge_and_save(&source.name, items_for_source);
            }
        }
        let all_items = cache::load_all_cached();
        let _ = tx.send(LoadResult::BackgroundUpdate(all_items));
    });
}

fn spawn_refresh_single_cached(source: feed::FeedSource, tx: mpsc::Sender<LoadResult>) {
    if let Some(cached) = cache::load_cached_items(&source.name) {
        if !cached.is_empty() {
            let _ = tx.send(LoadResult::Items(cached));
        }
    }

    thread::spawn(move || {
        let mut items = Vec::new();
        if let Ok(feed_items) = feed::fetch_feed(&source.url) {
            for mut item in feed_items {
                item.source_name = source.name.clone();
                items.push(item);
            }
        }
        items.sort_by(|a, b| b.date.cmp(&a.date));
        let merged = cache::merge_and_save(&source.name, items);
        let _ = tx.send(LoadResult::BackgroundUpdate(merged));
    });
}

fn spawn_refresh_all(sources: Vec<feed::FeedSource>, tx: mpsc::Sender<LoadResult>) {
    spawn_refresh_all_cached(sources, tx);
}

fn spawn_refresh_single(source: feed::FeedSource, tx: mpsc::Sender<LoadResult>) {
    spawn_refresh_single_cached(source, tx);
}

fn spawn_fetch_article(url: String, paywall_remover: bool, tx: mpsc::Sender<LoadResult>) {
    thread::spawn(move || {
        match reader::fetch_article(&url, paywall_remover) {
            Ok(article) => {
                let _ = tx.send(LoadResult::Article(article));
            }
            Err(e) => {
                let _ = tx.send(LoadResult::ArticleError(e.to_string()));
            }
        }
    });
}

fn spawn_add_feed(url: String, tx: mpsc::Sender<LoadResult>) {
    thread::spawn(move || {
        match feed::fetch_feed(&url) {
            Ok(items) => {
                let name = items
                    .first()
                    .map(|i| i.source_name.clone())
                    .unwrap_or_else(|| "Unknown".to_string());
                let _ = tx.send(LoadResult::FeedAdded(url, name));
            }
            Err(_) => {
                let _ = tx.send(LoadResult::FeedAddError(url));
            }
        }
    });
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    rx: mpsc::Receiver<LoadResult>,
    tx: mpsc::Sender<LoadResult>,
) -> Result<()> {
    loop {
        terminal.draw(|f| ui::render(f, app))?;

        if let Ok(result) = rx.try_recv() {
            match result {
                LoadResult::Items(items) => {
                    app.items = items;
                    app.loading = false;
                    app.background_loading = true;
                    app.item_index = 0;
                    app.item_list_state.select(Some(0));
                    app.status = format!("Loaded {} items (cached)", app.items.len());
                }
                LoadResult::BackgroundUpdate(items) => {
                    let old_count = app.items.len();
                    app.items = items;
                    app.background_loading = false;
                    let new_count = app.items.len();
                    if new_count > old_count {
                        app.status = format!("Updated: {} items (+{})", new_count, new_count - old_count);
                    } else {
                        app.status = format!("Updated: {} items", new_count);
                    }
                }
                LoadResult::Article(article) => {
                    app.show_article(article);
                }
                LoadResult::ArticleError(err) => {
                    app.article_loading = false;
                    app.status = format!("Failed to load article: {}", err);
                }
                LoadResult::FeedAdded(url, name) => {
                    app.add_feed_source(url, name);
                    app.loading = false;
                    app.status = "Feed added! Press 'r' to refresh.".to_string();
                }
                LoadResult::FeedAddError(url) => {
                    app.loading = false;
                    app.status = format!("Failed to fetch feed: {}", url);
                }
            }
        }

        if app.loading || app.background_loading || app.article_loading {
            app.tick_spinner();
        }

        if event::poll(Duration::from_millis(80))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                    return Ok(());
                }

                if app.loading {
                    continue;
                }

                if app.focus == app::Focus::Reader {
                    match key.code {
                        KeyCode::Esc | KeyCode::Char('q') => app.close_reader(),
                        KeyCode::Char('j') | KeyCode::Down => app.scroll_article_down(),
                        KeyCode::Char('k') | KeyCode::Up => app.scroll_article_up(),
                        KeyCode::Char(' ') | KeyCode::PageDown => app.scroll_article_page_down(20),
                        KeyCode::Char('b') | KeyCode::PageUp => app.scroll_article_page_up(20),
                        KeyCode::Char('g') => {
                            app.article_scroll = 0;
                        }
                        KeyCode::Char('G') => {
                            app.article_scroll = u16::MAX;
                        }
                        KeyCode::Char('o') => {
                            if let Some(url) = app.get_selected_url() {
                                let _ = std::process::Command::new("open").arg(&url).spawn();
                            }
                        }
                        _ => {}
                    }
                    continue;
                }

                if app.tag_editor_mode {
                    match key.code {
                        KeyCode::Enter => {
                            if app.tag_input.is_empty() {
                                app.submit_tags();
                            } else {
                                app.add_tag_from_input();
                            }
                        }
                        KeyCode::Esc => app.cancel_tag_editor(),
                        KeyCode::Char(c) => app.tag_input.push(c),
                        KeyCode::Backspace => {
                            if app.tag_input.is_empty() {
                                app.remove_selected_tag();
                            } else {
                                app.tag_input.pop();
                            }
                        }
                        KeyCode::Delete => {
                            app.remove_selected_tag();
                        }
                        KeyCode::Tab => {
                            app.next_tag();
                        }
                        KeyCode::BackTab => {
                            app.previous_tag();
                        }
                        KeyCode::Left | KeyCode::Up => {
                            if app.tag_input.is_empty() {
                                app.previous_tag();
                            }
                        }
                        KeyCode::Right | KeyCode::Down => {
                            if app.tag_input.is_empty() {
                                app.next_tag();
                            }
                        }
                        _ => {}
                    }
                } else if app.filter_mode {
                    match key.code {
                        KeyCode::Enter | KeyCode::Esc => app.cancel_filter(),
                        KeyCode::Char(c) => {
                            app.filter.push(c);
                            app.feed_index = 0;
                            app.item_index = 0;
                            app.tag_index = 0;
                            app.feed_list_state.select(Some(0));
                            app.item_list_state.select(Some(0));
                            app.tag_list_state.select(Some(0));
                        }
                        KeyCode::Backspace => {
                            app.filter.pop();
                            app.feed_index = 0;
                            app.item_index = 0;
                            app.tag_index = 0;
                            app.feed_list_state.select(Some(0));
                            app.item_list_state.select(Some(0));
                            app.tag_list_state.select(Some(0));
                        }
                        _ => {}
                    }
                } else if app.input_mode {
                    match key.code {
                        KeyCode::Enter => {
                            if let Some(url) = app.submit_input() {
                                app.loading = true;
                                app.status = "Adding feed...".to_string();
                                spawn_add_feed(url, tx.clone());
                            }
                        }
                        KeyCode::Esc => app.cancel_input(),
                        KeyCode::Char(c) => app.input.push(c),
                        KeyCode::Backspace => {
                            app.input.pop();
                        }
                        _ => {}
                    }
                } else {
                    if app.pending_g {
                        app.pending_g = false;
                        if key.code == KeyCode::Char('g') {
                            app.go_to_top();
                        }
                        continue;
                    }

                    match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Char('/') => {
                            if app.focus == app::Focus::Feeds
                                || app.focus == app::Focus::Items
                                || app.focus == app::Focus::Tags
                            {
                                app.start_filter();
                            }
                        }
                        KeyCode::Char('j') | KeyCode::Down => app.next(),
                        KeyCode::Char('k') | KeyCode::Up => app.previous(),
                        KeyCode::Char('g') => app.pending_g = true,
                        KeyCode::Char('G') => app.go_to_bottom(),
                        KeyCode::Tab => app.toggle_focus(),
                        KeyCode::BackTab => app.toggle_focus(),
                        KeyCode::Enter => {
                            if app.focus == app::Focus::Feeds && !app.sources.is_empty() {
                                app.loading = true;
                                if app.show_all && app.feed_index == 0 {
                                    app.status = "Loading all feeds...".to_string();
                                    spawn_refresh_all(app.sources.clone(), tx.clone());
                                } else {
                                    let idx = if app.show_all { app.feed_index - 1 } else { app.feed_index };
                                    let source = app.sources[idx].clone();
                                    app.status = format!("Loading {}...", source.name);
                                    spawn_refresh_single(source, tx.clone());
                                }
                                app.focus = app::Focus::Items;
                            } else if app.focus == app::Focus::Tags {
                                app.select_tag();
                            } else if app.focus == app::Focus::Items {
                                app.open_selected();
                            }
                        }
                        KeyCode::Char('t') => {
                            if app.focus == app::Focus::Feeds {
                                app.start_tag_editor();
                            }
                        }
                        KeyCode::Char('a') => app.start_add_feed(),
                        KeyCode::Char('d') => app.delete_selected(),
                        KeyCode::Char('r') => {
                            if !app.sources.is_empty() {
                                app.loading = true;
                                app.status = "Refreshing all feeds...".to_string();
                                spawn_refresh_all(app.sources.clone(), tx.clone());
                            }
                        }
                        KeyCode::Char('o') => {
                            if app.focus == app::Focus::Items && app.can_open_in_reader() {
                                if let Some(url) = app.get_selected_url() {
                                    app.article_loading = true;
                                    app.status = "Loading article...".to_string();
                                    spawn_fetch_article(url, app.paywall_remover, tx.clone());
                                }
                            } else {
                                app.open_selected();
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}
