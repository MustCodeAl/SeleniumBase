//! JavaScript anti-detection / fingerprint spoofing payloads.
//!
//! The functions in this module build self-contained IIFE scripts from a
//! [`Fingerprint`](crate::stealth::fingerprint::Fingerprint). The returned
//! strings are suitable for:
//!
//! * `Page.addScriptToEvaluateOnNewDocument` via CDP (injected before any page
//!   JavaScript runs),
//! * `Page.evaluate` after navigation (rustwright / Playwright-compatible
//!   mode), or
//! * execution by a [`BrowserSession`](crate::browser::session::BrowserSession)
//!   helper.
//!
//! The evasions are inspired by `chromiumoxide_stealth`, `eoka`,
//! `chaser-oxide`, and `undetected-chromedriver`.

use std::collections::HashMap;

use crate::stealth::fingerprint::{Fingerprint, OsType, ProxyMaskingMode};
use crate::stealth::providers::{default_registry, EvasionContext};

/// Generates a single combined bootstrap script for the given fingerprint.
///
/// This delegates to the [provider registry](crate::stealth::providers), which
/// assembles every applicable [`EvasionProvider`](crate::stealth::providers::EvasionProvider)
/// in priority order. The returned script is wrapped in an IIFE and modifies
/// prototype-level properties so spoofed values survive `getOwnPropertyNames`
/// scans and re-define attempts by page scripts.
pub fn bootstrap_script(fp: &Fingerprint) -> String {
    default_registry().bootstrap(&EvasionContext::new(fp))
}

/// Returns a script that scrubs `cdc_` / webdriver markers from `window` and
/// the prototype chain. Run inline on every navigation for WebDriver sessions.
pub fn cdc_scrub() -> String {
    "(() => {
  const re = /^[a-z]{3}_[a-zA-Z0-9]{22}_.*/;
  function scrub(obj) {
    if (!obj) return;
    Object.getOwnPropertyNames(obj).forEach(key => {
      if (re.test(key)) {
        try { delete obj[key]; } catch (e) {}
      }
    });
  }
  let o = window;
  while (o) {
    scrub(o);
    o = Object.getPrototypeOf(o);
  }
})();"
        .to_owned()
}

/// Returns Chromium command-line arguments derived from a fingerprint.
pub fn launch_args(fp: &Fingerprint) -> Vec<String> {
    let mut args = vec![
        "--disable-blink-features=AutomationControlled".to_owned(),
        "--disable-infobars".to_owned(),
        "--no-default-browser-check".to_owned(),
        "--no-first-run".to_owned(),
        "--no-service-autorun".to_owned(),
        "--no-pings".to_owned(),
        "--disable-background-timer-throttling".to_owned(),
        "--disable-backgrounding-occluded-windows".to_owned(),
        "--disable-renderer-backgrounding".to_owned(),
        "--disable-features=IsolateOrigins,site-per-process,PrivacySandboxSettings4".to_owned(),
    ];

    if let Some(ua) = fp.user_agent.as_deref() {
        args.push(format!("--user-agent={ua}"));
    }

    if let Some(locale) = fp.locale.as_deref() {
        args.push(format!("--lang={locale}"));
    }

    if let (Some(w), Some(h)) = (fp.screen_width, fp.screen_height) {
        args.push(format!("--window-size={w},{h}"));
    }

    match fp.webrtc_policy {
        crate::stealth::fingerprint::WebRtcPolicy::DisableNonProxiedUdp => {
            args.push("--force-webrtc-ip-handling-policy=disable_non_proxied_udp".to_owned());
        }
        crate::stealth::fingerprint::WebRtcPolicy::PublicInterfaceOnly => {
            args.push("--force-webrtc-ip-handling-policy=default_public_interface_only".to_owned());
        }
        crate::stealth::fingerprint::WebRtcPolicy::PublicAndPrivateInterfaces => {
            args.push(
                "--force-webrtc-ip-handling-policy=default_public_and_private_interfaces"
                    .to_owned(),
            );
        }
    }

    if matches!(fp.flags.proxy_masking, ProxyMaskingMode::Custom) {
        if let Some(proxy) = fp.proxy.as_ref() {
            args.push(format!("--proxy-server={}", proxy.to_url()));
        }
    }

    if fp.flags.quic_mode == crate::stealth::fingerprint::QuicMode::Disabled {
        args.push("--disable-quic".to_owned());
    }

    for (flag, value) in &fp.cmd_params {
        if value.is_empty() {
            args.push(format!("--{flag}"));
        } else {
            args.push(format!("--{flag}={value}"));
        }
    }

    args
}

/// Returns CDP parameter overrides recommended for a fingerprint.
pub fn cdp_overrides(fp: &Fingerprint) -> HashMap<String, serde_json::Value> {
    let mut map = HashMap::new();

    if let (Some(lat), Some(lon)) = (fp.latitude, fp.longitude) {
        map.insert(
            "Emulation.setGeolocationOverride".to_owned(),
            serde_json::json!({
                "latitude": lat,
                "longitude": lon,
                "accuracy": fp.accuracy.unwrap_or(100.0),
                "altitude": fp.altitude.unwrap_or(0.0),
            }),
        );
    }

    if let (Some(w), Some(h)) = (fp.screen_width, fp.screen_height) {
        map.insert(
            "Emulation.setDeviceMetricsOverride".to_owned(),
            serde_json::json!({
                "width": w,
                "height": h,
                "deviceScaleFactor": fp.pixel_ratio.unwrap_or(1.0),
                "mobile": fp.os_type == OsType::Android,
            }),
        );
    }

    if let Some(ua) = fp.user_agent.as_deref() {
        map.insert(
            "Network.setUserAgentOverride".to_owned(),
            serde_json::json!({ "userAgent": ua }),
        );
    }

    if let Some(tz) = fp.timezone.as_deref() {
        map.insert(
            "Emulation.setTimezoneOverride".to_owned(),
            serde_json::json!({ "timezoneId": tz }),
        );
    }

    map
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stealth::fingerprint::Fingerprint;

    #[test]
    fn bootstrap_includes_webdriver_evasion() {
        let fp = Fingerprint::windows_desktop();
        let script = bootstrap_script(&fp);
        assert!(script.contains("webdriver"));
        assert!(script.contains("Google Inc. (NVIDIA)"));
        assert!(script.contains("1920"));
    }

    #[test]
    fn launch_args_include_stealth_flags() {
        let fp = Fingerprint::windows_desktop();
        let args = launch_args(&fp);
        assert!(args.iter().any(|a| a.contains("AutomationControlled")));
        assert!(args.iter().any(|a| a.starts_with("--user-agent=")));
        assert!(args.iter().any(|a| a.starts_with("--window-size=")));
    }

    #[test]
    fn cdp_overrides_contain_screen_and_geo() {
        let fp = Fingerprint::builder()
            .screen(1920, 1080)
            .geolocation(40.0, -74.0)
            .build();
        let map = cdp_overrides(&fp);
        assert!(map.contains_key("Emulation.setDeviceMetricsOverride"));
        assert!(map.contains_key("Emulation.setGeolocationOverride"));
    }
}
