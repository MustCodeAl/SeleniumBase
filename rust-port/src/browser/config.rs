use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::error::SeleniumBaseError;
use crate::stealth::fingerprint::Fingerprint;

/// Supported browser types.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum Browser {
    #[default]
    Chrome,
    Chromium,
    Edge,
    Firefox,
}

/// Driver execution mode.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum DriverMode {
    #[default]
    WebDriver,
    Cdp,
    Uc,
}

/// Configuration for a browser session.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct BrowserConfig {
    pub webdriver_url: String,
    pub browser: Browser,
    pub headless: bool,
    pub mode: DriverMode,
    pub user_agent: Option<String>,
    pub locale: Option<String>,
    pub ad_block: bool,
    pub proxy: Option<String>,
    pub proxy_pac_url: Option<String>,
    pub user_data_dir: Option<String>,
    pub extension_dir: Option<String>,
    pub start_page: Option<String>,
    pub reuse_session: bool,
    pub mobile: bool,
    pub threads: Option<usize>,
    pub auto_start_driver: bool,
    /// Extra Chromium/Edge command-line arguments supplied by callers such as
    /// external profile payload integrations.
    #[serde(default)]
    pub extra_args: Vec<String>,
    /// Optional anti-detection fingerprint profile.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fingerprint: Option<Fingerprint>,
    /// Optional explicit path to the browser binary (Chrome/Chromium/Edge).
    /// When `native_spoofing` is enabled and this is unset, the crate will
    /// locate the system Chrome binary, patch a cached copy, and set this
    /// field automatically before launch.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub browser_binary_path: Option<PathBuf>,
}

impl Default for BrowserConfig {
    fn default() -> Self {
        Self::from_runtime_config(&crate::config::RuntimeConfig::default())
    }
}

impl BrowserConfig {
    /// Build a `BrowserConfig` from the current process environment.
    ///
    /// This is the preferred constructor for deployable applications because
    /// it honors `SB_WEBDRIVER_URL`, `SB_CHROME_BIN`, and other runtime
    /// variables without requiring code changes per environment.
    pub fn from_env() -> Result<Self, SeleniumBaseError> {
        let runtime = crate::config::RuntimeConfig::from_env()?;
        Ok(Self::from_runtime_config(&runtime))
    }

    fn from_runtime_config(runtime: &crate::config::RuntimeConfig) -> Self {
        Self {
            webdriver_url: runtime.webdriver_url.clone(),
            browser: Browser::Chrome,
            headless: true,
            mode: DriverMode::WebDriver,
            user_agent: None,
            locale: None,
            ad_block: false,
            proxy: None,
            proxy_pac_url: None,
            user_data_dir: None,
            extension_dir: None,
            start_page: None,
            reuse_session: false,
            mobile: false,
            threads: None,
            auto_start_driver: true,
            extra_args: Vec::new(),
            fingerprint: None,
            browser_binary_path: runtime.chrome_bin.clone(),
        }
    }
}

impl BrowserConfig {
    pub fn with_mode(mut self, mode: DriverMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn is_cdp_enabled(&self) -> bool {
        self.mode == DriverMode::Cdp || self.mode == DriverMode::Uc
    }

    pub fn is_uc_enabled(&self) -> bool {
        self.mode == DriverMode::Uc
    }

    pub fn is_default_webdriver_url(&self) -> bool {
        self.webdriver_url == "http://localhost:4444" || self.webdriver_url.is_empty()
    }

    pub fn with_extra_args(mut self, args: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.extra_args = args.into_iter().map(Into::into).collect();
        self
    }

    pub fn push_extra_arg(mut self, arg: impl Into<String>) -> Self {
        self.extra_args.push(arg.into());
        self
    }

    /// Returns true when the configured fingerprint requests native-level
    /// (binary + CDP) spoofing.
    pub fn native_spoofing_enabled(&self) -> bool {
        self.fingerprint
            .as_ref()
            .is_some_and(|fp| fp.flags.native_spoofing)
    }
}
