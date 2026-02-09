use anyhow::Result;
use rusqlite::{Connection, OpenFlags};
use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};
use crate::config::BrowserConfig;

pub fn show_history(config: &BrowserConfig) -> Result<()> {
    let tmp_copy = "/tmp/browser_history_rofi";
    fs::copy(&config.history_path, tmp_copy)?;
    
    let conn = Connection::open_with_flags(
        tmp_copy,
        OpenFlags::SQLITE_OPEN_READ_ONLY,
    )?;
    
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
            "-p", &format!("{} History", config.name),
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
    
    if let Some(idx) = selection.find("http") {
        let url = &selection[idx..];
        Command::new(&config.executable)
            .arg(url)
            .spawn()?;
        
        std::thread::sleep(std::time::Duration::from_millis(500));
        let _ = Command::new("i3-msg")
            .arg(format!("[class=\"{}\"] focus", config.window_class))
            .output();
    }
    
    Ok(())
}
