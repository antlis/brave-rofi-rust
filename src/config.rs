use anyhow::{anyhow, Result};
use std::env;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct BrowserConfig {
    pub name: String,
    pub executable: String,
    pub history_path: String,
    pub cdp_port: u16,
    /// Extra CDP ports (other profiles, e.g. web apps) whose tabs are listed too.
    pub extra_cdp_ports: Vec<u16>,
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
            extra_cdp_ports: Self::extra_cdp_ports(),
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
            extra_cdp_ports: Self::extra_cdp_ports(),
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
            extra_cdp_ports: Self::extra_cdp_ports(),
            post_switch_hook: Self::post_switch_hook(),
        }
    }

    /// `BROWSER_EXTRA_CDP_PORTS`: comma-separated ports or ranges, e.g. "9223-9239,9300".
    fn extra_cdp_ports() -> Vec<u16> {
        parse_ports(&env::var("BROWSER_EXTRA_CDP_PORTS").unwrap_or_default())
    }

    /// Primary port first, then the extras.
    pub fn cdp_ports(&self) -> Vec<u16> {
        let mut ports = vec![self.cdp_port];
        ports.extend(self.extra_cdp_ports.iter().filter(|p| **p != self.cdp_port));
        ports
    }

    fn post_switch_hook() -> Option<String> {
        env::var("BROWSER_POST_SWITCH_HOOK")
            .ok()
            .filter(|hook| !hook.trim().is_empty())
    }

    /// Runs the hook with `BROWSER_CDP_PORT` set to the port of the switched-to tab,
    /// so it can raise the right window when several profiles are open.
    pub fn run_post_switch_hook(&self, port: u16) -> Result<()> {
        if let Some(hook) = &self.post_switch_hook {
            let status = Command::new("sh")
                .args(["-c", hook])
                .env("BROWSER_CDP_PORT", port.to_string())
                .status()?;
            if !status.success() {
                return Err(anyhow!("post switch hook failed: {}", hook));
            }
        }

        Ok(())
    }
}

fn parse_ports(spec: &str) -> Vec<u16> {
    spec.split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .flat_map(|part| match part.split_once('-') {
            Some((from, to)) => match (from.trim().parse(), to.trim().parse()) {
                (Ok(from), Ok(to)) => (from..=to).collect(),
                _ => Vec::new(),
            },
            None => part.parse().into_iter().collect(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{parse_ports, BrowserConfig};
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
            extra_cdp_ports: Vec::new(),
            post_switch_hook: Some(hook),
        };

        config.run_post_switch_hook(9222).unwrap();

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
            extra_cdp_ports: Vec::new(),
            post_switch_hook: Some("exit 7".to_string()),
        };

        assert!(config.run_post_switch_hook(9222).is_err());
    }

    #[test]
    fn post_switch_hook_receives_port() {
        let marker = std::env::temp_dir().join(format!(
            "brave-rofi-hook-port-test-{}",
            std::process::id()
        ));
        let config = BrowserConfig {
            name: "Test".to_string(),
            executable: "chromium".to_string(),
            history_path: String::new(),
            cdp_port: 9222,
            extra_cdp_ports: Vec::new(),
            post_switch_hook: Some(format!("printf $BROWSER_CDP_PORT > {}", marker.display())),
        };

        config.run_post_switch_hook(9223).unwrap();

        assert_eq!(fs::read_to_string(&marker).unwrap(), "9223");
        fs::remove_file(marker).unwrap();
    }

    #[test]
    fn parses_ports_and_ranges() {
        assert_eq!(parse_ports(""), Vec::<u16>::new());
        assert_eq!(parse_ports("9223-9225, 9300,junk"), vec![9223, 9224, 9225, 9300]);
    }

    #[test]
    fn cdp_ports_puts_primary_first_without_duplicates() {
        let config = BrowserConfig {
            name: "Test".to_string(),
            executable: "chromium".to_string(),
            history_path: String::new(),
            cdp_port: 9222,
            extra_cdp_ports: vec![9222, 9223],
            post_switch_hook: None,
        };

        assert_eq!(config.cdp_ports(), vec![9222, 9223]);
    }
}
