use serde::{Deserialize, Serialize};

use crate::browser::config::{Browser, BrowserConfig, DriverMode};
use crate::error::SeleniumBaseError;
use crate::stealth::fingerprint as stealth_fp;
use crate::stealth::fingerprint::{
    CanvasNoiseMode, Fingerprint as StealthFingerprint, MaskingMode, NoiseMode, OsType, PopupMode,
    ProxyMaskingMode, QuicMode, StartupBehavior, StealthFlags,
};

/// Top-level external browser profile payload.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ProfileParams {
    pub name: String,
    #[serde(default = "default_browser_type")]
    pub browser_type: String,
    #[serde(default = "default_folder_id")]
    pub folder_id: String,
    #[serde(default = "default_os_type")]
    pub os_type: String,
    #[serde(default)]
    pub core_version: Option<u32>,
    #[serde(default)]
    pub core_minor_version: Option<u32>,
    #[serde(default = "default_auto_update_core")]
    pub auto_update_core: bool,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default = "default_times")]
    pub times: u32,
    #[serde(default)]
    pub notes: String,
    pub parameters: Parameters,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Parameters {
    #[serde(default)]
    pub flags: Flags,
    #[serde(default)]
    pub fingerprint: Fingerprint,
    #[serde(default)]
    pub storage: StorageOptions,
    #[serde(default)]
    pub proxy: Option<ProxyConfig>,
    #[serde(default)]
    pub custom_start_urls: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Flags {
    #[serde(default = "default_webrtc_masking")]
    pub webrtc_masking: String,
    #[serde(default = "default_proxy_masking")]
    pub proxy_masking: String,
    #[serde(default = "default_geolocation_popup")]
    pub geolocation_popup: String,
    #[serde(default = "default_audio_masking")]
    pub audio_masking: String,
    #[serde(default = "default_graphics_noise")]
    pub graphics_noise: String,
    #[serde(default = "default_ports_masking")]
    pub ports_masking: String,
    #[serde(default = "default_navigator_masking")]
    pub navigator_masking: String,
    #[serde(default = "default_localization_masking")]
    pub localization_masking: String,
    #[serde(default = "default_timezone_masking")]
    pub timezone_masking: String,
    #[serde(default = "default_graphics_masking")]
    pub graphics_masking: String,
    #[serde(default = "default_fonts_masking")]
    pub fonts_masking: String,
    #[serde(default = "default_media_devices_masking")]
    pub media_devices_masking: String,
    #[serde(default = "default_screen_masking")]
    pub screen_masking: String,
    #[serde(default = "default_geolocation_masking")]
    pub geolocation_masking: String,
    #[serde(default = "default_quic_mode")]
    pub quic_mode: String,
    #[serde(default)]
    pub canvas_noise: Option<String>,
    #[serde(default = "default_startup_behavior")]
    pub startup_behavior: String,
}

impl Default for Flags {
    fn default() -> Self {
        Self {
            webrtc_masking: default_webrtc_masking(),
            proxy_masking: default_proxy_masking(),
            geolocation_popup: default_geolocation_popup(),
            audio_masking: default_audio_masking(),
            graphics_noise: default_graphics_noise(),
            ports_masking: default_ports_masking(),
            navigator_masking: default_navigator_masking(),
            localization_masking: default_localization_masking(),
            timezone_masking: default_timezone_masking(),
            graphics_masking: default_graphics_masking(),
            fonts_masking: default_fonts_masking(),
            media_devices_masking: default_media_devices_masking(),
            screen_masking: default_screen_masking(),
            geolocation_masking: default_geolocation_masking(),
            quic_mode: default_quic_mode(),
            canvas_noise: None,
            startup_behavior: default_startup_behavior(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StorageOptions {
    #[serde(default = "default_is_local")]
    pub is_local: bool,
    #[serde(default = "default_save_service_worker")]
    pub save_service_worker: bool,
}

impl Default for StorageOptions {
    fn default() -> Self {
        Self {
            is_local: default_is_local(),
            save_service_worker: default_save_service_worker(),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Fingerprint {
    #[serde(default)]
    pub navigator: Option<NavigatorFingerprint>,
    #[serde(default)]
    pub localization: Option<LocalizationFingerprint>,
    #[serde(default)]
    pub timezone: Option<TimezoneFingerprint>,
    #[serde(default)]
    pub graphic: Option<GraphicFingerprint>,
    #[serde(default)]
    pub webrtc: Option<WebrtcFingerprint>,
    #[serde(default)]
    pub media_devices: Option<MediaDevicesFingerprint>,
    #[serde(default)]
    pub screen: Option<ScreenFingerprint>,
    #[serde(default)]
    pub geolocation: Option<GeolocationFingerprint>,
    #[serde(default)]
    pub ports: Vec<u16>,
    #[serde(default)]
    pub fonts: Vec<String>,
    #[serde(default)]
    pub cmd_params: CmdParams,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct NavigatorFingerprint {
    #[serde(default)]
    pub hardware_concurrency: Option<u32>,
    pub user_agent: String,
    pub platform: String,
    #[serde(default)]
    pub os_cpu: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct LocalizationFingerprint {
    pub languages: String,
    pub locale: String,
    pub accept_languages: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct TimezoneFingerprint {
    pub zone: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct GraphicFingerprint {
    pub renderer: String,
    pub vendor: String,
    #[serde(default)]
    pub vendor_id: String,
    #[serde(default)]
    pub renderer_id: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct WebrtcFingerprint {
    pub public_ip: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct MediaDevicesFingerprint {
    #[serde(default)]
    pub audio_outputs: u32,
    #[serde(default)]
    pub audio_inputs: u32,
    #[serde(default)]
    pub video_inputs: u32,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ScreenFingerprint {
    pub width: u32,
    pub height: u32,
    pub pixel_ratio: f64,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct GeolocationFingerprint {
    pub latitude: f64,
    pub longitude: f64,
    #[serde(default)]
    pub accuracy: f64,
    #[serde(default)]
    pub altitude: f64,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct CmdParams {
    #[serde(default)]
    pub params: Vec<CmdParam>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct CmdParam {
    pub flag: String,
    #[serde(default)]
    pub value: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ProxyConfig {
    #[serde(rename = "type")]
    pub proxy_type: String,
    pub host: String,
    pub port: u16,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub password: String,
    #[serde(default)]
    pub save_traffic: bool,
}

fn default_browser_type() -> String {
    "mimic".into()
}
fn default_folder_id() -> String {
    "default".into()
}
fn default_os_type() -> String {
    "windows".into()
}
fn default_auto_update_core() -> bool {
    true
}
fn default_times() -> u32 {
    1
}

fn default_webrtc_masking() -> String {
    "mask".into()
}
fn default_proxy_masking() -> String {
    "disabled".into()
}
fn default_geolocation_popup() -> String {
    "prompt".into()
}
fn default_audio_masking() -> String {
    "natural".into()
}
fn default_graphics_noise() -> String {
    "mask".into()
}
fn default_ports_masking() -> String {
    "mask".into()
}
fn default_navigator_masking() -> String {
    "mask".into()
}
fn default_localization_masking() -> String {
    "mask".into()
}
fn default_timezone_masking() -> String {
    "mask".into()
}
fn default_graphics_masking() -> String {
    "mask".into()
}
fn default_fonts_masking() -> String {
    "mask".into()
}
fn default_media_devices_masking() -> String {
    "natural".into()
}
fn default_screen_masking() -> String {
    "mask".into()
}
fn default_geolocation_masking() -> String {
    "mask".into()
}
fn default_quic_mode() -> String {
    "disabled".into()
}
fn default_startup_behavior() -> String {
    "recover".into()
}

fn default_is_local() -> bool {
    true
}
fn default_save_service_worker() -> bool {
    true
}

fn parse_masking(s: &str) -> MaskingMode {
    match s.to_lowercase().as_str() {
        "natural" => MaskingMode::Natural,
        "custom" => MaskingMode::Custom,
        "disabled" => MaskingMode::Disabled,
        _ => MaskingMode::Mask,
    }
}

fn parse_noise(s: &str) -> NoiseMode {
    match s.to_lowercase().as_str() {
        "natural" => NoiseMode::Natural,
        _ => NoiseMode::Mask,
    }
}

fn parse_canvas_noise(s: &str) -> CanvasNoiseMode {
    match s.to_lowercase().as_str() {
        "natural" => CanvasNoiseMode::Natural,
        "disabled" => CanvasNoiseMode::Disabled,
        _ => CanvasNoiseMode::Mask,
    }
}

fn parse_popup(s: &str) -> PopupMode {
    match s.to_lowercase().as_str() {
        "allow" => PopupMode::Allow,
        "block" => PopupMode::Block,
        _ => PopupMode::Prompt,
    }
}

fn parse_proxy_masking(s: &str) -> ProxyMaskingMode {
    match s.to_lowercase().as_str() {
        "custom" => ProxyMaskingMode::Custom,
        _ => ProxyMaskingMode::Disabled,
    }
}

fn parse_quic(s: &str) -> QuicMode {
    match s.to_lowercase().as_str() {
        "natural" => QuicMode::Natural,
        _ => QuicMode::Disabled,
    }
}

fn parse_startup(s: &str) -> StartupBehavior {
    match s.to_lowercase().as_str() {
        "custom" => StartupBehavior::Custom,
        _ => StartupBehavior::Recover,
    }
}

fn parse_os(s: &str) -> OsType {
    match s.to_lowercase().as_str() {
        "macos" => OsType::Macos,
        "linux" => OsType::Linux,
        "android" => OsType::Android,
        _ => OsType::Windows,
    }
}

impl ProfileParams {
    /// Converts the external profile payload into a [`StealthFingerprint`] that can
    /// be injected into a browser session.
    pub fn to_fingerprint(&self) -> StealthFingerprint {
        let mut builder = StealthFingerprint::builder()
            .os_type(parse_os(&self.os_type))
            .flags(StealthFlags {
                webrtc_masking: parse_masking(&self.parameters.flags.webrtc_masking),
                audio_masking: parse_masking(&self.parameters.flags.audio_masking),
                graphics_noise: parse_noise(&self.parameters.flags.graphics_noise),
                geolocation_popup: parse_popup(&self.parameters.flags.geolocation_popup),
                navigator_masking: parse_masking(&self.parameters.flags.navigator_masking),
                localization_masking: parse_masking(&self.parameters.flags.localization_masking),
                timezone_masking: parse_masking(&self.parameters.flags.timezone_masking),
                graphics_masking: parse_masking(&self.parameters.flags.graphics_masking),
                fonts_masking: parse_masking(&self.parameters.flags.fonts_masking),
                media_devices_masking: parse_masking(&self.parameters.flags.media_devices_masking),
                screen_masking: parse_masking(&self.parameters.flags.screen_masking),
                geolocation_masking: parse_masking(&self.parameters.flags.geolocation_masking),
                ports_masking: parse_noise(&self.parameters.flags.ports_masking),
                proxy_masking: parse_proxy_masking(&self.parameters.flags.proxy_masking),
                quic_mode: parse_quic(&self.parameters.flags.quic_mode),
                canvas_noise: self
                    .parameters
                    .flags
                    .canvas_noise
                    .as_deref()
                    .map(parse_canvas_noise)
                    .unwrap_or_default(),
                startup_behavior: parse_startup(&self.parameters.flags.startup_behavior),
                ..StealthFlags::balanced()
            })
            .local_storage(self.parameters.storage.is_local)
            .save_service_worker(self.parameters.storage.save_service_worker)
            .custom_start_urls(self.parameters.custom_start_urls.clone())
            .fonts(self.parameters.fingerprint.fonts.clone());

        if let Some(nav) = self.parameters.fingerprint.navigator.as_ref() {
            builder = builder
                .user_agent(nav.user_agent.clone())
                .platform(nav.platform.clone())
                .hardware_concurrency(nav.hardware_concurrency.unwrap_or(8));
            if !nav.os_cpu.is_empty() {
                // Kept for completeness; os_cpu is rarely exposed to page JS.
                let _ = &nav.os_cpu;
            }
        }

        if let Some(loc) = self.parameters.fingerprint.localization.as_ref() {
            builder = builder
                .locale(loc.locale.clone())
                .languages(loc.languages.clone())
                .accept_languages(loc.accept_languages.clone());
        }

        if let Some(tz) = self.parameters.fingerprint.timezone.as_ref() {
            builder = builder.timezone(tz.zone.clone());
        }

        if let Some(gpu) = self.parameters.fingerprint.graphic.as_ref() {
            builder = builder.webgl(gpu.vendor.clone(), gpu.renderer.clone());
        }

        if let Some(media) = self.parameters.fingerprint.media_devices.as_ref() {
            builder =
                builder.media_devices(media.audio_inputs, media.audio_outputs, media.video_inputs);
        }

        if let Some(screen) = self.parameters.fingerprint.screen.as_ref() {
            builder = builder
                .screen(screen.width, screen.height)
                .pixel_ratio(screen.pixel_ratio);
        }

        if let Some(geo) = self.parameters.fingerprint.geolocation.as_ref() {
            builder = builder
                .geolocation(geo.latitude, geo.longitude)
                .altitude(geo.altitude)
                .accuracy(geo.accuracy);
        }

        if let Some(proxy) = self.parameters.proxy.as_ref() {
            builder = builder.proxy(stealth_fp::ProxyConfig {
                r#type: proxy.proxy_type.clone(),
                host: proxy.host.clone(),
                port: proxy.port,
                username: Some(proxy.username.clone()).filter(|s| !s.is_empty()),
                password: Some(proxy.password.clone()).filter(|s| !s.is_empty()),
                save_traffic: proxy.save_traffic,
            });
        }

        let mut cmd_params = std::collections::HashMap::new();
        for p in self.parameters.fingerprint.cmd_params.params.iter() {
            cmd_params.insert(p.flag.clone(), p.value.clone());
        }
        builder = builder.cmd_params(cmd_params);

        builder.build()
    }

    /// Translates the external profile payload into a `BrowserConfig` that
    /// `seleniumbase-rs` can launch.
    ///
    /// Not every anti-detect flag has a direct Selenium/Chrome capability
    /// equivalent. The conversion applies the values that do map cleanly:
    /// browser type, user agent, locale, proxy, and extra command-line flags.
    pub fn to_browser_config(&self, container_url: impl Into<String>) -> BrowserConfig {
        let mut config = BrowserConfig {
            webdriver_url: container_url.into(),
            browser: self.browser(),
            headless: false,
            mode: if self.browser_type == "stealthfox" {
                DriverMode::WebDriver
            } else {
                DriverMode::Uc
            },
            user_agent: self.user_agent(),
            locale: self.locale(),
            proxy: self.proxy_string(),
            proxy_pac_url: None,
            user_data_dir: self.user_data_dir(),
            extension_dir: None,
            start_page: self.parameters.custom_start_urls.first().cloned(),
            reuse_session: false,
            mobile: self.os_type == "android",
            threads: None,
            ad_block: self
                .parameters
                .proxy
                .as_ref()
                .map(|p| p.save_traffic)
                .unwrap_or(false),
            auto_start_driver: true,
            extra_args: Vec::new(),
            fingerprint: Some(self.to_fingerprint()),
        };

        for extra in self.extra_args() {
            config.extra_args.push(extra);
        }

        config
    }

    /// Returns the `Browser` variant inferred from `browser_type`.
    pub fn browser(&self) -> Browser {
        match self.browser_type.as_str() {
            "stealthfox" => Browser::Firefox,
            _ => Browser::Chrome,
        }
    }

    /// User agent extracted from custom navigator fingerprint when available.
    pub fn user_agent(&self) -> Option<String> {
        self.parameters
            .fingerprint
            .navigator
            .as_ref()
            .map(|n| n.user_agent.clone())
            .filter(|s| !s.is_empty())
    }

    /// Locale extracted from custom localization fingerprint when available.
    pub fn locale(&self) -> Option<String> {
        self.parameters
            .fingerprint
            .localization
            .as_ref()
            .map(|l| l.locale.clone())
            .filter(|s| !s.is_empty())
    }

    /// Proxy URL built from `parameters.proxy`.
    pub fn proxy_string(&self) -> Option<String> {
        self.parameters.proxy.as_ref().map(|p| {
            if p.username.is_empty() {
                format!("{}://{}:{}", p.proxy_type, p.host, p.port)
            } else {
                format!(
                    "{}://{}:{}@{}:{}",
                    p.proxy_type, p.username, p.password, p.host, p.port
                )
            }
        })
    }

    /// Per-profile persistent data directory when `storage.is_local` is true.
    pub fn user_data_dir(&self) -> Option<String> {
        if self.parameters.storage.is_local {
            Some(format!("./profile-data/{}", self.folder_id))
        } else {
            None
        }
    }

    /// Additional Chromium command-line flags parsed from `cmd_params`.
    pub fn extra_args(&self) -> Vec<String> {
        self.parameters
            .fingerprint
            .cmd_params
            .params
            .iter()
            .map(|p| {
                if p.value.is_empty() {
                    format!("--{}", p.flag)
                } else {
                    format!("--{}={}", p.flag, p.value)
                }
            })
            .collect()
    }

    /// Applies runtime fingerprint overrides to an active `BaseCase`.
    ///
    /// This covers screen size and geolocation, which must be set after the
    /// browser session is alive.
    pub async fn apply_runtime_overrides(
        &self,
        sb: &mut crate::BaseCase,
    ) -> Result<(), SeleniumBaseError> {
        if let Some(screen) = &self.parameters.fingerprint.screen {
            sb.set_window_size(screen.width, screen.height).await?;
        }
        if let Some(geo) = &self.parameters.fingerprint.geolocation {
            sb.set_geolocation(geo.latitude, geo.longitude, geo.accuracy)
                .await?;
        }
        for url in self.parameters.custom_start_urls.iter().skip(1) {
            sb.open(url).await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parse_profile_payload() {
        let raw = json!({
            "name": "Profile_name",
            "browser_type": "mimic",
            "folder_id": "4500dd84-d8c5-4450-b2df-1c64daed8bad",
            "core_version": 124,
            "auto_update_core": false,
            "os_type": "windows",
            "times": 1,
            "notes": "asd",
            "parameters": {
                "flags": {
                    "webrtc_masking": "custom",
                    "startup_behavior": "custom"
                },
                "storage": {
                    "is_local": false,
                    "save_service_worker": false
                },
                "fingerprint": {
                    "navigator": {
                        "hardware_concurrency": 8,
                        "platform": "Win32",
                        "user_agent": "Mozilla/5.0",
                        "os_cpu": ""
                    },
                    "screen": { "width": 1920, "height": 1200, "pixel_ratio": 1 }
                },
                "proxy": {
                    "type": "http",
                    "host": "proxyhost.com",
                    "port": 8081,
                    "username": "user",
                    "password": "pass",
                    "save_traffic": false
                },
                "custom_start_urls": ["https://example.com"]
            }
        });
        let params: ProfileParams = serde_json::from_value(raw).unwrap();
        assert_eq!(params.name, "Profile_name");
        assert_eq!(params.browser_type, "mimic");
        assert_eq!(params.core_version, Some(124));
        assert!(!params.auto_update_core);
        assert_eq!(
            params.parameters.proxy.as_ref().unwrap().host,
            "proxyhost.com"
        );
        assert_eq!(params.user_agent().unwrap(), "Mozilla/5.0");
        assert_eq!(
            params.proxy_string().unwrap(),
            "http://user:pass@proxyhost.com:8081"
        );
    }

    #[test]
    fn defaults_are_applied() {
        let params: ProfileParams = serde_json::from_value(json!({
            "name": "Minimal",
            "parameters": {}
        }))
        .unwrap();
        assert_eq!(params.browser_type, "mimic");
        assert_eq!(params.os_type, "windows");
        assert_eq!(params.parameters.flags.webrtc_masking, "mask");
        assert!(params.parameters.storage.is_local);
        assert_eq!(params.times, 1);
    }
}
