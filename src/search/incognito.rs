use anyhow::Result;
use std::process::Command;
use urlencoding::encode;
use super::prompt;

pub async fn run() -> Result<()> {
    let query = prompt("Search Brave (Incognito)");
    if query.is_empty() {
        return Ok(());
    }
    
    let search_url = format!(
        "https://search.brave.com/search?q={}",
        encode(&query)
    );
    
    // Open URL in incognito via command line
    // Use --new-window to force opening in existing incognito session
    Command::new("brave-browser-beta")
        .args(["--incognito", "--new-window", &search_url])
        .spawn()?;
    
    focus_brave();
    
    Ok(())
}

/* ───────── helpers ───────── */
fn focus_brave() {
    let _ = Command::new("i3-msg")
        .arg(r#"[class="Brave-browser"] focus"#)
        .output();
}
