use anyhow::Result;
use std::fs;
use std::process::{Command, Stdio};
use std::io::Write;
use crate::config::BrowserConfig;

pub fn show_bookmarks(incognito: bool, config: &BrowserConfig) -> Result<()> {
    let bookmarks_path = format!("{}/.config/surfraw/bookmarks", std::env::var("HOME")?);
    
    let content = fs::read_to_string(&bookmarks_path)?;
    let bookmarks: Vec<String> = content
        .lines()
        .filter(|line| !line.is_empty())
        .filter(|line| !line.starts_with('#'))
        .filter(|line| !line.starts_with('/'))
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
            "-color-window", "#000000, #000000, #000000",
            "-color-normal", "#000000, #b3e774, #000000, #b3e774, #000000",
            "-color-active", "#000000, #b3e774, #000000, #b3e774, #000000",
            "-color-urgent", "#000000, #b3e774, #000000, #b3e774, #000000",
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
        let surfraw_output = Command::new("surfraw")
            .arg("-print")
            .arg(&selection)
            .output();
        
        if let Ok(output) = surfraw_output {
            let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
            
            if !url.is_empty() {
                if incognito {
                    Command::new("sh")
                        .arg("-c")
                        .arg(format!("{} --incognito '{}' >/dev/null 2>&1 &", config.executable, url))
                        .spawn()?;
                } else {
                    Command::new("surfraw")
                        .arg(format!("-browser={}", config.executable))
                        .arg(&selection)
                        .spawn()?;
                }
            }
        }
        
        std::thread::sleep(std::time::Duration::from_millis(500));
        let _ = Command::new("i3-msg")
            .arg(format!("[class=\"{}\"] focus", config.window_class))
            .output();
    }
    
    Ok(())
}
