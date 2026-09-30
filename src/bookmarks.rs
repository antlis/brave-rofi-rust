use anyhow::Result;
use std::fs;
use std::process::{Command, Stdio};
use std::io::Write;
use crate::config::BrowserConfig;

pub fn show_bookmarks(incognito: bool, config: &BrowserConfig) -> Result<()> {
    let bookmarks_path = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?
        .join(".config/surfraw/bookmarks");
    
    let content = fs::read_to_string(&bookmarks_path)?;
    let bookmarks: Vec<String> = content
        .lines()
        .filter(|line| !line.is_empty())
        .filter(|line| !line.starts_with('#'))
        .filter(|line| !line.starts_with('/'))
        .filter(|line| line.contains(' '))
        .map(|s| s.to_string())
        .collect();
    
    let mut sorted = bookmarks;
    sorted.sort();
    let menu = sorted.join("\n");
    
    let mut child = Command::new("rofi")
        .args([
            "-dmenu",
            "-i",
            "-p", "bookmarks:",
            "-mesg", ">>> Edit to add new bookmarks at ~/.config/surfraw/bookmarks",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(menu.as_bytes())?;
        stdin.flush()?;
    }
    
    let output = child.wait_with_output()?;
    let selection = String::from_utf8_lossy(&output.stdout).trim().to_string();
    
    if !selection.is_empty() {
        let url = selection
            .split_whitespace()
            .nth(1)
            .unwrap_or("")
            .split(" ;;")
            .next()
            .unwrap_or("")
            .trim()
            .to_string();
        
        if !url.is_empty() {
            let full_url = if url.starts_with("http://") || url.starts_with("https://") {
                url
            } else {
                format!("https://{}", url)
            };

            if incognito {
                Command::new(&config.executable)
                    .arg("--incognito")
                    .arg(&full_url)
                    .spawn()?;
            } else {
                Command::new(&config.executable)
                    .arg(&full_url)
                    .spawn()?;
            }
        }
        
        std::thread::sleep(std::time::Duration::from_millis(500));
        config.run_post_switch_hook(config.cdp_port)?;
    }
    
    Ok(())
}
