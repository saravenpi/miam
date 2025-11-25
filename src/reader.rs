use anyhow::Result;
use readability::extractor;
use reqwest::blocking::Client;
use std::io::Cursor;
use url::Url;

pub struct Article {
    pub title: String,
    pub content: String,
}

fn create_client(user_agent: Option<&str>, timeout_secs: u64) -> Result<Client> {
    let ua = user_agent.unwrap_or("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36");
    Client::builder()
        .user_agent(ua)
        .timeout(std::time::Duration::from_secs(timeout_secs))
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to create HTTP client: {}", e))
}

pub fn fetch_article(url_str: &str, paywall_remover: bool) -> Result<Article> {
    if paywall_remover {
        fetch_with_paywall_bypass(url_str)
    } else {
        fetch_direct(url_str, None, 15)
    }
}

fn fetch_with_paywall_bypass(url_str: &str) -> Result<Article> {
    // Try direct fetch first (fastest)
    if let Ok(article) = fetch_direct(url_str, None, 10) {
        return Ok(article);
    }

    // Try with Googlebot UA (often bypasses soft paywalls)
    if let Ok(article) = try_googlebot_ua(url_str) {
        return Ok(article);
    }

    // Try 12ft.io as last resort (slower)
    if let Ok(article) = try_12ft_io(url_str) {
        return Ok(article);
    }

    anyhow::bail!("Failed to fetch article after trying all methods")
}

fn try_12ft_io(url_str: &str) -> Result<Article> {
    let proxied_url = format!("https://12ft.io/{}", url_str);
    fetch_direct(&proxied_url, None, 10)
}

fn try_googlebot_ua(url_str: &str) -> Result<Article> {
    let googlebot_ua = "Mozilla/5.0 (compatible; Googlebot/2.1; +http://www.google.com/bot.html)";
    fetch_direct(url_str, Some(googlebot_ua), 10)
}

fn fetch_direct(url_str: &str, user_agent: Option<&str>, timeout_secs: u64) -> Result<Article> {
    let client = create_client(user_agent, timeout_secs)?;
    let response = client.get(url_str).send()
        .map_err(|e| anyhow::anyhow!("Failed to fetch article: {}", e))?;

    let status = response.status();
    if !status.is_success() {
        anyhow::bail!("Access denied (HTTP {})", status.as_u16());
    }

    let html = response.text()?;

    let url = Url::parse(url_str)?;
    let mut cursor = Cursor::new(html);
    let product = extractor::extract(&mut cursor, &url)
        .map_err(|e| anyhow::anyhow!("Failed to parse article: {}", e))?;

    let content = html_to_text(&product.content);

    Ok(Article {
        title: product.title,
        content,
    })
}

fn html_to_text(html: &str) -> String {
    // Simple regex-based approach for better performance
    use regex::Regex;

    // Remove script and style tags with content
    let script_style_re = Regex::new(r"(?is)<(script|style)[^>]*>.*?</\1>").unwrap();
    let mut text = script_style_re.replace_all(html, "").to_string();

    // Add spacing for block elements
    let block_re = Regex::new(r"(?i)</(p|div|h[1-6]|li|blockquote)>").unwrap();
    text = block_re.replace_all(&text, "\n").to_string();

    // Add bullet for list items
    let li_re = Regex::new(r"(?i)<li[^>]*>").unwrap();
    text = li_re.replace_all(&text, "\n• ").to_string();

    // Remove all remaining HTML tags
    let tag_re = Regex::new(r"<[^>]+>").unwrap();
    text = tag_re.replace_all(&text, "").to_string();

    // Decode common HTML entities
    text = text
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&#39;", "'")
        .replace("&mdash;", "\u{2014}")
        .replace("&ndash;", "\u{2013}")
        .replace("&hellip;", "\u{2026}")
        .replace("&ldquo;", "\u{201C}")
        .replace("&rdquo;", "\u{201D}")
        .replace("&lsquo;", "\u{2018}")
        .replace("&rsquo;", "\u{2019}");

    // Collapse multiple newlines and trim
    let whitespace_re = Regex::new(r"\n{3,}").unwrap();
    text = whitespace_re.replace_all(&text, "\n\n").to_string();

    text.trim().to_string()
}
