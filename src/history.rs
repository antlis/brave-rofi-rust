use anyhow::Result;
use rusqlite::{Connection, OpenFlags};
use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};

pub fn show_history() -> Result<()> {
    let home = std::env::var("HOME")?;
    let history_path = format!(
        "{}/.config/BraveSoftware/Brave-Browser-Beta/Default/History",
        home
    );
    
    // Brave locks DB → copy it
    let tmp_copy = "/tmp/brave_history_rofi";
    fs::copy(&history_path, tmp_copy)?;
    
    let conn = Connection::open_with_flags(
        tmp_copy,
        OpenFlags::SQLITE_OPEN_READ_ONLY,
    )?;
    
    // Approx same logic as: cols = terminal_width / 3
    let cols: usize = 40;
    
    let mut stmt = conn.prepare(
        r#"
        SELECT title, url
        FROM urls
        WHERE title IS NOT NULL AND title != ''
        ORDER BY last_visit_time DESC
        LIMIT 100000
        "#,
    )?;
    
    let rows = stmt.query_map([], |row| {
        let title: String = row.get(0)?;
        let url: String = row.get(1)?;
        let truncated = title.chars().take(cols).collect::<String>();
        Ok(format!("{:<width$}  {}", truncated, url, width = cols))
    })?;
    
    let mut menu = String::new();
    for row in rows {
        menu.push_str(&row?);
        menu.push('\n');
    }
    
    let mut child = Command::new("rofi")
        .args([
            "-dmenu",
            "-i",
            "-p", "Brave History",
            "-theme-str",
            "window { fullscreen: true; } mainbox { padding: 2%; }",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(menu.as_bytes())?;
    }
    
    let output = child.wait_with_output()?;
    let selection = String::from_utf8_lossy(&output.stdout).trim().to_string();
    
    if selection.is_empty() {
        return Ok(());
    }
    
    // Extract URL (same logic as your sed)
    if let Some(idx) = selection.find("http") {
        let url = &selection[idx..];
        Command::new("brave-browser-beta")
            .arg(url)
            .spawn()?;
        
        // Focus brave window
        std::thread::sleep(std::time::Duration::from_millis(500));
        let _ = Command::new("i3-msg")
            .arg("[class=\"Brave-browser\"] focus")
            .output();
    }
    
    Ok(())
}
