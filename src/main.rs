mod bookmarks;

use anyhow::{anyhow, Result};
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use std::process::{Command, Stdio};
use std::io::Write;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use url::Url;

#[derive(Debug, Clone)]
struct Tab {
    target_id: String,
    title: String,
    url: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let tabs = get_tabs().await?;
    
    let menu = build_menu(&tabs);
    let selection = show_rofi_menu(&menu)?;
    
    if !selection.is_empty() {
        handle_selection(selection, tabs).await?;
    }

    Ok(())
}

/* ───────────────────────────────────────────── */
/* CDP                                          */
/* ───────────────────────────────────────────── */

async fn get_tabs() -> Result<Vec<Tab>> {
    let version: serde_json::Value = reqwest_blocking("http://localhost:9222/json/version")?;
    let ws_url = version["webSocketDebuggerUrl"]
        .as_str()
        .ok_or_else(|| anyhow!("No debugger URL"))?;

    let (mut ws, _) = connect_async(Url::parse(ws_url)?).await?;

    // Enable discovery (REQUIRED FOR BRAVE)
    send_cdp(
        &mut ws,
        json!({
            "id": 1,
            "method": "Target.setDiscoverTargets",
            "params": { "discover": true }
        }),
    )
    .await?;

    send_cdp(
        &mut ws,
        json!({
            "id": 2,
            "method": "Target.setAutoAttach",
            "params": { "autoAttach": true, "waitForDebuggerOnStart": false, "flatten": true }
        }),
    )
    .await?;

    send_cdp(
        &mut ws,
        json!({
            "id": 3,
            "method": "Target.getTargets"
        }),
    )
    .await?;

    while let Some(msg) = ws.next().await {
        let msg = msg?;
        if let Ok(txt) = msg.to_text() {
            let v: serde_json::Value = serde_json::from_str(txt)?;
            if let Some(targets) = v["result"]["targetInfos"].as_array() {
                let tabs = targets
                    .iter()
                    .filter(|t| t["type"] == "page"
                        && !t["url"].as_str().unwrap_or("").starts_with("chrome-extension://"))
                    .map(|t| Tab {
                        target_id: t["targetId"].as_str().unwrap().to_string(),
                        title: t["title"].as_str().unwrap_or("Untitled").to_string(),
                        url: t["url"].as_str().unwrap_or("").to_string(),
                    })
                    .collect();
                return Ok(tabs);
            }
        }
    }

    Err(anyhow!("Failed to fetch tabs"))
}

async fn send_cdp(
    ws: &mut tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    msg: serde_json::Value,
) -> Result<()> {
    ws.send(Message::Text(msg.to_string())).await?;
    Ok(())
}

/* ───────────────────────────────────────────── */
/* Rofi Menu                                    */
/* ───────────────────────────────────────────── */

fn build_menu(tabs: &[Tab]) -> String {
    let mut menu = String::new();
    menu.push_str(&format!("Tabs: {}\n", tabs.len()));
    menu.push_str("────\n");
    menu.push_str("Search (Brave)\n");
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

fn show_rofi_menu(menu: &str) -> Result<String> {
    let mut child = Command::new("rofi")
        .args([
            "-dmenu",
            "-i",
            "-p", "Brave Tabs",
            "-theme-str", "window { fullscreen: true; } mainbox { padding: 2%; }"
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

async fn handle_selection(sel: String, tabs: Vec<Tab>) -> Result<()> {
    if sel == "Search (Brave)" {
        let query = rofi_prompt("Search Brave");
        if !query.is_empty() {
            open_tab(&format!(
                "https://search.brave.com/search?q={}",
                urlencoding::encode(&query)
            ))
            .await?;
            find_and_focus_brave_window(&query);
        }
    } else if sel == "- Bookmarks" {
        bookmarks::show_bookmarks(false)?;
    } else if sel == "- Bookmarks incognito" {
        bookmarks::show_bookmarks(true)?;
    } else if sel == "- History" {
        bookmarks::show_history()?;
    } else if sel == "- Search in incognito" {
        bookmarks::search_incognito()?;
    } else if sel == "- New Tab" {
        open_tab("brave://newtab").await?;
        focus_brave();
    } else if sel == "- Close Tab" {
        // Build options from current tabs
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
                        let _ = close_tab(&tab.target_id).await;
                    }
                }
            }
        }
    } else if sel == "- Close ALL Tabs" {
        let confirm = rofi_confirm("Close ALL tabs?");
        if confirm == "YES" {
            // Re-fetch all tabs to close everything
            let all_tabs = get_tabs().await?;
            for t in all_tabs {
                let _ = close_tab(&t.target_id).await;
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
            activate_tab(&tab.target_id).await?;
            find_and_focus_brave_window(&tab.title);
        }
    }

    Ok(())
}

/* ───────────────────────────────────────────── */
/* Helpers                                      */
/* ───────────────────────────────────────────── */

fn rofi_prompt(prompt: &str) -> String {
    let output = Command::new("rofi")
        .args(["-dmenu", "-p", prompt])
        .output();
    
    if let Ok(out) = output {
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    } else {
        String::new()
    }
}

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

fn focus_brave() {
    let _ = Command::new("i3-msg")
        .arg("[class=\"Brave-browser\"] focus")
        .output();
}

fn find_and_focus_brave_window(tab_title: &str) {
    // Get i3 window tree
    if let Ok(output) = Command::new("i3-msg")
        .args(["-t", "get_tree"])
        .output()
    {
        if let Ok(tree) = serde_json::from_slice::<serde_json::Value>(&output.stdout) {
            let windows = find_brave_windows(&tree);
            
            if windows.is_empty() {
                return;
            }
            
            // Try to find window with matching title
            let matching = windows.iter()
                .find(|(_, name)| name.contains(tab_title));
            
            let window_id = matching
                .or_else(|| windows.first())
                .map(|(id, _)| id);
            
            if let Some(id) = window_id {
                let _ = Command::new("i3-msg")
                    .arg(format!("[id=\"{}\"] focus", id))
                    .output();
            }
        }
    }
}

fn find_brave_windows(node: &serde_json::Value) -> Vec<(u64, String)> {
    let mut windows = Vec::new();
    
    if let Some(window) = node["window"].as_u64() {
        if let Some(name) = node["name"].as_str() {
            if name.contains("Brave") {
                windows.push((window, name.to_string()));
            }
        }
    }
    
    if let Some(nodes) = node["nodes"].as_array() {
        for child in nodes {
            windows.extend(find_brave_windows(child));
        }
    }
    
    windows
}

async fn open_tab(url: &str) -> Result<()> {
    cdp_simple("Target.createTarget", json!({ "url": url })).await
}

async fn activate_tab(id: &str) -> Result<()> {
    cdp_simple("Target.activateTarget", json!({ "targetId": id })).await
}

async fn close_tab(id: &str) -> Result<()> {
    cdp_simple("Target.closeTarget", json!({ "targetId": id })).await
}

async fn cdp_simple(method: &str, params: serde_json::Value) -> Result<()> {
    let version: serde_json::Value = reqwest_blocking("http://localhost:9222/json/version")?;
    let ws_url = version["webSocketDebuggerUrl"]
        .as_str()
        .ok_or_else(|| anyhow!("No debugger URL"))?;
    let (mut ws, _) = connect_async(Url::parse(ws_url)?).await?;
    send_cdp(&mut ws, json!({ "id": 1, "method": method, "params": params })).await?;
    Ok(())
}

fn reqwest_blocking(url: &str) -> Result<serde_json::Value> {
    let out = Command::new("curl").arg("-s").arg(url).output()?;
    Ok(serde_json::from_slice(&out.stdout)?)
}
