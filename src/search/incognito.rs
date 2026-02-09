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
    
    let search_url = format!(
        "https://search.brave.com/search?q={}",
        encode(&query)
    );
    
    Command::new("sh")
        .arg("-c")
        .arg(format!("{} --incognito '{}' >/dev/null 2>&1 &", config.executable, search_url))
        .spawn()?;
    
    focus_browser(config);
    
    Ok(())
}

fn focus_browser(config: &BrowserConfig) {
    let _ = Command::new("i3-msg")
        .arg(format!("[class=\"{}\"] focus", config.window_class))
        .output();
}
