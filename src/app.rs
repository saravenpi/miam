use crate::config::Config;
use crate::feed::{FeedItem, FeedSource};
use crate::reader::Article;

#[derive(Clone, Copy, PartialEq)]
pub enum Focus {
    Feeds,
    Items,
    Reader,
}

use ratatui::widgets::ListState;

pub struct App {
    pub sources: Vec<FeedSource>,
    pub items: Vec<FeedItem>,
    pub feed_index: usize,
    pub item_index: usize,
    pub focus: Focus,
    pub input: String,
    pub input_mode: bool,
    pub status: String,
    pub show_all: bool,
    pub loading: bool,
    pub spinner_frame: usize,
    pub feed_list_state: ListState,
    pub item_list_state: ListState,
    pub use_invidious: bool,
    pub invidious_instance: String,
    pub pending_g: bool,
    pub background_loading: bool,
    pub current_article: Option<Article>,
    pub article_scroll: u16,
    pub article_loading: bool,
    pub filter_mode: bool,
    pub filter: String,
    pub show_tooltips: bool,
}

impl App {
    pub fn new() -> Self {
        let mut feed_list_state = ListState::default();
        feed_list_state.select(Some(0));
        let mut item_list_state = ListState::default();
        item_list_state.select(Some(0));

        Self {
            sources: Vec::new(),
            items: Vec::new(),
            feed_index: 0,
            item_index: 0,
            focus: Focus::Feeds,
            input: String::new(),
            input_mode: false,
            status: String::new(),
            show_all: true,
            loading: false,
            spinner_frame: 0,
            feed_list_state,
            item_list_state,
            use_invidious: false,
            invidious_instance: "yewtu.be".to_string(),
            pending_g: false,
            background_loading: false,
            current_article: None,
            article_scroll: 0,
            article_loading: false,
            filter_mode: false,
            filter: String::new(),
            show_tooltips: true,
        }
    }

    pub fn go_to_top(&mut self) {
        match self.focus {
            Focus::Feeds => {
                self.feed_index = 0;
                self.feed_list_state.select(Some(0));
            }
            Focus::Items => {
                self.item_index = 0;
                self.item_list_state.select(Some(0));
            }
            Focus::Reader => {}
        }
    }

    pub fn go_to_bottom(&mut self) {
        match self.focus {
            Focus::Feeds => {
                let len = self.feed_list_len();
                if len > 0 {
                    self.feed_index = len - 1;
                    self.feed_list_state.select(Some(self.feed_index));
                }
            }
            Focus::Items => {
                let len = self.item_list_len();
                if len > 0 {
                    self.item_index = len - 1;
                    self.item_list_state.select(Some(self.item_index));
                }
            }
            Focus::Reader => {}
        }
    }

    pub fn tick_spinner(&mut self) {
        if self.loading || self.background_loading {
            self.spinner_frame = (self.spinner_frame + 1) % 10;
        }
    }

    pub fn spinner_char(&self) -> char {
        const SPINNER: [char; 10] = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
        SPINNER[self.spinner_frame]
    }

    pub fn load_config(&mut self) {
        if let Some(config) = Config::load() {
            self.use_invidious = config.settings.invidious;
            self.invidious_instance = config.get_invidious_instance().to_string();
            self.show_tooltips = config.settings.show_tooltips;
            self.sources = config.sources;
        }
    }

    pub fn save_config(&self) {
        let config = Config {
            sources: self.sources.clone(),
            settings: crate::config::Settings {
                invidious: self.use_invidious,
                invidious_instance: Some(self.invidious_instance.clone()),
                show_tooltips: self.show_tooltips,
            },
        };
        config.save();
    }

    fn feed_list_len(&self) -> usize {
        let filtered_count = if self.filter.is_empty() {
            self.sources.len()
        } else {
            self.get_filtered_sources().len()
        };

        if self.show_all && self.filter.is_empty() {
            filtered_count + 1
        } else {
            filtered_count
        }
    }

    fn item_list_len(&self) -> usize {
        if self.filter.is_empty() {
            self.items.len()
        } else {
            self.get_filtered_items().len()
        }
    }

    pub fn next(&mut self) {
        match self.focus {
            Focus::Feeds => {
                let len = self.feed_list_len();
                if len > 0 {
                    self.feed_index = (self.feed_index + 1) % len;
                    self.feed_list_state.select(Some(self.feed_index));
                }
            }
            Focus::Items => {
                let len = self.item_list_len();
                if len > 0 {
                    self.item_index = (self.item_index + 1) % len;
                    self.item_list_state.select(Some(self.item_index));
                }
            }
            Focus::Reader => {}
        }
    }

    pub fn previous(&mut self) {
        match self.focus {
            Focus::Feeds => {
                let len = self.feed_list_len();
                if len > 0 {
                    self.feed_index = self.feed_index.checked_sub(1).unwrap_or(len - 1);
                    self.feed_list_state.select(Some(self.feed_index));
                }
            }
            Focus::Items => {
                let len = self.item_list_len();
                if len > 0 {
                    self.item_index = self
                        .item_index
                        .checked_sub(1)
                        .unwrap_or(len - 1);
                    self.item_list_state.select(Some(self.item_index));
                }
            }
            Focus::Reader => {}
        }
    }

    pub fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Feeds => Focus::Items,
            Focus::Items => Focus::Feeds,
            Focus::Reader => Focus::Items,
        };
    }

    pub fn start_add_feed(&mut self) {
        self.input_mode = true;
        self.input.clear();
        self.status = "Enter feed URL:".to_string();
    }

    pub fn submit_input(&mut self) -> Option<String> {
        if !self.input.is_empty() {
            let url = self.input.clone();
            self.input_mode = false;
            self.input.clear();
            return Some(url);
        }
        self.input_mode = false;
        self.input.clear();
        None
    }

    pub fn add_feed_source(&mut self, url: String, name: String) {
        let source = FeedSource {
            name: if name.is_empty() {
                url.clone()
            } else {
                name
            },
            url,
        };
        self.sources.push(source);
        self.save_config();
    }

    pub fn cancel_input(&mut self) {
        self.input_mode = false;
        self.input.clear();
        self.status.clear();
    }

    pub fn start_filter(&mut self) {
        self.filter_mode = true;
        self.filter.clear();
        match self.focus {
            Focus::Feeds => self.status = "Filter feeds:".to_string(),
            Focus::Items => self.status = "Filter items:".to_string(),
            Focus::Reader => {}
        }
    }

    pub fn cancel_filter(&mut self) {
        self.filter_mode = false;
        self.filter.clear();
        self.status.clear();
        self.feed_index = 0;
        self.item_index = 0;
        self.feed_list_state.select(Some(0));
        self.item_list_state.select(Some(0));
    }

    pub fn get_filtered_sources(&self) -> Vec<(usize, &FeedSource)> {
        if self.filter.is_empty() {
            self.sources.iter().enumerate().collect()
        } else {
            let filter_lower = self.filter.to_lowercase();
            self.sources
                .iter()
                .enumerate()
                .filter(|(_, s)| s.name.to_lowercase().contains(&filter_lower))
                .collect()
        }
    }

    pub fn get_filtered_items(&self) -> Vec<(usize, &FeedItem)> {
        if self.filter.is_empty() {
            self.items.iter().enumerate().collect()
        } else {
            let filter_lower = self.filter.to_lowercase();
            self.items
                .iter()
                .enumerate()
                .filter(|(_, i)| i.title.to_lowercase().contains(&filter_lower))
                .collect()
        }
    }

    pub fn delete_selected(&mut self) {
        if self.focus == Focus::Feeds {
            if self.show_all && self.feed_index == 0 {
                return;
            }
            let idx = if self.show_all { self.feed_index - 1 } else { self.feed_index };
            if idx < self.sources.len() {
                self.sources.remove(idx);
                let len = self.feed_list_len();
                if self.feed_index >= len && len > 0 {
                    self.feed_index = len - 1;
                }
                self.save_config();
                self.status = "Feed removed".to_string();
            }
        }
    }

    pub fn get_selected_item(&self) -> Option<&FeedItem> {
        let filtered = self.get_filtered_items();
        filtered.get(self.item_index).map(|(_, item)| *item)
    }

    pub fn open_selected(&mut self) {
        if self.focus == Focus::Items {
            if let Some(item) = self.get_selected_item() {
                if let Some(link) = &item.link {
                    let url = if self.use_invidious {
                        self.convert_to_invidious(link)
                    } else {
                        link.clone()
                    };
                    let _ = std::process::Command::new("open").arg(&url).spawn();
                }
            }
        }
    }

    pub fn can_open_in_reader(&self) -> bool {
        if self.focus == Focus::Items {
            if let Some(item) = self.get_selected_item() {
                if let Some(link) = &item.link {
                    return !self.is_youtube_link(link);
                }
            }
        }
        false
    }

    pub fn get_selected_url(&self) -> Option<String> {
        self.get_selected_item().and_then(|item| item.link.clone())
    }

    pub fn show_article(&mut self, article: crate::reader::Article) {
        self.current_article = Some(article);
        self.article_scroll = 0;
        self.focus = Focus::Reader;
        self.article_loading = false;
    }

    pub fn close_reader(&mut self) {
        self.current_article = None;
        self.article_scroll = 0;
        self.focus = Focus::Items;
    }

    pub fn scroll_article_down(&mut self) {
        self.article_scroll = self.article_scroll.saturating_add(1);
    }

    pub fn scroll_article_up(&mut self) {
        self.article_scroll = self.article_scroll.saturating_sub(1);
    }

    pub fn scroll_article_page_down(&mut self, page_size: u16) {
        self.article_scroll = self.article_scroll.saturating_add(page_size);
    }

    pub fn scroll_article_page_up(&mut self, page_size: u16) {
        self.article_scroll = self.article_scroll.saturating_sub(page_size);
    }

    fn is_youtube_link(&self, url: &str) -> bool {
        url.contains("youtube.com") || url.contains("youtu.be")
    }

    fn convert_to_invidious(&self, url: &str) -> String {
        if url.contains("youtube.com/watch?v=") {
            if let Some(video_id) = url.split("v=").nth(1) {
                let video_id = video_id.split('&').next().unwrap_or(video_id);
                return format!("https://{}/watch?v={}", self.invidious_instance, video_id);
            }
        } else if url.contains("youtu.be/") {
            if let Some(video_id) = url.split("youtu.be/").nth(1) {
                let video_id = video_id.split('?').next().unwrap_or(video_id);
                return format!("https://{}/watch?v={}", self.invidious_instance, video_id);
            }
        }
        url.to_string()
    }
}
