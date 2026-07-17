use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::error::SeleniumBaseError;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    pub browser: String,
    pub headless: bool,
    pub timeout_seconds: u64,
    pub screenshot_dir: String,
    pub proxy: Option<String>,
    pub window_width: u32,
    pub window_height: u32,
    pub user_data_dir: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            browser: "chrome".to_string(),
            headless: false,
            timeout_seconds: 30,
            screenshot_dir: "screenshots".to_string(),
            proxy: None,
            window_width: 1920,
            window_height: 1080,
            user_data_dir: None,
        }
    }
}

impl Settings {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, SeleniumBaseError> {
        let content = fs::read_to_string(path)
            .map_err(|e| SeleniumBaseError::InvalidConfig(format!("failed to read settings: {e}")))?;
        serde_json::from_str(&content)
            .map_err(|e| SeleniumBaseError::InvalidConfig(format!("failed to parse settings: {e}")))
    }

    pub fn from_env() -> Result<Self, SeleniumBaseError> {
        let settings = Self::default();
        Self::apply_env_overrides(settings)
    }

    pub fn load<P: AsRef<Path>>(path: Option<P>) -> Result<Self, SeleniumBaseError> {
        let settings = match path {
            Some(p) => Self::from_file(p)?,
            None => Self::default(),
        };
        Self::apply_env_overrides(settings)
    }

    fn apply_env_overrides(mut settings: Self) -> Result<Self, SeleniumBaseError> {
        if let Ok(v) = std::env::var("SB_BROWSER") {
            settings.browser = v;
        }
        if let Ok(v) = std::env::var("SB_HEADLESS") {
            settings.headless = parse_bool(&v)?;
        }
        if let Ok(v) = std::env::var("SB_TIMEOUT") {
            settings.timeout_seconds = v
                .parse()
                .map_err(|e| SeleniumBaseError::InvalidConfig(format!("SB_TIMEOUT: {e}")))?;
        }
        if let Ok(v) = std::env::var("SB_SCREENSHOT_DIR") {
            settings.screenshot_dir = v;
        }
        if let Ok(v) = std::env::var("SB_PROXY") {
            settings.proxy = Some(v);
        }
        if let Ok(v) = std::env::var("SB_WINDOW_WIDTH") {
            settings.window_width = v
                .parse()
                .map_err(|e| SeleniumBaseError::InvalidConfig(format!("SB_WINDOW_WIDTH: {e}")))?;
        }
        if let Ok(v) = std::env::var("SB_WINDOW_HEIGHT") {
            settings.window_height = v
                .parse()
                .map_err(|e| SeleniumBaseError::InvalidConfig(format!("SB_WINDOW_HEIGHT: {e}")))?;
        }
        if let Ok(v) = std::env::var("SB_USER_DATA_DIR") {
            settings.user_data_dir = Some(v);
        }
        Ok(settings)
    }
}

fn parse_bool(value: &str) -> Result<bool, SeleniumBaseError> {
    match value.to_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Ok(true),
        "false" | "0" | "no" | "off" => Ok(false),
        _ => Err(SeleniumBaseError::InvalidConfig(format!("cannot parse bool: {value}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn default_settings_are_sensible() {
        let s = Settings::default();
        assert_eq!(s.browser, "chrome");
        assert!(!s.headless);
        assert_eq!(s.timeout_seconds, 30);
        assert_eq!(s.screenshot_dir, "screenshots");
        assert_eq!(s.window_width, 1920);
        assert_eq!(s.window_height, 1080);
        assert!(s.proxy.is_none());
    }

    #[test]
    fn settings_from_file() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        let json = r#"{"browser":"firefox","headless":true,"timeout_seconds":60,"screenshot_dir":"shots","proxy":"http://proxy:8080","window_width":1280,"window_height":720,"user_data_dir":null}"#;
        tmp.write_all(json.as_bytes()).unwrap();
        let s = Settings::from_file(tmp.path()).unwrap();
        assert_eq!(s.browser, "firefox");
        assert!(s.headless);
        assert_eq!(s.timeout_seconds, 60);
        assert_eq!(s.screenshot_dir, "shots");
        assert_eq!(s.proxy, Some("http://proxy:8080".to_string()));
        assert_eq!(s.window_width, 1280);
        assert_eq!(s.window_height, 720);
    }

    #[test]
    fn parse_bool_values() {
        assert!(parse_bool("true").unwrap());
        assert!(parse_bool("YES").unwrap());
        assert!(parse_bool("1").unwrap());
        assert!(!parse_bool("false").unwrap());
        assert!(!parse_bool("OFF").unwrap());
        assert!(parse_bool("not-a-bool").is_err());
    }
}
