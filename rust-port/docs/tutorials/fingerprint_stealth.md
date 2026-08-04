# Fingerprint & Stealth Profiles

`seleniumbase-rs` can spoof many of the browser signals that bot-detection
services inspect: `navigator.webdriver`, user agent, platform, screen size,
hardware concurrency, WebGL vendor/renderer, canvas/audio noise, time zone,
geolocation, media devices, fonts, and more.

The feature set is inspired by external anti-detect profile tooling,
`chromiumoxide_stealth`, `undetected-chromedriver`, `eoka`, and `chaser-oxide`.
It works in both WebDriver + CDP mode and in the native `rustwright`
Playwright-compatible mode.

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

`StealthFlags` controls which dimensions are spoofed. Every flag is one of the
following modes:

| Mode | Meaning | When to use |
|---|---|---|
| `Natural` | Use the browser's real value. | Trust the host for audio, media devices, or fonts. |
| `Mask` | Apply a generic, deterministic spoofed value. | Hide the real WebRTC IP policy, screen size, or timezone. |
| `Custom` | Use the explicit value supplied in the `Fingerprint`. | Set a specific `user_agent`, `screen` resolution, `proxy`, or `geolocation`. |
| `Disabled` | Turn the feature off entirely. | Disable WebRTC, block QUIC, or leave proxy unconfigured. |

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
| `battery_masking` | natural / mask / custom / disabled | mask |
| `connection_masking` | natural / mask / custom / disabled | mask |
| `speech_masking` | natural / mask / custom / disabled | mask |
| `bluetooth_masking` | natural / mask / custom / disabled | mask |
| `client_hints_masking` | natural / mask / custom / disabled | mask |
| `native_tostring_masking` | natural / mask / custom / disabled | mask |
| `chrome_runtime_masking` | natural / mask / custom / disabled | mask |
| `headless_masking` | natural / mask / custom / disabled | mask |
| `humanize` | bool | false |
| `block_trackers` | bool | false |
| `proxy_masking` | disabled / custom | disabled |

Use `StealthFlags::balanced()` for sensible defaults or `StealthFlags::all_custom()`
when every value is supplied explicitly.

### Concrete mask-mode examples

#### Custom navigator (`navigator_masking: Custom`)

```rust
use seleniumbase_rs::{Fingerprint, StealthFlags};

let mut fp = Fingerprint::builder()
    .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36")
    .platform("Win32")
    .hardware_concurrency(8)
    .device_memory(8.0)
    .build();
fp.flags.navigator_masking = seleniumbase_rs::MaskingMode::Custom;
```

When the bootstrap runs, it overrides `navigator.userAgent`, `navigator.platform`,
`hardwareConcurrency`, and `deviceMemory` with the values you supplied.

#### Custom screen + geolocation (`screen_masking` / `geolocation_masking: Custom`)

```rust
use seleniumbase_rs::{Fingerprint, StealthFlags};

let mut fp = Fingerprint::builder()
    .screen(1920, 1080)
    .geolocation(52.52, 13.405)
    .timezone("Europe/Berlin")
    .build();
fp.flags.screen_masking = seleniumbase_rs::MaskingMode::Custom;
fp.flags.geolocation_masking = seleniumbase_rs::MaskingMode::Custom;
fp.flags.timezone_masking = seleniumbase_rs::MaskingMode::Mask;
```

The launcher resizes the browser window to 1920×1080, overrides the geolocation
via CDP, and the bootstrap spoofs `Intl.DateTimeFormat` to Berlin time.

#### Custom proxy (`proxy_masking: Custom`)

```rust
use seleniumbase_rs::{Fingerprint, ProxyMaskingMode};

let mut fp = Fingerprint::default();
fp.proxy = Some("http://alice:secret@proxy.example.com:8080".parse().unwrap());
fp.flags.proxy_masking = ProxyMaskingMode::Custom;
```

`ProxyMaskingMode::Custom` emits `--proxy-server=http://alice:secret@proxy.example.com:8080`.
Use `ProxyMaskingMode::Disabled` to leave proxy configuration empty.

#### WebRTC masked

```rust
use seleniumbase_rs::{Fingerprint, MaskingMode, WebRtcPolicy};

let mut fp = Fingerprint::default();
fp.flags.webrtc_masking = MaskingMode::Mask;
fp.webrtc_policy = WebRtcPolicy::DisableNonProxiedUdp;
```

This adds `--force-webrtc-ip-handling-policy=disable_non_proxied_udp` so WebRTC
cannot leak the local IP address. Set `webrtc_masking: Disabled` to omit the
flag and let the browser use its default policy.

## rustwright / Playwright mode

```rust
use seleniumbase_rs::browser::playwright::PlaywrightSession;
use seleniumbase_rs::Fingerprint;

let fp = Fingerprint::windows_desktop();
let session = PlaywrightSession::launch_with_fingerprint(&fp).await?;
session.goto("https://example.com").await?;
```

## External profile payloads

The `profile_payloads::ProfileParams` type maps to a `Fingerprint` via
`ProfileParams::to_fingerprint()`. `ProfileParams::to_browser_config()` attaches
the fingerprint to the returned `BrowserConfig`, so the spoofed values are
applied automatically at launch.

## Provider architecture

Every evasion is an [`EvasionProvider`](crate::EvasionProvider): a small,
self-contained unit that returns a JavaScript snippet for an
[`EvasionContext`](crate::EvasionContext) (the session fingerprint plus a
deterministic seed and runtime config). Providers are held by an
[`EvasionRegistry`](crate::EvasionRegistry) and assembled into a single
bootstrap script in priority order (lower `priority()` runs first).

```rust
use seleniumbase_rs::{default_registry, EvasionContext, Fingerprint};

let fp = Fingerprint::windows_desktop();
let ctx = EvasionContext::new(&fp);            // seed derived from the fingerprint
let script = default_registry().bootstrap(&ctx);
assert!(script.contains("webdriver"));
```

`evasions::bootstrap_script(&fp)` is a thin wrapper over
`default_registry().bootstrap(&EvasionContext::new(&fp))`, so existing callers
keep working while new evasions are added centrally.

### Adding your own evasion

1. Implement `EvasionProvider` for a unit struct. Return the JavaScript from
   `script`, gate inclusion with `applies`, and order it with `priority`.
2. Register it at runtime:

```rust
use seleniumbase_rs::{default_registry, EvasionContext, EvasionProvider, Fingerprint};

struct HidePrint;
impl EvasionProvider for HidePrint {
    fn name(&self) -> &str { "hide_print" }
    fn priority(&self) -> i32 { 140 }
    fn script(&self, _ctx: &EvasionContext) -> Option<String> {
        Some("window.print = function() {};".to_owned())
    }
}

let mut registry = default_registry();
registry.register_provider(Box::new(HidePrint));
let fp = Fingerprint::windows_desktop();
let script = registry.bootstrap(&EvasionContext::new(&fp));
assert!(script.contains("hide_print"));
```

See [Contributing](../CONTRIBUTING.md) for adding a built-in provider.

### Built-in providers

Registered by `default_registry()`, in execution order:

| Priority | Name | Spoofs |
|---|---|---|
| 5 | `native_tostring` | `Function.prototype.toString` returns `[native code]` for patched functions |
| 10 | `webdriver` | `navigator.webdriver`, `callPhantom`, `__selenium`, `__driver` markers |
| 15 | `cdp_markers` | scrubs `$cdc_`, `__webdriver`, `__driver` properties |
| 20 | `chrome_runtime` | `window.chrome` `runtime` / `app` / `csi` / `loadTimes` |
| 30 | `navigator_props` | userAgent, platform, languages, hardwareConcurrency, deviceMemory, vendor, oscpu, productSub, buildID, maxTouchPoints |
| 35 | `permissions` | `Notification.permission`, `permissions.query` |
| 40 | `plugins` | five PDF-viewer plugins and mimeTypes |
| 45 | `window_geometry` | screen/window dimensions, `devicePixelRatio` |
| 50 | `webgl` | WebGL1/WebGL2 `UNMASKED_VENDOR_WEBGL` / `UNMASKED_RENDERER_WEBGL` |
| 55 | `canvas_noise` | deterministic per-session noise on `toDataURL` / `toBlob` |
| 60 | `audio_noise` | deterministic noise on `AudioBuffer.getChannelData` |
| 65 | `webrtc` | filters STUN/TURN servers to prevent IP leaks |
| 70 | `battery` | Battery Status API |
| 72 | `connection` | Network Information API (`effectiveType`, `rtt`, `downlink`) |
| 75 | `media_devices` | `enumerateDevices` audioinput/audiooutput/videoinput |
| 78 | `speech` | `speechSynthesis.getVoices` |
| 80 | `bluetooth` | `navigator.bluetooth` stub |
| 85 | `headless` | `matchMedia`, `prefers-reduced-motion`, missing-plugin tells |
| 88 | `client_hints` | `navigator.userAgentData` brands/platform/architecture/model |
| 90 | `timezone` | `Intl.DateTimeFormat` / `Date` time zone |
| 92 | `localization` | language / locale |
| 95 | `geolocation` | `geolocation.getCurrentPosition` |
| 110 | `hairline` | image-`srcset` / device-pixel hairline feature detection |
| 120 | `iframe` | `contentWindow` self-defense for same-origin frames |
| 130 | `tracker_block` | optional fetch guard for known tracker hosts |

List them at runtime with `default_registry().provider_names()`, or over MCP
with the `list_evasion_providers` tool.

## Profile coherence validation

`Fingerprint::validate()` returns a [`CoherenceReport`](crate::CoherenceReport)
that flags mismatched signals (for example a macOS user agent with a `Win32`
platform, or a mobile user agent with `maxTouchPoints == 0`):

```rust
use seleniumbase_rs::Fingerprint;

let report = Fingerprint::windows_desktop().validate();
assert!(report.is_coherent());
for warning in &report.warnings {
    eprintln!("warning: {warning}");
}
```

The MCP `validate_fingerprint` tool exposes the same report.

## Humanized input

Enable `StealthFlags::humanize` (and configure `Fingerprint::humanize`) to opt
into human-like timing. The [`humanize`](crate::stealth::humanize) module
provides deterministic helpers:

```rust
use seleniumbase_rs::stealth::humanize::{bezier_mouse_path, keystroke_delays, Point};

let path = bezier_mouse_path(Point::new(0.0, 0.0), Point::new(200.0, 90.0), 24, 7);
let delays = keystroke_delays("hello", 40, 180, 7);
assert_eq!(path.len(), 24);
assert_eq!(delays.len(), 5);
```

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
* Battery Status, Network Information, Speech Synthesis, and Bluetooth APIs
* Client Hints (`navigator.userAgentData`) brands, platform, and architecture
* WebRTC STUN/TURN filtering to prevent local-IP leaks
* Launch args such as `--disable-blink-features=AutomationControlled`

## Complementary defenses

Runtime fingerprints work best when the browser also hides engine-level
automation markers:

* Patch the `chromedriver` binary with [`ChromedriverPatcher`](crate::ChromedriverPatcher)
  to remove injected `cdc_` and `__webdriver` signatures. See the
  [Binary Patching tutorial](./binary_patching.md).
* Pass extra Chromium flags returned by [`engine_spoofing_args()`](crate::engine_spoofing_args)
  through `BrowserConfig::with_extra_args` or `StealthOptions::extra_args`.
* Combine a `Fingerprint`, UC mode, binary patching, and engine args for the
  strongest anti-detection profile.

## Limitations

* TLS / JA3 / JA4 fingerprint spoofing is not implemented. For pure HTTP
  requests that need browser-faithful TLS, consider `wreq` + `wreq-util`.
* The spoofed values are applied at the CDP / JavaScript layer; no Chromium
  source patching is performed.
