//! Fingerprint / anti-detection profile configuration.
//!
//! A [`Fingerprint`] captures the browser personality dimensions exposed by
//! Multilogin-like profile APIs: navigator, screen, timezone, geolocation,
//! WebGL, media devices, fonts, proxy, and masking flags.
//!
//! Use [`Fingerprint::builder()`] to construct a profile, then pass it to
//! [`StealthOptions`](crate::stealth::options::StealthOptions) or launch a
//! [`PlaywrightSession`](crate::browser::playwright::PlaywrightSession) with it.
//!
//! # Example
//!
//! ```rust
//! use seleniumbase_rs::stealth::fingerprint::{Fingerprint, OsType};
//!
//! let fp = Fingerprint::builder()
//!     .os_type(OsType::Windows)
//!     .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 …")
//!     .screen(1920, 1080)
//!     .hardware_concurrency(8)
//!     .build();
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Browser persona requested by a profile.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum BrowserType {
    /// Chromium-based mimic persona (default).
    #[default]
    Mimic,
    /// Firefox-oriented persona (conceptual; WebDriver mode uses Chromium).
    Stealthfox,
}

/// Operating-system persona.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum OsType {
    #[default]
    Windows,
    Macos,
    Linux,
    Android,
}

impl OsType {
    /// Returns the `navigator.platform` value that matches the OS persona.
    pub fn platform(&self) -> &'static str {
        match self {
            OsType::Windows => "Win32",
            OsType::Macos => "MacIntel",
            OsType::Linux => "Linux x86_64",
            OsType::Android => "Linux armv8l",
        }
    }

    /// Common default screen size for the OS persona.
    pub fn default_screen(&self) -> (u32, u32) {
        match self {
            OsType::Windows => (1920, 1080),
            OsType::Macos => (1920, 1080),
            OsType::Linux => (1920, 1080),
            OsType::Android => (412, 732),
        }
    }
}

/// Masking mode for a fingerprint dimension.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum MaskingMode {
    /// Use the browser's natural value.
    #[default]
    Natural,
    /// Mask / randomize / generic value.
    Mask,
    /// Use a custom value supplied in the fingerprint.
    Custom,
    /// Disable the feature entirely.
    Disabled,
}

/// Noise strategy for graphics/canvas.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum NoiseMode {
    /// Add deterministic noise.
    #[default]
    Mask,
    /// Use natural rendering.
    Natural,
}

/// Canvas-specific noise mode.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum CanvasNoiseMode {
    #[default]
    Mask,
    Natural,
    Disabled,
}

/// Geolocation permission popup behaviour.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum PopupMode {
    #[default]
    Prompt,
    Allow,
    Block,
}

/// Proxy masking mode.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ProxyMaskingMode {
    #[default]
    Disabled,
    Custom,
}

/// QUIC mode.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum QuicMode {
    #[default]
    Natural,
    Disabled,
}

/// Startup behaviour for a profile session.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum StartupBehavior {
    /// Restore previous session tabs.
    #[default]
    Recover,
    /// Open custom start URLs.
    Custom,
}

/// Masking flags controlling which dimensions are spoofed.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StealthFlags {
    pub webrtc_masking: MaskingMode,
    pub audio_masking: MaskingMode,
    pub graphics_noise: NoiseMode,
    pub geolocation_popup: PopupMode,
    pub navigator_masking: MaskingMode,
    pub localization_masking: MaskingMode,
    pub timezone_masking: MaskingMode,
    pub graphics_masking: MaskingMode,
    pub fonts_masking: MaskingMode,
    pub media_devices_masking: MaskingMode,
    pub screen_masking: MaskingMode,
    pub geolocation_masking: MaskingMode,
    pub ports_masking: NoiseMode,
    pub proxy_masking: ProxyMaskingMode,
    pub quic_mode: QuicMode,
    pub canvas_noise: CanvasNoiseMode,
    pub startup_behavior: StartupBehavior,
}

impl StealthFlags {
    /// Sensible defaults that mask most signals except media devices and audio.
    pub fn balanced() -> Self {
        Self {
            webrtc_masking: MaskingMode::Mask,
            audio_masking: MaskingMode::Natural,
            graphics_noise: NoiseMode::Mask,
            geolocation_popup: PopupMode::Prompt,
            navigator_masking: MaskingMode::Mask,
            localization_masking: MaskingMode::Mask,
            timezone_masking: MaskingMode::Mask,
            graphics_masking: MaskingMode::Mask,
            fonts_masking: MaskingMode::Mask,
            media_devices_masking: MaskingMode::Natural,
            screen_masking: MaskingMode::Mask,
            geolocation_masking: MaskingMode::Mask,
            ports_masking: NoiseMode::Mask,
            proxy_masking: ProxyMaskingMode::Disabled,
            quic_mode: QuicMode::Natural,
            canvas_noise: CanvasNoiseMode::Mask,
            startup_behavior: StartupBehavior::Recover,
        }
    }

    /// Every maskable dimension set to `Custom`; useful when every value is
    /// supplied explicitly by the caller.
    pub fn all_custom() -> Self {
        Self {
            webrtc_masking: MaskingMode::Custom,
            audio_masking: MaskingMode::Custom,
            graphics_noise: NoiseMode::Mask,
            geolocation_popup: PopupMode::Prompt,
            navigator_masking: MaskingMode::Custom,
            localization_masking: MaskingMode::Custom,
            timezone_masking: MaskingMode::Custom,
            graphics_masking: MaskingMode::Custom,
            fonts_masking: MaskingMode::Custom,
            media_devices_masking: MaskingMode::Custom,
            screen_masking: MaskingMode::Custom,
            geolocation_masking: MaskingMode::Custom,
            ports_masking: NoiseMode::Mask,
            proxy_masking: ProxyMaskingMode::Custom,
            quic_mode: QuicMode::Natural,
            canvas_noise: CanvasNoiseMode::Mask,
            startup_behavior: StartupBehavior::Custom,
        }
    }
}

/// WebRTC IP-handling policy.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum WebRtcPolicy {
    #[default]
    DisableNonProxiedUdp,
    PublicAndPrivateInterfaces,
    PublicInterfaceOnly,
}

/// Proxy configuration for a profile.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub r#type: String,
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    pub save_traffic: bool,
}

impl ProxyConfig {
    /// Returns the proxy URL used for Chrome command-line flags.
    pub fn to_url(&self) -> String {
        let auth = match (&self.username, &self.password) {
            (Some(u), Some(p)) => format!("{u}:{p}@"),
            _ => String::new(),
        };
        format!("{}://{}{}:{}", self.r#type, auth, self.host, self.port)
    }
}

/// Complete browser fingerprint / anti-detection profile.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Fingerprint {
    pub browser_type: BrowserType,
    pub os_type: OsType,
    pub core_version: Option<u32>,

    // Navigator
    pub user_agent: Option<String>,
    pub platform: Option<String>,
    pub hardware_concurrency: Option<u32>,
    pub device_memory: Option<f64>,
    pub max_touch_points: Option<u32>,

    // Localization
    pub locale: Option<String>,
    pub languages: Option<String>,
    pub accept_languages: Option<String>,

    // Timezone
    pub timezone: Option<String>,

    // Screen
    pub screen_width: Option<u32>,
    pub screen_height: Option<u32>,
    pub pixel_ratio: Option<f64>,
    pub color_depth: Option<u32>,

    // WebGL
    pub webgl_vendor: Option<String>,
    pub webgl_renderer: Option<String>,

    // Media devices
    pub audio_inputs: Option<u32>,
    pub audio_outputs: Option<u32>,
    pub video_inputs: Option<u32>,

    // Geolocation
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub altitude: Option<f64>,
    pub accuracy: Option<f64>,

    // Fonts
    pub fonts: Vec<String>,

    // Proxy
    pub proxy: Option<ProxyConfig>,

    // WebRTC
    pub webrtc_policy: WebRtcPolicy,

    // Storage
    pub local_storage: bool,
    pub save_service_worker: bool,

    // Start URLs
    pub custom_start_urls: Vec<String>,

    // Command-line params injected into the browser.
    pub cmd_params: HashMap<String, String>,

    // Masking flags
    pub flags: StealthFlags,
}

impl Fingerprint {
    /// Returns a builder for fluent construction.
    pub fn builder() -> FingerprintBuilder {
        FingerprintBuilder::default()
    }

    /// Quick preset for a Windows desktop Chrome profile.
    pub fn windows_desktop() -> Self {
        Self::builder()
            .os_type(OsType::Windows)
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36")
            .platform("Win32")
            .screen(1920, 1080)
            .hardware_concurrency(8)
            .device_memory(8.0)
            .locale("en-US")
            .languages("en-US,en;q=0.9")
            .timezone("America/New_York")
            .webgl("Google Inc. (NVIDIA)", "ANGLE (NVIDIA, NVIDIA GeForce RTX 4070 Ti Direct3D11 vs_5_0 ps_5_0, D3D11)")
            .flags(StealthFlags::balanced())
            .build()
    }

    /// Quick preset for a macOS desktop Chrome profile.
    pub fn macos_desktop() -> Self {
        Self::builder()
            .os_type(OsType::Macos)
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36")
            .platform("MacIntel")
            .screen(1920, 1080)
            .hardware_concurrency(8)
            .device_memory(8.0)
            .locale("en-US")
            .languages("en-US,en;q=0.9")
            .timezone("America/Los_Angeles")
            .webgl("Apple Inc.", "Apple M3")
            .flags(StealthFlags::balanced())
            .build()
    }

    /// Quick preset for an Android mobile Chrome profile.
    pub fn android_mobile() -> Self {
        Self::builder()
            .os_type(OsType::Android)
            .user_agent("Mozilla/5.0 (Linux; Android 14; SM-S918B) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Mobile Safari/537.36")
            .platform("Linux armv8l")
            .screen(412, 915)
            .pixel_ratio(3.0)
            .hardware_concurrency(8)
            .max_touch_points(5)
            .locale("en-US")
            .languages("en-US,en;q=0.9")
            .timezone("America/New_York")
            .webgl("Qualcomm", "Adreno (TM) 740")
            .flags(StealthFlags::balanced())
            .build()
    }
}

/// Fluent builder for [`Fingerprint`].
#[derive(Clone, Debug, Default)]
pub struct FingerprintBuilder {
    inner: Fingerprint,
}

impl FingerprintBuilder {
    pub fn browser_type(mut self, v: BrowserType) -> Self {
        self.inner.browser_type = v;
        self
    }

    pub fn os_type(mut self, v: OsType) -> Self {
        self.inner.os_type = v;
        self
    }

    pub fn core_version(mut self, v: u32) -> Self {
        self.inner.core_version = Some(v);
        self
    }

    pub fn user_agent(mut self, v: impl Into<String>) -> Self {
        self.inner.user_agent = Some(v.into());
        self
    }

    pub fn platform(mut self, v: impl Into<String>) -> Self {
        self.inner.platform = Some(v.into());
        self
    }

    pub fn hardware_concurrency(mut self, v: u32) -> Self {
        self.inner.hardware_concurrency = Some(v);
        self
    }

    pub fn device_memory(mut self, v: f64) -> Self {
        self.inner.device_memory = Some(v);
        self
    }

    pub fn max_touch_points(mut self, v: u32) -> Self {
        self.inner.max_touch_points = Some(v);
        self
    }

    pub fn locale(mut self, v: impl Into<String>) -> Self {
        self.inner.locale = Some(v.into());
        self
    }

    pub fn languages(mut self, v: impl Into<String>) -> Self {
        self.inner.languages = Some(v.into());
        self
    }

    pub fn accept_languages(mut self, v: impl Into<String>) -> Self {
        self.inner.accept_languages = Some(v.into());
        self
    }

    pub fn timezone(mut self, v: impl Into<String>) -> Self {
        self.inner.timezone = Some(v.into());
        self
    }

    pub fn screen(mut self, width: u32, height: u32) -> Self {
        self.inner.screen_width = Some(width);
        self.inner.screen_height = Some(height);
        self
    }

    pub fn pixel_ratio(mut self, v: f64) -> Self {
        self.inner.pixel_ratio = Some(v);
        self
    }

    pub fn color_depth(mut self, v: u32) -> Self {
        self.inner.color_depth = Some(v);
        self
    }

    pub fn webgl(mut self, vendor: impl Into<String>, renderer: impl Into<String>) -> Self {
        self.inner.webgl_vendor = Some(vendor.into());
        self.inner.webgl_renderer = Some(renderer.into());
        self
    }

    pub fn media_devices(mut self, audio_in: u32, audio_out: u32, video_in: u32) -> Self {
        self.inner.audio_inputs = Some(audio_in);
        self.inner.audio_outputs = Some(audio_out);
        self.inner.video_inputs = Some(video_in);
        self
    }

    pub fn geolocation(mut self, latitude: f64, longitude: f64) -> Self {
        self.inner.latitude = Some(latitude);
        self.inner.longitude = Some(longitude);
        self
    }

    pub fn altitude(mut self, v: f64) -> Self {
        self.inner.altitude = Some(v);
        self
    }

    pub fn accuracy(mut self, v: f64) -> Self {
        self.inner.accuracy = Some(v);
        self
    }

    pub fn fonts(mut self, v: Vec<String>) -> Self {
        self.inner.fonts = v;
        self
    }

    pub fn proxy(mut self, v: ProxyConfig) -> Self {
        self.inner.proxy = Some(v);
        self
    }

    pub fn webrtc_policy(mut self, v: WebRtcPolicy) -> Self {
        self.inner.webrtc_policy = v;
        self
    }

    pub fn local_storage(mut self, v: bool) -> Self {
        self.inner.local_storage = v;
        self
    }

    pub fn save_service_worker(mut self, v: bool) -> Self {
        self.inner.save_service_worker = v;
        self
    }

    pub fn custom_start_urls(mut self, v: Vec<String>) -> Self {
        self.inner.custom_start_urls = v;
        self
    }

    pub fn cmd_params(mut self, v: HashMap<String, String>) -> Self {
        self.inner.cmd_params = v;
        self
    }

    pub fn flags(mut self, v: StealthFlags) -> Self {
        self.inner.flags = v;
        self
    }

    pub fn build(self) -> Fingerprint {
        self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_builds_profile() {
        let fp = Fingerprint::builder()
            .os_type(OsType::Windows)
            .user_agent("UA")
            .screen(1920, 1080)
            .hardware_concurrency(8)
            .build();

        assert_eq!(fp.os_type, OsType::Windows);
        assert_eq!(fp.user_agent.as_deref(), Some("UA"));
        assert_eq!(fp.screen_width, Some(1920));
        assert_eq!(fp.hardware_concurrency, Some(8));
    }

    #[test]
    fn presets_are_valid() {
        assert_eq!(Fingerprint::windows_desktop().os_type, OsType::Windows);
        assert_eq!(Fingerprint::macos_desktop().os_type, OsType::Macos);
        assert_eq!(Fingerprint::android_mobile().os_type, OsType::Android);
    }

    #[test]
    fn proxy_url_includes_auth() {
        let proxy = ProxyConfig {
            r#type: "http".to_owned(),
            host: "proxy.example.com".to_owned(),
            port: 8080,
            username: Some("u".to_owned()),
            password: Some("p".to_owned()),
            save_traffic: false,
        };
        assert_eq!(proxy.to_url(), "http://u:p@proxy.example.com:8080");
    }
}
