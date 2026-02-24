//! Helper functions for client operations.

use crate::types::{Clients, Tx};
use regex::Regex;
use std::sync::OnceLock;
use std::time::SystemTime;

/// Get client name by ID.
pub async fn client_name_by_id(clients: &Clients, id: &str) -> String {
    clients
        .get(id)
        .map(|r| r.value().name.clone())
        .unwrap_or_else(|| id.to_string())
}

/// Get client tx channel by ID.
pub async fn client_tx_by_id(clients: &Clients, id: &str) -> Option<Tx> {
    clients.get(id).map(|r| r.value().tx.clone())
}

/// Get current Unix timestamp.
pub fn now_ts() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub fn censor_profanity(text: &str) -> String {
    static PROFANITY_RE: OnceLock<Regex> = OnceLock::new();
    let re = PROFANITY_RE.get_or_init(|| {
        Regex::new(r"(?i)\b(badword1|badword2|badword3)\b").unwrap() // Placeholder list
    });
    re.replace_all(text, "****").to_string()
}

/// Make a username unique among currently connected clients.
pub async fn make_unique_name(clients: &Clients, desired: &str) -> String {
    let mut candidate = desired.to_string();
    let mut suffix = 1usize;
    loop {
        let collision = clients
            .iter()
            .any(|r| r.value().name.eq_ignore_ascii_case(&candidate));
        if !collision {
            return candidate;
        }
        candidate = format!("{}-{}", desired, suffix);
        suffix += 1;
    }
}

/// Fetch URL preview (OG tags)
pub async fn fetch_preview(url: &str) -> Option<(String, String, String)> {
    static TITLE_SEL: OnceLock<scraper::Selector> = OnceLock::new();
    static DESC_SEL: OnceLock<scraper::Selector> = OnceLock::new();
    static IMAGE_SEL: OnceLock<scraper::Selector> = OnceLock::new();
    static TITLE_TAG: OnceLock<scraper::Selector> = OnceLock::new();

    let resp = reqwest::get(url).await.ok()?;
    let html = resp.text().await.ok()?;

    let document = scraper::Html::parse_document(&html);

    let title_selector =
        TITLE_SEL.get_or_init(|| scraper::Selector::parse("meta[property='og:title']").unwrap());
    let desc_selector = DESC_SEL
        .get_or_init(|| scraper::Selector::parse("meta[property='og:description']").unwrap());
    let image_selector =
        IMAGE_SEL.get_or_init(|| scraper::Selector::parse("meta[property='og:image']").unwrap());
    let title_tag = TITLE_TAG.get_or_init(|| scraper::Selector::parse("title").unwrap());

    let title = document
        .select(title_selector)
        .next()
        .and_then(|e| e.value().attr("content"))
        .map(|s| s.to_string())
        .or_else(|| document.select(title_tag).next().map(|e| e.inner_html()))
        .unwrap_or_default();

    let desc = document
        .select(desc_selector)
        .next()
        .and_then(|e| e.value().attr("content"))
        .map(|s| s.to_string())
        .unwrap_or_default();

    let image = document
        .select(image_selector)
        .next()
        .and_then(|e| e.value().attr("content"))
        .map(|s| s.to_string())
        .unwrap_or_default();

    if title.is_empty() && desc.is_empty() {
        return None;
    }

    Some((title, desc, image))
}
