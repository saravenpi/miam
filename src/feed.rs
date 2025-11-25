use anyhow::{Context, Result};
use atom_syndication::Feed as AtomFeed;
use chrono::{DateTime, Utc};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct FeedSource {
    pub name: String,
    pub url: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct FeedItem {
    pub title: String,
    pub link: Option<String>,
    pub date: DateTime<Utc>,
    pub source_name: String,
}

fn create_client() -> Result<Client> {
    Client::builder()
        .user_agent("miam/0.1.0 (RSS Reader)")
        .build()
        .context("Failed to create HTTP client")
}

fn normalize_url(url: &str) -> String {
    if let Some(id) = url
        .strip_prefix("https://rss.app/feed/")
        .or_else(|| url.strip_prefix("http://rss.app/feed/"))
    {
        let id = id.trim_end_matches(".xml");
        return format!("https://rss.app/feeds/{}.xml", id);
    }

    if url.contains("youtube.com/channel/") {
        if let Some(channel_id) = url.split("/channel/").nth(1) {
            let channel_id = channel_id.split('/').next().unwrap_or(channel_id);
            let channel_id = channel_id.split('?').next().unwrap_or(channel_id);
            return format!(
                "https://www.youtube.com/feeds/videos.xml?channel_id={}",
                channel_id
            );
        }
    }

    if url.contains("youtube.com/@") {
        if let Some(handle) = url.split("/@").nth(1) {
            let handle = handle.split('/').next().unwrap_or(handle);
            let handle = handle.split('?').next().unwrap_or(handle);
            if let Ok(channel_id) = fetch_channel_id_from_handle(handle) {
                return format!(
                    "https://www.youtube.com/feeds/videos.xml?channel_id={}",
                    channel_id
                );
            }
        }
    }

    url.to_string()
}

fn fetch_channel_id_from_handle(handle: &str) -> Result<String> {
    let client = create_client()?;
    let url = format!("https://www.youtube.com/@{}", handle);
    let response = client.get(&url).send()?.text()?;

    if let Some(start) = response.find("\"externalId\":\"") {
        let start = start + 14;
        if let Some(end) = response[start..].find('"') {
            return Ok(response[start..start + end].to_string());
        }
    }

    if let Some(start) = response.find("\"channelId\":\"") {
        let start = start + 13;
        if let Some(end) = response[start..].find('"') {
            return Ok(response[start..start + end].to_string());
        }
    }

    anyhow::bail!("Could not find channel ID for handle: {}", handle)
}

pub fn fetch_feed(url: &str) -> Result<Vec<FeedItem>> {
    let client = create_client()?;
    let normalized_url = normalize_url(url);
    let content = client.get(&normalized_url).send()?.bytes()?;

    if let Ok(items) = parse_rss(&content) {
        return Ok(items);
    }

    if let Ok(items) = parse_atom(&content) {
        return Ok(items);
    }

    anyhow::bail!("Failed to parse feed as RSS or Atom")
}

fn parse_rss(content: &[u8]) -> Result<Vec<FeedItem>> {
    let channel = rss::Channel::read_from(content)?;
    let source_name = channel.title().to_string();

    let items: Vec<FeedItem> = channel
        .items()
        .iter()
        .map(|item| {
            let date = item
                .pub_date()
                .and_then(|d| DateTime::parse_from_rfc2822(d).ok())
                .map(|d| d.with_timezone(&Utc))
                .unwrap_or_else(Utc::now);

            FeedItem {
                title: item.title().unwrap_or("Untitled").to_string(),
                link: item.link().map(String::from),
                date,
                source_name: source_name.clone(),
            }
        })
        .collect();

    Ok(items)
}

fn parse_atom(content: &[u8]) -> Result<Vec<FeedItem>> {
    let feed = AtomFeed::read_from(content)?;
    let source_name = feed.title().to_string();

    let items: Vec<FeedItem> = feed
        .entries()
        .iter()
        .map(|entry| {
            let date = entry
                .updated()
                .with_timezone(&Utc);

            let link = entry
                .links()
                .first()
                .map(|l| l.href().to_string());

            FeedItem {
                title: entry.title().to_string(),
                link,
                date,
                source_name: source_name.clone(),
            }
        })
        .collect();

    Ok(items)
}
