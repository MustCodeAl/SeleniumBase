# Multilogin-Style Profiles

`seleniumbase-rs` can consume the same profile-creation payload used by the
Multilogin anti-detect platform. This is exposed through the
`seleniumbase_rs::multilogin` module and the Tauri multi-profile example.

## Supported fields

The `ProfileParams` type mirrors the Multilogin `POST /profile/create` body:

- `name`, `browser_type` (`mimic` or `stealthfox`), `os_type`
- `folder_id`, `tags`, `notes`, `times`
- `core_version`, `core_minor_version`, `auto_update_core`
- `parameters.flags` — masking mode for WebRTC, audio, fonts, geolocation,
  graphics, navigator, ports, proxy, screen, timezone, canvas noise, QUIC, and
  startup behavior.
- `parameters.fingerprint` — custom values for navigator, localization,
  timezone, graphics, WebRTC, media devices, screen, geolocation, ports, fonts,
  and extra command-line parameters.
- `parameters.storage` — local vs cloud storage options.
- `parameters.proxy` — HTTP/HTTPS/SOCKS proxy with optional credentials and
  traffic saving.
- `parameters.custom_start_urls` — up to 5 URLs to open on launch.

## Mapping to SeleniumBase concepts

Not every anti-detect flag has a direct WebDriver/Chrome capability. The Rust
port applies what it can and preserves the rest as metadata:

| Multilogin field | Applied via |
|---|---|
| `browser_type` | `Browser::Chrome` or `Browser::Firefox` |
| `os_type == android` | mobile emulation |
| `fingerprint.navigator.user_agent` | `--user-agent` argument |
| `fingerprint.localization.locale` | `--lang` argument |
| `parameters.proxy` | `--proxy-server` argument |
| `fingerprint.screen` | `BaseCase::set_window_size` at runtime |
| `fingerprint.geolocation` | `Emulation.setGeolocationOverride` CDP call |
| `fingerprint.cmd_params` | extra Chromium arguments |
| `parameters.custom_start_urls` | first URL is used as `start_page`; extras are opened at runtime |

Flags such as canvas noise, font masking, WebRTC masking, and idle-time behavior
masking are stored in the profile and exposed to custom CDP scripts, extensions,
or future anti-detect injection features.

## Programmatic usage

```rust
use seleniumbase_rs::multilogin::ProfileParams;
use serde_json::json;

let raw = json!({
    "name": "custom-profile",
    "browser_type": "mimic",
    "os_type": "windows",
    "parameters": {
        "flags": {
            "webrtc_masking": "custom",
            "navigator_masking": "custom"
        },
        "fingerprint": {
            "navigator": {
                "user_agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 ...",
                "platform": "Win32",
                "hardware_concurrency": 8
            }
        },
        "storage": { "is_local": true }
    }
});

let params: ProfileParams = serde_json::from_value(raw).unwrap();
let config = params.to_browser_config("http://localhost:4444");

let mut sb = seleniumbase_rs::BaseCase::new(config).await.unwrap();
params.apply_runtime_overrides(&mut sb).await.unwrap();
```

## Tauri multi-profile app

The `examples/tauri-multilogin` app accepts a Multilogin JSON payload in the
**Import Multilogin profile JSON** section. It converts the payload into a local
profile and launches it with the converted `BrowserConfig`. Profiles imported
this way show a `multilogin` badge in the profile list.

## Further reading

- [Multilogin fingerprint masking glossary](https://multilogin.com/glossary/fingerprint-masking/)
- [Multilogin device spoofing glossary](https://multilogin.com/glossary/device-spoofing/)
- [BrowserLeaks](https://browserleaks.com/)
