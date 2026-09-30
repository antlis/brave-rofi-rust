use anyhow::{anyhow, Result};
use std::env;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct BrowserConfig {
    pub name: String,
    pub executable: String,
    pub history_path: String,
    pub cdp_port: u16,
    pub post_switch_hook: Option<String>,
}

impl BrowserConfig {
    pub fn from_env() -> Self {
        let browser = env::var("BROWSER").unwrap_or_else(|_| "brave-beta".to_string());
        
        match browser.as_str() {
            "brave-beta" => Self::brave_beta(browser),
            "brave" => Self::brave(browser),
            "chromium" => Self::chromium(browser),
            _ => {
                eprintln!("Unknown browser '{}', using brave-beta", browser);
                Self::brave_beta("brave-beta".to_string())
            }
        }
    }
    
    fn brave_beta(executable: String) -> Self {
        let home = env::var("HOME").unwrap_or_else(|_| "/home/user".to_string());
        Self {
            name: "Brave Beta".to_string(),
            executable,
            history_path: format!(
                "{}/.config/BraveSoftware/Brave-Browser-Beta/Default/History",
                home
            ),
            cdp_port: 9222,
            post_switch_hook: Self::post_switch_hook(),
        }
    }
    
    fn brave(executable: String) -> Self {
        let home = env::var("HOME").unwrap_or_else(|_| "/home/user".to_string());
        Self {
            name: "Brave".to_string(),
            executable,
            history_path: format!(
                "{}/.config/BraveSoftware/Brave-Browser/Default/History",
                home
            ),
            cdp_port: 9222,
            post_switch_hook: Self::post_switch_hook(),
        }
    }
    
    fn chromium(executable: String) -> Self {
        let home = env::var("HOME").unwrap_or_else(|_| "/home/user".to_string());
        Self {
            name: "Chromium".to_string(),
            executable,
            history_path: format!("{}/.config/chromium/Default/History", home),
            cdp_port: 9222,
            post_switch_hook: Self::post_switch_hook(),
        }
    }

    fn post_switch_hook() -> Option<String> {
        env::var("BROWSER_POST_SWITCH_HOOK")
            .ok()
            .filter(|hook| !hook.trim().is_empty())
    }

    pub fn run_post_switch_hook(&self) -> Result<()> {
        if let Some(hook) = &self.post_switch_hook {
            let status = Command::new("sh").args(["-c", hook]).status()?;
            if !status.success() {
                return Err(anyhow!("post switch hook failed: {}", hook));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::BrowserConfig;
    use std::fs;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    #[test]
    fn chromium_can_be_selected_from_browser_env() {
        let _guard = env_lock().lock().unwrap();
        std::env::set_var("BROWSER", "chromium");
        std::env::remove_var("BROWSER_POST_SWITCH_HOOK");

        let config = BrowserConfig::from_env();

        assert_eq!(config.name, "Chromium");
        assert_eq!(config.executable, "chromium");
        assert!(config.history_path.ends_with("/.config/chromium/Default/History"));
        assert_eq!(config.cdp_port, 9222);
        assert!(config.post_switch_hook.is_none());

        std::env::remove_var("BROWSER");
    }

    #[test]
    fn brave_uses_browser_env_as_executable() {
        let _guard = env_lock().lock().unwrap();
        std::env::set_var("BROWSER", "brave");
        std::env::remove_var("BROWSER_POST_SWITCH_HOOK");

        let config = BrowserConfig::from_env();

        assert_eq!(config.name, "Brave");
        assert_eq!(config.executable, "brave");
        assert!(config
            .history_path
            .ends_with("/.config/BraveSoftware/Brave-Browser/Default/History"));

        std::env::remove_var("BROWSER");
    }

    #[test]
    fn post_switch_hook_runs_configured_command() {
        let _guard = env_lock().lock().unwrap();
        let marker = std::env::temp_dir().join(format!(
            "brave-rofi-hook-test-{}",
            std::process::id()
        ));
        let hook = format!("printf hooked > {}", marker.display());
        let config = BrowserConfig {
            name: "Test".to_string(),
            executable: "chromium".to_string(),
            history_path: String::new(),
            cdp_port: 9222,
            post_switch_hook: Some(hook),
        };

        config.run_post_switch_hook().unwrap();

        assert_eq!(fs::read_to_string(&marker).unwrap(), "hooked");
        fs::remove_file(marker).unwrap();
    }

    #[test]
    fn post_switch_hook_reports_failure() {
        let config = BrowserConfig {
            name: "Test".to_string(),
            executable: "chromium".to_string(),
            history_path: String::new(),
            cdp_port: 9222,
            post_switch_hook: Some("exit 7".to_string()),
        };

        assert!(config.run_post_switch_hook().is_err());
    }
}
