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
                    // Use shell to properly detach and handle multiple calls
                    Command::new("sh")
                        .arg("-c")
                        .arg(format!("brave-browser-beta --incognito '{}' >/dev/null 2>&1 &", url))
                        .spawn()?;
                } else {
                    // Normal mode - use surfraw
                    Command::new("surfraw")
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
