use anyhow::Result;
use readability::extractor;
use reqwest::blocking::Client;
use std::io::Cursor;
use url::Url;

pub struct Article {
    pub title: String,
    pub content: String,
}

fn create_client(user_agent: Option<&str>) -> Result<Client> {
    let ua = user_agent.unwrap_or("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36");
    Client::builder()
        .user_agent(ua)
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to create HTTP client: {}", e))
}

pub fn fetch_article(url_str: &str, paywall_remover: bool) -> Result<Article> {
    if paywall_remover {
        fetch_with_paywall_bypass(url_str)
    } else {
        fetch_direct(url_str, None)
    }
}

fn fetch_with_paywall_bypass(url_str: &str) -> Result<Article> {
    if let Ok(article) = try_12ft_io(url_str) {
        return Ok(article);
    }

    if let Ok(article) = try_archive_is(url_str) {
        return Ok(article);
    }

    if let Ok(article) = try_googlebot_ua(url_str) {
        return Ok(article);
    }

    fetch_direct(url_str, None)
}

fn try_12ft_io(url_str: &str) -> Result<Article> {
    let proxied_url = format!("https://12ft.io/{}", url_str);
    fetch_direct(&proxied_url, None)
}

fn try_archive_is(url_str: &str) -> Result<Article> {
    let encoded_url = urlencoding::encode(url_str);
    let archive_url = format!("https://archive.is/newest/{}", encoded_url);
    fetch_direct(&archive_url, None)
}

fn try_googlebot_ua(url_str: &str) -> Result<Article> {
    let googlebot_ua = "Mozilla/5.0 (compatible; Googlebot/2.1; +http://www.google.com/bot.html)";
    fetch_direct(url_str, Some(googlebot_ua))
}

fn fetch_direct(url_str: &str, user_agent: Option<&str>) -> Result<Article> {
    let client = create_client(user_agent)?;
    let response = client.get(url_str).send()
        .map_err(|e| anyhow::anyhow!("Failed to fetch article: {}. Try enabling paywall_remover in config.", e))?;

    let status = response.status();
    if !status.is_success() {
        anyhow::bail!(
            "Access denied (HTTP {}). Enable 'paywall_remover: true' in ~/.config/miam/config.yml to bypass restrictions.",
            status.as_u16()
        );
    }

    let html = response.text()?;

    let url = Url::parse(url_str)?;
    let mut cursor = Cursor::new(html);
    let product = extractor::extract(&mut cursor, &url)
        .map_err(|e| anyhow::anyhow!("Failed to parse article: {}. Try enabling paywall_remover in config.", e))?;

    let content = html_to_text(&product.content);

    Ok(Article {
        title: product.title,
        content,
    })
}

fn html_to_text(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    let mut in_script_or_style = false;
    let mut in_code_or_pre = false;
    let mut last_was_space = false;
    let mut chars = html.chars().peekable();
    let mut tag_name = String::new();
    let mut collecting_tag_name = false;

    while let Some(c) = chars.next() {
        match c {
            '<' => {
                in_tag = true;
                collecting_tag_name = true;
                tag_name.clear();

                let tag_preview: String = chars.clone().take(25).collect();
                let tag_lower = tag_preview.to_lowercase();

                if tag_lower.starts_with("script") || tag_lower.starts_with("style") {
                    in_script_or_style = true;
                }

                if tag_lower.starts_with("/script") || tag_lower.starts_with("/style") {
                    in_script_or_style = false;
                }

                if tag_lower.starts_with("code") || tag_lower.starts_with("pre") {
                    in_code_or_pre = true;
                }

                if tag_lower.starts_with("/code") || tag_lower.starts_with("/pre") {
                    in_code_or_pre = false;
                    if !result.ends_with('\n') && !result.is_empty() {
                        result.push('\n');
                    }
                }

                if !in_script_or_style {
                    if tag_preview.starts_with("h1") {
                        if !result.is_empty() && !result.ends_with("\n\n") {
                            result.push_str("\n\n");
                        }
                        result.push_str("# ");
                    } else if tag_preview.starts_with("h2") {
                        if !result.is_empty() && !result.ends_with("\n\n") {
                            result.push_str("\n\n");
                        }
                        result.push_str("## ");
                    } else if tag_preview.starts_with("h3") {
                        if !result.is_empty() && !result.ends_with("\n\n") {
                            result.push_str("\n\n");
                        }
                        result.push_str("### ");
                    } else if tag_preview.starts_with("/h1")
                        || tag_preview.starts_with("/h2")
                        || tag_preview.starts_with("/h3")
                        || tag_preview.starts_with("/h4")
                        || tag_preview.starts_with("/h5")
                        || tag_preview.starts_with("/h6")
                    {
                        if !result.ends_with('\n') {
                            result.push('\n');
                        }
                        last_was_space = true;
                    } else if tag_preview.starts_with("br")
                        || tag_preview.starts_with("p ")
                        || tag_preview.starts_with("p>")
                        || tag_preview.starts_with("/p")
                        || tag_preview.starts_with("div")
                        || tag_preview.starts_with("/div")
                    {
                        if !result.ends_with('\n') && !result.is_empty() {
                            result.push('\n');
                        }
                        last_was_space = true;
                    } else if tag_preview.starts_with("li") {
                        if !result.ends_with('\n') && !result.is_empty() {
                            result.push('\n');
                        }
                        result.push_str("• ");
                        last_was_space = true;
                    } else if tag_preview.starts_with("/li") {
                        if !result.ends_with('\n') {
                            result.push('\n');
                        }
                        last_was_space = true;
                    } else if tag_preview.starts_with("blockquote") {
                        if !result.ends_with('\n') && !result.is_empty() {
                            result.push('\n');
                        }
                        result.push_str("  ");
                    } else if tag_preview.starts_with("/blockquote") {
                        if !result.ends_with('\n') {
                            result.push('\n');
                        }
                        last_was_space = true;
                    } else if tag_preview.starts_with("strong") || tag_preview.starts_with("/strong") || tag_preview.starts_with("b ") || tag_preview.starts_with("b>") || tag_preview.starts_with("/b>") {
                        result.push_str("**");
                    } else if tag_preview.starts_with("em") || tag_preview.starts_with("/em") || tag_preview.starts_with("i ") || tag_preview.starts_with("i>") || tag_preview.starts_with("/i>") {
                        result.push('*');
                    }
                }
            }
            '>' => {
                in_tag = false;
                collecting_tag_name = false;
            }
            ' ' | '\n' | '\t' if collecting_tag_name => {
                collecting_tag_name = false;
            }
            _ if collecting_tag_name => {
                tag_name.push(c);
            }
            '&' if !in_tag && !in_script_or_style => {
                let entity: String = chars.clone().take(15).take_while(|&ch| ch != ';').collect();
                let replacement = match entity.as_str() {
                    "nbsp" => " ",
                    "amp" => "&",
                    "lt" => "<",
                    "gt" => ">",
                    "quot" => "\"",
                    "apos" | "#39" | "#x27" => "'",
                    "mdash" | "#8212" => "\u{2014}",
                    "ndash" | "#8211" => "\u{2013}",
                    "hellip" | "#8230" => "\u{2026}",
                    "ldquo" | "#8220" => "\u{201C}",
                    "rdquo" | "#8221" => "\u{201D}",
                    "lsquo" | "#8216" => "\u{2018}",
                    "rsquo" | "#8217" => "\u{2019}",
                    "bull" | "#8226" => "\u{2022}",
                    "middot" => "\u{00B7}",
                    "copy" => "\u{00A9}",
                    "reg" => "\u{00AE}",
                    "trade" => "\u{2122}",
                    "euro" => "\u{20AC}",
                    "pound" => "\u{00A3}",
                    "yen" => "\u{00A5}",
                    _ => "",
                };
                if !replacement.is_empty() {
                    for _ in 0..=entity.len() {
                        chars.next();
                    }
                    result.push_str(replacement);
                    last_was_space = replacement == " ";
                } else if entity.starts_with('#') {
                    let num_str = entity.trim_start_matches('#').trim_start_matches("x");
                    if let Ok(code) = if entity.contains('x') {
                        u32::from_str_radix(num_str, 16)
                    } else {
                        num_str.parse()
                    } {
                        if let Some(ch) = char::from_u32(code) {
                            for _ in 0..=entity.len() {
                                chars.next();
                            }
                            result.push(ch);
                            last_was_space = ch.is_whitespace();
                        } else {
                            result.push('&');
                        }
                    } else {
                        result.push('&');
                    }
                } else {
                    result.push('&');
                }
            }
            _ if !in_tag && !in_script_or_style => {
                if in_code_or_pre {
                    result.push(c);
                } else if c.is_whitespace() {
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
