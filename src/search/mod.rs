pub mod regular;
pub mod incognito;

/// Shared helper for prompting search text
pub fn prompt(query_label: &str) -> String {
    use std::process::Command;

    let out = Command::new("rofi")
        .args(["-dmenu", "-p", query_label])
        .output();

    match out {
        Ok(o) => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        Err(_) => String::new(),
    }
}
