use anyhow::Result;
use readability::extractor;
use reqwest::blocking::Client;
use std::io::Cursor;
use url::Url;

pub struct Article {
    pub title: String,
    pub content: String,
}

fn create_client() -> Result<Client> {
    Client::builder()
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36")
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to create HTTP client: {}", e))
}

pub fn fetch_article(url_str: &str) -> Result<Article> {
    let client = create_client()?;
    let response = client.get(url_str).send()?;
    let html = response.text()?;

    let url = Url::parse(url_str)?;
    let mut cursor = Cursor::new(html);
    let product = extractor::extract(&mut cursor, &url)?;

    let content = html_to_text(&product.content);

    Ok(Article {
        title: product.title,
        content,
    })
}

fn html_to_text(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    let mut last_was_space = false;
    let mut chars = html.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '<' => {
                in_tag = true;
                let tag_start: String = chars.clone().take(10).collect();
                if tag_start.starts_with("br")
                    || tag_start.starts_with("p")
                    || tag_start.starts_with("/p")
                    || tag_start.starts_with("div")
                    || tag_start.starts_with("/div")
                    || tag_start.starts_with("li")
                    || tag_start.starts_with("/li")
                    || tag_start.starts_with("h1")
                    || tag_start.starts_with("h2")
                    || tag_start.starts_with("h3")
                    || tag_start.starts_with("/h")
                {
                    if !result.ends_with('\n') && !result.is_empty() {
                        result.push('\n');
                    }
                    last_was_space = true;
                }
            }
            '>' => {
                in_tag = false;
            }
            '&' if !in_tag => {
                let entity: String = chars.clone().take(10).take_while(|&c| c != ';').collect();
                let replacement = match entity.as_str() {
                    "nbsp" => " ",
                    "amp" => "&",
                    "lt" => "<",
                    "gt" => ">",
                    "quot" => "\"",
                    "apos" => "'",
                    "#39" => "'",
                    "#x27" => "'",
                    "mdash" => "—",
                    "ndash" => "–",
                    "hellip" => "...",
                    _ => "",
                };
                if !replacement.is_empty() {
                    for _ in 0..=entity.len() {
                        chars.next();
                    }
                    result.push_str(replacement);
                    last_was_space = replacement == " ";
                } else {
                    result.push('&');
                }
            }
            _ if !in_tag => {
                if c.is_whitespace() {
                    if !last_was_space && !result.is_empty() {
                        result.push(' ');
                        last_was_space = true;
                    }
                } else {
                    result.push(c);
                    last_was_space = false;
                }
            }
            _ => {}
        }
    }

    result
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
}
