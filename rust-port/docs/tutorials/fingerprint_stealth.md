# Fingerprint & Stealth Profiles

`seleniumbase-rs` can spoof many of the browser signals that bot-detection
services inspect: `navigator.webdriver`, user agent, platform, screen size,
hardware concurrency, WebGL vendor/renderer, canvas/audio noise, time zone,
geolocation, media devices, fonts, and more.

The feature set is inspired by Multilogin, `chromiumoxide_stealth`,
`undetected-chromedriver`, `eoka`, and `chaser-oxide`. It works in both
WebDriver + CDP mode and in the native `rustwright` Playwright-compatible
mode.

## Quick start

```rust
use seleniumbase_rs::{BaseCase, BrowserConfig, DriverMode, Fingerprint};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fp = Fingerprint::windows_desktop();
    let config = BrowserConfig {
        mode: DriverMode::Uc,
        fingerprint: Some(fp),
        ..BrowserConfig::default()
    };

    let mut sb = BaseCase::new(config).await?;
    sb.open("https://example.com").await?;
    sb.quit().await?;
    Ok(())
}
```

## Fingerprint presets

* `Fingerprint::windows_desktop()`
* `Fingerprint::macos_desktop()`
* `Fingerprint::android_mobile()`

Each preset sets a coherent user agent, platform, screen size, WebGL strings,
and masking flags.

## Building a custom fingerprint

```rust
use seleniumbase_rs::Fingerprint;

let fp = Fingerprint::builder()
    .user_agent("Mozilla/5.0 …")
    .platform("Win32")
    .screen(1920, 1080)
    .hardware_concurrency(8)
    .device_memory(8.0)
    .locale("en-US")
    .timezone("America/New_York")
    .webgl("Google Inc. (NVIDIA)", "ANGLE …")
    .geolocation(40.7128, -74.0060)
    .media_devices(1, 1, 2)
    .build();
```

## Masking flags

`StealthFlags` controls which dimensions are spoofed:

| Flag | Modes | Default |
|---|---|---|
| `navigator_masking` | natural / mask / custom / disabled | mask |
| `screen_masking` | natural / mask / custom / disabled | mask |
| `graphics_masking` | natural / mask / custom / disabled | mask |
| `audio_masking` | natural / mask / custom / disabled | natural |
| `media_devices_masking` | natural / mask / custom / disabled | natural |
| `canvas_noise` | mask / natural / disabled | mask |
| `webrtc_masking` | natural / mask / custom / disabled | mask |
| `geolocation_masking` | natural / mask / custom / disabled | mask |
| `proxy_masking` | disabled / custom | disabled |

Use `StealthFlags::balanced()` for sensible defaults or `StealthFlags::all_custom()`
when every value is supplied explicitly.

## rustwright / Playwright mode

```rust
use seleniumbase_rs::browser::playwright::PlaywrightSession;
use seleniumbase_rs::Fingerprint;

let fp = Fingerprint::windows_desktop();
let session = PlaywrightSession::launch_with_fingerprint(&fp).await?;
session.goto("https://example.com").await?;
```

## Multilogin payloads

The `multilogin::ProfileParams` type already maps to a `Fingerprint` via
`ProfileParams::to_fingerprint()`. `ProfileParams::to_browser_config()` now
attaches the fingerprint to the returned `BrowserConfig`, so the spoofed
values are applied automatically at launch.

## What is spoofed

* `navigator.webdriver` removed
* `navigator.userAgent`, `platform`, `hardwareConcurrency`, `deviceMemory`, `languages`
* `window.chrome.runtime` / `window.chrome.app` stubs
* `navigator.plugins` / `navigator.mimeTypes`
* `navigator.permissions.query` for notifications
* `screen.width`, `height`, `availWidth`, `availHeight`, `colorDepth`
* WebGL `UNMASKED_VENDOR_WEBGL` / `UNMASKED_RENDERER_WEBGL`
* Canvas `toDataURL` noise
* Audio `AnalyserNode.getFloatFrequencyData` noise
* `navigator.mediaDevices.enumerateDevices`
* Time zone via `Intl.DateTimeFormat`
* Geolocation via CDP `Emulation.setGeolocationOverride` and JS fallback
* CDP marker scrubbing (`cdc_`, `__webdriver`, `__selenium`, etc.)
* Launch args such as `--disable-blink-features=AutomationControlled`

## Limitations

* TLS / JA3 / JA4 fingerprint spoofing is not implemented. For pure HTTP
  requests that need browser-faithful TLS, consider `wreq` + `wreq-util`.
* The spoofed values are applied at the CDP / JavaScript layer; no Chromium
  source or binary patching is performed except for the existing chromedriver
  `cdc_` patcher.
