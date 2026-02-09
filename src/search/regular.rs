use anyhow::Result;
use urlencoding::encode;
use crate::{open_tab, focus_browser, config::BrowserConfig};
use super::prompt;

pub async fn run(config: &BrowserConfig) -> Result<()> {
    let query = prompt(&format!("Search {}", config.name));
    if query.is_empty() {
        return Ok(());
    }
    let url = format!(
        "https://search.brave.com/search?q={}",
        encode(&query)
    );
    open_tab(&url, config).await?;
    focus_browser(config);
    Ok(())
}
