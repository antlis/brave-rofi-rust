use anyhow::Result;
use urlencoding::encode;

use super::prompt;
use crate::{config::BrowserConfig, open_tab};

pub async fn run(config: &BrowserConfig) -> Result<()> {
    let query = prompt(&format!("Search {}", config.name));
    if query.is_empty() {
        return Ok(());
    }
    let url = format!("https://search.brave.com/search?q={}", encode(&query));
    open_tab(&url, config).await?;
    config.run_post_switch_hook()?;
    Ok(())
}
