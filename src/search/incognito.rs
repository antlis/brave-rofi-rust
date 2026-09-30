use anyhow::Result;
use std::process::Command;
use urlencoding::encode;

use super::prompt;
use crate::config::BrowserConfig;

pub async fn run(config: &BrowserConfig) -> Result<()> {
    let query = prompt(&format!("Search {} (Incognito)", config.name));
    if query.is_empty() {
        return Ok(());
    }
    
    let search_url = format!("https://search.brave.com/search?q={}", encode(&query));
    
    Command::new(&config.executable)
        .arg("--incognito")
        .arg(&search_url)
        .spawn()?;

    config.run_post_switch_hook(config.cdp_port)?;
    
    Ok(())
}
