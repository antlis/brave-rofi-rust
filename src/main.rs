mod bookmarks;
mod config;
mod history;
mod search;

use anyhow::{anyhow, Result};
use futures_util::SinkExt;
use serde_json::json;
use std::io::Write;
use std::process::{Command, Stdio};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use url::Url;

use config::BrowserConfig;

#[derive(Debug, Clone)]
struct Tab {
    target_id: String,
    title: String,
    url: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = BrowserConfig::from_env();
    let tabs = get_tabs(&config).await?;
    
    let menu = build_menu(&tabs, &config);
    let selection = show_rofi_menu(&menu, &config)?;
    
    if !selection.is_empty() {
        handle_selection(selection, tabs, &config).await?;
    }

    Ok(())
}

/* ───────────────────────────────────────────── */
/* CDP                                          */
/* ───────────────────────────────────────────── */

async fn get_tabs(config: &BrowserConfig) -> Result<Vec<Tab>> {
    let cdp_url = format!("http://localhost:{}/json", config.cdp_port);
    let targets: serde_json::Value = reqwest_blocking(&cdp_url)?;
    let tabs = targets
        .as_array()
        .ok_or_else(|| anyhow!("Expected array from /json"))?
        .iter()
        .filter(|t| {
            t["type"] == "page"
                && !t["url"]
                    .as_str()
                    .unwrap_or("")
                    .starts_with("chrome-extension://")
        })
        .map(|t| Tab {
            target_id: t["id"].as_str().unwrap_or("").to_string(),
            title: t["title"].as_str().unwrap_or("Untitled").to_string(),
            url: t["url"].as_str().unwrap_or("").to_string(),
        })
        .collect();
    Ok(tabs)
}

/* ───────────────────────────────────────────── */
/* Rofi Menu                                    */
/* ───────────────────────────────────────────── */

fn build_menu(tabs: &[Tab], config: &BrowserConfig) -> String {
    let mut menu = String::new();
    menu.push_str(&format!("Tabs: {}\n", tabs.len()));
    menu.push_str("────\n");
    menu.push_str(&format!("Search ({})\n", config.name));
    menu.push_str("────\n");

    for (i, tab) in tabs.iter().enumerate() {
        menu.push_str(&format!("{}. {} - {}\n", i + 1, tab.title, tab.url));
    }

    menu.push_str("────\n");
    menu.push_str("- Bookmarks\n");
    menu.push_str("- Bookmarks incognito\n");
    menu.push_str("- New Tab\n");
    menu.push_str("- Close Tab\n");
    menu.push_str("- Close ALL Tabs\n");
    menu.push_str("- Search in incognito\n");
    menu.push_str("- History\n");
    menu.push_str("- Exit\n");
    
    menu
}

fn show_rofi_menu(menu: &str, config: &BrowserConfig) -> Result<String> {
    let mut child = Command::new("rofi")
        .args([
            "-dmenu",
            "-i",
            "-p",
            &format!("{} Tabs", config.name),
            "-theme-str",
            "window { fullscreen: true; } mainbox { padding: 2%; }",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(menu.as_bytes())?;
        stdin.flush()?;
        drop(stdin);
    }
    
    let output = child.wait_with_output()?;
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/* ───────────────────────────────────────────── */
/* Actions                                      */
/* ───────────────────────────────────────────── */

async fn handle_selection(sel: String, tabs: Vec<Tab>, config: &BrowserConfig) -> Result<()> {
    if sel.starts_with("Search (") {
        search::regular::run(config).await?;
    } else if sel == "- Bookmarks" {
        tokio::task::spawn_blocking({
            let cfg = config.clone();
            move || bookmarks::show_bookmarks(false, &cfg)
        })
        .await??;
    } else if sel == "- Bookmarks incognito" {
        tokio::task::spawn_blocking({
            let cfg = config.clone();
            move || bookmarks::show_bookmarks(true, &cfg)
        })
        .await??;
    } else if sel == "- History" {
        tokio::task::spawn_blocking({
            let cfg = config.clone();
            move || history::show_history(&cfg)
        })
        .await??;
    } else if sel == "- Search in incognito" {
        search::incognito::run(config).await?;
    } else if sel == "- New Tab" {
        open_tab("about:blank", config).await?;
        tokio::time::sleep(std::time::Duration::from_millis(150)).await;
        config.run_post_switch_hook()?;
    } else if sel == "- Close Tab" {
        let tab_options: Vec<String> = tabs.iter()
            .enumerate()
            .map(|(i, t)| format!("{}. {} - {}", i + 1, t.title, t.url))
            .collect();
        let options = tab_options.join("\n");
        let chosen = rofi_multi_select("Close tabs", &options);
        for line in chosen.lines() {
            if let Some(idx_str) = line.split('.').next() {
                if let Ok(idx) = idx_str.parse::<usize>() {
                    let idx = idx.saturating_sub(1);
                    if let Some(tab) = tabs.get(idx) {
                        let _ = close_tab(&tab.target_id, config).await;
                    }
                }
            }
        }
    } else if sel == "- Close ALL Tabs" {
        let confirm = rofi_confirm("Close ALL tabs?");
        if confirm == "YES" {
            let all_tabs = get_tabs(config).await?;
            for t in all_tabs {
                let _ = close_tab(&t.target_id, config).await;
            }
        }
    } else if sel == "- Exit" {
        std::process::exit(0);
    } else if sel.chars().next().map_or(false, |c| c.is_numeric()) {
        let idx: usize = sel
            .split('.')
            .next()
            .ok_or_else(|| anyhow!("Failed to parse selection"))?
            .parse::<usize>()?;
        let idx = idx.saturating_sub(1);
        if let Some(tab) = tabs.get(idx) {
            activate_tab(&tab.target_id, config).await?;
            config.run_post_switch_hook()?;
        }
    }

    Ok(())
}

/* ───────────────────────────────────────────── */
/* Helpers                                      */
/* ───────────────────────────────────────────── */

fn rofi_confirm(prompt: &str) -> String {
    let mut child = Command::new("rofi")
        .args(["-dmenu", "-p", prompt])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(b"NO\nYES\n");
        let _ = stdin.flush();
    }
    
    let output = child.wait_with_output().unwrap();
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn rofi_multi_select(prompt: &str, options: &str) -> String {
    let mut child = Command::new("rofi")
        .args(["-dmenu", "-multi-select", "-p", prompt])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(options.as_bytes());
        let _ = stdin.flush();
    }
    
    let output = child.wait_with_output().unwrap();
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

async fn open_tab(url: &str, config: &BrowserConfig) -> Result<()> {
    cdp_simple("Target.createTarget", json!({ "url": url }), config).await
}

async fn close_tab(id: &str, config: &BrowserConfig) -> Result<()> {
    cdp_simple("Target.closeTarget", json!({ "targetId": id }), config).await
}

async fn activate_tab(id: &str, config: &BrowserConfig) -> Result<()> {
    cdp_simple("Target.activateTarget", json!({ "targetId": id }), config).await
}

async fn cdp_simple(method: &str, params: serde_json::Value, config: &BrowserConfig) -> Result<()> {
    let cdp_url = format!("http://localhost:{}/json/version", config.cdp_port);
    let version: serde_json::Value = reqwest_blocking(&cdp_url)?;
    let ws_url = version["webSocketDebuggerUrl"]
        .as_str()
        .ok_or_else(|| anyhow!("No debugger URL"))?;
    let (mut ws, _) = connect_async(Url::parse(ws_url)?).await?;
    let msg = json!({ "id": 1, "method": method, "params": params });
    ws.send(Message::Text(msg.to_string())).await?;
    Ok(())
}

fn reqwest_blocking(url: &str) -> Result<serde_json::Value> {
    let out = Command::new("curl").arg("-s").arg(url).output()?;
    Ok(serde_json::from_slice(&out.stdout)?)
}
