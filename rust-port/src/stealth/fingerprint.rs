//! Fingerprint / anti-detection profile configuration.
//!
//! A [`Fingerprint`] captures the browser personality dimensions exposed by
//! External anti-detect browser profile APIs: navigator, screen, timezone, geolocation,
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
///
/// Newer dimensions (battery, connection, speech, bluetooth, client hints,
/// native `toString` masking, chrome runtime, headless fixes and tracker
/// blocking) were appended after the original set; they default to
/// [`MaskingMode::Natural`] / `false` so existing callers keep their behaviour.
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

    // --- Extended dimensions (appended for backward compatibility) ---
    /// Mask the Battery Status API (`navigator.getBattery`).
    #[serde(default)]
    pub battery_masking: MaskingMode,
    /// Mask the Network Information API (`navigator.connection`).
    #[serde(default)]
    pub connection_masking: MaskingMode,
    /// Spoof `speechSynthesis.getVoices()`.
    #[serde(default)]
    pub speech_masking: MaskingMode,
    /// Stub the Web Bluetooth API.
    #[serde(default)]
    pub bluetooth_masking: MaskingMode,
    /// Override `navigator.userAgentData` (Client Hints).
    #[serde(default)]
    pub client_hints_masking: MaskingMode,
    /// Mask `Function.prototype.toString` so patched natives look native.
    #[serde(default)]
    pub native_tostring_masking: MaskingMode,
    /// Install `window.chrome` runtime/app/csi/loadTimes stubs.
    #[serde(default)]
    pub chrome_runtime_masking: MaskingMode,
    /// Apply headless-mode fixes (matchMedia, permissions consistency).
    #[serde(default)]
    pub headless_masking: MaskingMode,
    /// Enable humanized input timing helpers.
    #[serde(default)]
    pub humanize: bool,
    /// Emit a tracker/fingerprint-script domain block list.
    #[serde(default)]
    pub block_trackers: bool,
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
            battery_masking: MaskingMode::Mask,
            connection_masking: MaskingMode::Mask,
            speech_masking: MaskingMode::Mask,
            bluetooth_masking: MaskingMode::Mask,
            client_hints_masking: MaskingMode::Mask,
            native_tostring_masking: MaskingMode::Mask,
            chrome_runtime_masking: MaskingMode::Mask,
            headless_masking: MaskingMode::Mask,
            humanize: false,
            block_trackers: false,
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
            battery_masking: MaskingMode::Custom,
            connection_masking: MaskingMode::Custom,
            speech_masking: MaskingMode::Custom,
            bluetooth_masking: MaskingMode::Custom,
            client_hints_masking: MaskingMode::Custom,
            native_tostring_masking: MaskingMode::Custom,
            chrome_runtime_masking: MaskingMode::Custom,
            headless_masking: MaskingMode::Custom,
            humanize: false,
            block_trackers: false,
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

/// Battery Status API profile (`navigator.getBattery()`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BatteryProfile {
    pub charging: bool,
    /// Battery level in the range `0.0..=1.0`.
    pub level: f64,
    /// Seconds until fully charged (`f64::INFINITY` when unknown).
    pub charging_time: f64,
    /// Seconds until empty (`f64::INFINITY` when charging).
    pub discharging_time: f64,
}

impl Default for BatteryProfile {
    fn default() -> Self {
        Self {
            charging: true,
            level: 0.94,
            charging_time: 0.0,
            discharging_time: f64::INFINITY,
        }
    }
}

/// Network Information API profile (`navigator.connection`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConnectionProfile {
    pub effective_type: String,
    pub rtt: u32,
    pub downlink: f64,
    pub save_data: bool,
}

impl Default for ConnectionProfile {
    fn default() -> Self {
        Self {
            effective_type: "4g".to_owned(),
            rtt: 50,
            downlink: 10.0,
            save_data: false,
        }
    }
}

/// A single synthesized speech voice for `speechSynthesis.getVoices()`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpeechVoice {
    pub name: String,
    pub lang: String,
    pub default: bool,
    pub local_service: bool,
    pub voice_uri: String,
}

impl SpeechVoice {
    /// Returns a plausible default set of English voices.
    pub fn default_set() -> Vec<Self> {
        vec![
            Self {
                name: "Microsoft David - English (United States)".to_owned(),
                lang: "en-US".to_owned(),
                default: true,
                local_service: true,
                voice_uri: "Microsoft David - English (United States)".to_owned(),
            },
            Self {
                name: "Microsoft Zira - English (United States)".to_owned(),
                lang: "en-US".to_owned(),
                default: false,
                local_service: true,
                voice_uri: "Microsoft Zira - English (United States)".to_owned(),
            },
            Self {
                name: "Google US English".to_owned(),
                lang: "en-US".to_owned(),
                default: false,
                local_service: false,
                voice_uri: "Google US English".to_owned(),
            },
        ]
    }
}

/// A `(brand, version)` pair used by the Client Hints `brands` array.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrandVersion {
    pub brand: String,
    pub version: String,
}

impl BrandVersion {
    pub fn new(brand: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            brand: brand.into(),
            version: version.into(),
        }
    }
}

/// User-Agent Client Hints (`navigator.userAgentData`).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClientHints {
    pub brands: Vec<BrandVersion>,
    pub full_version_list: Vec<BrandVersion>,
    pub platform: String,
    pub platform_version: String,
    pub architecture: String,
    pub bitness: String,
    pub model: String,
    pub ua_full_version: String,
    pub mobile: bool,
}

/// Humanized input-timing configuration.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HumanizeConfig {
    pub enabled: bool,
    pub min_keystroke_delay_ms: u64,
    pub max_keystroke_delay_ms: u64,
    /// Number of intermediate points sampled along a mouse path.
    pub mouse_steps: u32,
}

impl Default for HumanizeConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            min_keystroke_delay_ms: 40,
            max_keystroke_delay_ms: 180,
            mouse_steps: 24,
        }
    }
}

/// Default list of well-known tracking / fingerprinting script hosts that a
/// network interceptor may block. This is a generic, brand-neutral starter set.
pub fn default_tracker_hosts() -> Vec<String> {
    [
        "doubleclick.net",
        "googlesyndication.com",
        "google-analytics.com",
        "googletagmanager.com",
        "adservice.google.com",
        "facebook.net",
        "connect.facebook.net",
        "fpjs.io",
        "fingerprintjs.com",
        "cdn.fingerprint.com",
        "scorecardresearch.com",
        "hotjar.com",
        "segment.io",
        "amplitude.com",
        "mixpanel.com",
    ]
    .iter()
    .map(|s| (*s).to_owned())
    .collect()
}

/// Coherence report produced by [`Fingerprint::validate`].
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CoherenceReport {
    /// Hard inconsistencies likely to trip detection (e.g. OS/UA mismatch).
    pub errors: Vec<String>,
    /// Softer issues worth reviewing.
    pub warnings: Vec<String>,
}

impl CoherenceReport {
    /// Returns `true` when there are no hard errors.
    pub fn is_coherent(&self) -> bool {
        self.errors.is_empty()
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
    /// `navigator.vendor` (e.g. `Google Inc.`).
    #[serde(default)]
    pub vendor: Option<String>,
    /// `navigator.oscpu` (Firefox persona).
    #[serde(default)]
    pub oscpu: Option<String>,
    /// `navigator.productSub` (usually `20030107` for Chromium).
    #[serde(default)]
    pub product_sub: Option<String>,
    /// `navigator.buildID` (Firefox persona).
    #[serde(default)]
    pub build_id: Option<String>,
    /// `navigator.appVersion` override.
    #[serde(default)]
    pub app_version: Option<String>,

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

    // --- Extended dimensions (appended for backward compatibility) ---
    /// Deterministic noise seed. When `None`, a seed is derived from the
    /// user-agent so repeated sessions with the same persona are stable.
    #[serde(default)]
    pub seed: Option<u64>,
    /// Battery Status API profile.
    #[serde(default)]
    pub battery: Option<BatteryProfile>,
    /// Network Information API profile.
    #[serde(default)]
    pub connection: Option<ConnectionProfile>,
    /// Synthesized speech voices.
    #[serde(default)]
    pub speech_voices: Vec<SpeechVoice>,
    /// User-Agent Client Hints.
    #[serde(default)]
    pub client_hints: Option<ClientHints>,
    /// Humanized input timing configuration.
    #[serde(default)]
    pub humanize: HumanizeConfig,
    /// Custom tracker host block list (empty falls back to
    /// [`default_tracker_hosts`]).
    #[serde(default)]
    pub blocked_trackers: Vec<String>,

    // Masking flags
    pub flags: StealthFlags,
}

impl Fingerprint {
    /// Returns a builder for fluent construction.
    pub fn builder() -> FingerprintBuilder {
        FingerprintBuilder::default()
    }

    /// Returns the deterministic noise seed for this profile.
    ///
    /// Uses the explicit [`seed`](Fingerprint::seed) when set, otherwise
    /// derives a stable value from the user-agent (falling back to a constant
    /// when no user-agent is present) so noise is reproducible per persona.
    pub fn seed_value(&self) -> u64 {
        if let Some(seed) = self.seed {
            return seed;
        }
        match self.user_agent.as_deref() {
            Some(ua) => fnv1a(ua.as_bytes()),
            None => 0x9E37_79B9_7F4A_7C15,
        }
    }

    /// Validates cross-field coherence and returns a [`CoherenceReport`].
    ///
    /// This catches common mismatches such as a macOS user-agent paired with a
    /// `Win32` platform, or a mobile user-agent without touch points, that make
    /// an otherwise convincing profile easy to detect.
    pub fn validate(&self) -> CoherenceReport {
        let mut report = CoherenceReport::default();
        let ua = self.user_agent.as_deref().unwrap_or("");
        let ua_lower = ua.to_lowercase();

        let platform = self
            .platform
            .clone()
            .unwrap_or_else(|| self.os_type.platform().to_owned());

        // OS persona vs navigator.platform.
        let platform_ok = match self.os_type {
            OsType::Windows => platform.starts_with("Win"),
            OsType::Macos => platform.contains("Mac"),
            OsType::Linux => platform.contains("Linux") && !platform.contains("arm"),
            OsType::Android => platform.contains("arm") || platform.contains("Linux"),
        };
        if !platform_ok {
            report.errors.push(format!(
                "navigator.platform '{platform}' does not match OS persona {:?}",
                self.os_type
            ));
        }

        // OS persona vs user-agent token.
        if !ua.is_empty() {
            let ua_os_ok = match self.os_type {
                OsType::Windows => ua_lower.contains("windows"),
                OsType::Macos => ua_lower.contains("mac os"),
                OsType::Linux => ua_lower.contains("linux") || ua_lower.contains("x11"),
                OsType::Android => ua_lower.contains("android"),
            };
            if !ua_os_ok {
                report.errors.push(format!(
                    "user-agent does not contain a token for OS persona {:?}",
                    self.os_type
                ));
            }
        }

        // Mobile user-agents need touch points.
        let is_mobile_ua = ua_lower.contains("mobile") || ua_lower.contains("android");
        if is_mobile_ua && self.max_touch_points.unwrap_or(0) == 0 {
            report
                .warnings
                .push("mobile user-agent but maxTouchPoints is 0".to_owned());
        }
        if !is_mobile_ua && self.max_touch_points.unwrap_or(0) > 0 {
            report
                .warnings
                .push("desktop user-agent but maxTouchPoints is greater than 0".to_owned());
        }

        // Locale vs primary language.
        if let (Some(locale), Some(langs)) = (self.locale.as_deref(), self.languages.as_deref()) {
            let primary = langs.split(',').next().unwrap_or("").trim();
            if !primary.is_empty() && !primary.eq_ignore_ascii_case(locale) {
                report.warnings.push(format!(
                    "locale '{locale}' differs from primary language '{primary}'"
                ));
            }
        }

        // Timezone present but no geolocation (soft).
        if self.timezone.is_some() && self.latitude.is_none() {
            report
                .warnings
                .push("timezone set without geolocation; consider matching coordinates".to_owned());
        }

        // Screen sanity.
        if let (Some(w), Some(h)) = (self.screen_width, self.screen_height) {
            if w == 0 || h == 0 {
                report
                    .errors
                    .push("screen dimensions must be non-zero".to_owned());
            }
        }

        report
    }

    /// Returns the tracker host block list, defaulting to
    /// [`default_tracker_hosts`] when none were supplied.
    pub fn tracker_hosts(&self) -> Vec<String> {
        if self.blocked_trackers.is_empty() {
            default_tracker_hosts()
        } else {
            self.blocked_trackers.clone()
        }
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

    /// Sets the deterministic noise seed.
    pub fn seed(mut self, v: u64) -> Self {
        self.inner.seed = Some(v);
        self
    }

    /// Sets `navigator.vendor`.
    pub fn vendor(mut self, v: impl Into<String>) -> Self {
        self.inner.vendor = Some(v.into());
        self
    }

    /// Sets the Battery Status API profile.
    pub fn battery(mut self, v: BatteryProfile) -> Self {
        self.inner.battery = Some(v);
        self
    }

    /// Sets the Network Information API profile.
    pub fn connection(mut self, v: ConnectionProfile) -> Self {
        self.inner.connection = Some(v);
        self
    }

    /// Sets the synthesized speech voices.
    pub fn speech_voices(mut self, v: Vec<SpeechVoice>) -> Self {
        self.inner.speech_voices = v;
        self
    }

    /// Sets the User-Agent Client Hints.
    pub fn client_hints(mut self, v: ClientHints) -> Self {
        self.inner.client_hints = Some(v);
        self
    }

    /// Sets the humanized input timing configuration.
    pub fn humanize(mut self, v: HumanizeConfig) -> Self {
        self.inner.humanize = v;
        self
    }

    /// Sets a custom tracker host block list.
    pub fn blocked_trackers(mut self, v: Vec<String>) -> Self {
        self.inner.blocked_trackers = v;
        self
    }

    pub fn build(self) -> Fingerprint {
        self.inner
    }
}

/// FNV-1a 64-bit hash used to derive a stable noise seed from the user-agent.
fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in bytes {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
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

    #[test]
    fn presets_are_coherent() {
        for fp in [
            Fingerprint::windows_desktop(),
            Fingerprint::macos_desktop(),
            Fingerprint::android_mobile(),
        ] {
            let report = fp.validate();
            assert!(report.is_coherent(), "preset errors: {:?}", report.errors);
        }
    }

    #[test]
    fn mismatched_platform_and_ua_is_incoherent() {
        // macOS user-agent with a Windows platform string is a classic tell.
        let fp = Fingerprint::builder()
            .os_type(OsType::Macos)
            .user_agent(
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 \
                 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36",
            )
            .platform("Win32")
            .build();
        let report = fp.validate();
        assert!(!report.is_coherent());
        assert!(report.errors.iter().any(|e| e.contains("platform")));
    }

    #[test]
    fn seed_is_deterministic_from_user_agent() {
        let fp = Fingerprint::windows_desktop();
        assert_eq!(fp.seed_value(), fp.seed_value());
        let explicit = Fingerprint::builder().seed(1234).build();
        assert_eq!(explicit.seed_value(), 1234);
    }

    #[test]
    fn tracker_hosts_fall_back_to_defaults() {
        let fp = Fingerprint::windows_desktop();
        assert!(!fp.tracker_hosts().is_empty());
        let custom = Fingerprint::builder()
            .blocked_trackers(vec!["example-metrics.test".to_owned()])
            .build();
        assert_eq!(custom.tracker_hosts(), vec!["example-metrics.test"]);
    }
}
