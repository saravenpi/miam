use crate::feed::FeedItem;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

fn cache_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|p| p.join(".miam"))
}

fn cache_file_path(feed_name: &str) -> Option<PathBuf> {
    let safe_name: String = feed_name
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect();
    cache_dir().map(|p| p.join(format!("{}.yml", safe_name)))
}

pub fn ensure_cache_dir() {
    if let Some(dir) = cache_dir() {
        let _ = fs::create_dir_all(dir);
    }
}

pub fn load_cached_items(feed_name: &str) -> Option<Vec<FeedItem>> {
    let path = cache_file_path(feed_name)?;
    let content = fs::read_to_string(path).ok()?;
    serde_yaml::from_str(&content).ok()
}

pub fn save_cached_items(feed_name: &str, items: &[FeedItem]) {
    if let Some(path) = cache_file_path(feed_name) {
        ensure_cache_dir();
        if let Ok(content) = serde_yaml::to_string(items) {
            let _ = fs::write(path, content);
        }
    }
}

pub fn load_all_cached() -> Vec<FeedItem> {
    let mut all_items = Vec::new();
    if let Some(dir) = cache_dir() {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                if entry.path().extension().map(|e| e == "yml").unwrap_or(false) {
                    if let Ok(content) = fs::read_to_string(entry.path()) {
                        if let Ok(items) = serde_yaml::from_str::<Vec<FeedItem>>(&content) {
                            all_items.extend(items);
                        }
                    }
                }
            }
        }
    }
    all_items.sort_by(|a, b| b.date.cmp(&a.date));
    dedup_items(&mut all_items);
    all_items
}

pub fn merge_and_save(feed_name: &str, new_items: Vec<FeedItem>) -> Vec<FeedItem> {
    let mut all_items = load_cached_items(feed_name).unwrap_or_default();
    all_items.extend(new_items);
    all_items.sort_by(|a, b| b.date.cmp(&a.date));
    dedup_items(&mut all_items);
    save_cached_items(feed_name, &all_items);
    all_items
}

fn dedup_items(items: &mut Vec<FeedItem>) {
    let mut seen: HashMap<String, ()> = HashMap::new();
    items.retain(|item| {
        let key = item.link.clone().unwrap_or_else(|| item.title.clone());
        seen.insert(key, ()).is_none()
    });
}
