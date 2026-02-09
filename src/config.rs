use std::env;

#[derive(Debug, Clone)]
pub struct BrowserConfig {
    pub name: String,
    pub executable: String,
    pub history_path: String,
    pub window_class: String,
    pub cdp_port: u16,
}

impl BrowserConfig {
    pub fn from_env() -> Self {
        let browser = env::var("ROFI_BROWSER").unwrap_or_else(|_| "brave-beta".to_string());
        
        match browser.as_str() {
            "brave-beta" => Self::brave_beta(),
            "brave" => Self::brave(),
            "zen" => Self::zen(),
            "chromium" => Self::chromium(),
            _ => {
                eprintln!("Unknown browser '{}', using brave-beta", browser);
                Self::brave_beta()
            }
        }
    }
    
    fn brave_beta() -> Self {
        let home = env::var("HOME").unwrap_or_else(|_| "/home/user".to_string());
        Self {
            name: "Brave Beta".to_string(),
            executable: "brave-browser-beta".to_string(),
            history_path: format!("{}/.config/BraveSoftware/Brave-Browser-Beta/Default/History", home),
            window_class: "Brave-browser".to_string(),
            cdp_port: 9222,
        }
    }
    
    fn brave() -> Self {
        let home = env::var("HOME").unwrap_or_else(|_| "/home/user".to_string());
        Self {
            name: "Brave".to_string(),
            executable: "brave-browser".to_string(),
            history_path: format!("{}/.config/BraveSoftware/Brave-Browser/Default/History", home),
            window_class: "Brave-browser".to_string(),
            cdp_port: 9222,
        }
    }
    
    fn zen() -> Self {
        let home = env::var("HOME").unwrap_or_else(|_| "/home/user".to_string());
        Self {
            name: "Zen Browser".to_string(),
            executable: "zen-browser".to_string(),
            history_path: format!("{}/.zen/default/places.sqlite", home),
            window_class: "zen".to_string(),
            cdp_port: 9222,
        }
    }
    
    fn chromium() -> Self {
        let home = env::var("HOME").unwrap_or_else(|_| "/home/user".to_string());
        Self {
            name: "Chromium".to_string(),
            executable: "chromium".to_string(),
            history_path: format!("{}/.config/chromium/Default/History", home),
            window_class: "Chromium".to_string(),
            cdp_port: 9222,
        }
    }
}
