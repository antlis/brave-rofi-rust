use anyhow::Result;
use std::fs;
use std::process::{Command, Stdio};
use std::io::Write;

pub fn show_bookmarks(incognito: bool) -> Result<()> {
    let bookmarks_path = format!("{}/.config/surfraw/bookmarks", std::env::var("HOME")?);
    
    // Read and process bookmarks file
    let content = fs::read_to_string(&bookmarks_path)?;
    let bookmarks: Vec<String> = content
        .lines()
        .filter(|line| !line.is_empty())           // Remove empty lines
        .filter(|line| !line.starts_with('#'))     // Remove comments
        .filter(|line| !line.starts_with('/'))     // Remove lines starting with /
        .map(|s| s.to_string())
        .collect();
    
    let mut sorted = bookmarks;
    sorted.sort();
    let menu = sorted.join("\n");
    
    // Show rofi menu with custom colors
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
        eprintln!("Selected bookmark: {}", selection);
        eprintln!("Incognito mode: {}", incognito);
        
        // Get URL from surfraw
        let surfraw_output = Command::new("surfraw")
            .arg("-print")
            .arg(&selection)
            .output();
        
        if let Ok(output) = surfraw_output {
            let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
            eprintln!("Surfraw resolved to: {}", url);
            
            if !url.is_empty() {
                if incognito {
                    // Open in incognito using CDP to reuse existing incognito window
                    open_incognito_tab(&url)?;
                } else {
                    // Normal mode - use surfraw
                    let _ = Command::new("surfraw")
                        .arg("-browser=brave-browser-beta")
                        .arg(&selection)
                        .spawn()?;
                }
            }
        }
        
        // Focus brave window
        std::thread::sleep(std::time::Duration::from_millis(500));
        let _ = Command::new("i3-msg")
            .arg("[class=\"Brave-browser\"] focus")
            .output();
    }
    
    Ok(())
}

fn open_incognito_tab(url: &str) -> Result<()> {
    // Try to open via CDP first (reuses existing incognito window)
    let cdp_result = Command::new("curl")
        .arg("-s")
        .arg("http://localhost:9222/json/new")
        .arg("-d")
        .arg(format!("{{\"url\":\"{}\"}}", url))
        .output();
    
    match cdp_result {
        Ok(output) if output.status.success() => {
            eprintln!("Opened in existing Brave session via CDP");
            Ok(())
        }
        _ => {
            // Fallback: Launch new incognito window if CDP fails
            eprintln!("CDP failed, launching new incognito window");
            Command::new("brave-browser-beta")
                .arg("--incognito")
                .arg(url)
                .spawn()?;
            Ok(())
        }
    }
}

pub fn show_history() -> Result<()> {
    // Call the existing script for now
    // TODO: Implement direct history reading from Brave's SQLite database
    Command::new("sh")
        .arg("-c")
        .arg("~/bin/rofi/rofi-brave-beta-history")
        .spawn()?;
    Ok(())
}

pub fn search_incognito() -> Result<()> {
    // Call the existing script for now
    // TODO: Implement direct incognito search
    Command::new("sh")
        .arg("-c")
        .arg("~/bin/rofi/rofi-brave-debug-incognito")
        .spawn()?;
    Ok(())
}
