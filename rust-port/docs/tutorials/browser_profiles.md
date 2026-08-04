# Browser Profiles

`seleniumbase-rs` can import external browser profile payloads in a generic
anti-detect JSON format. The format is exposed through the
`seleniumbase_rs::profile_payloads` module and the Tauri profile-manager example.

This is **not** tied to any third-party product. The payload describes a browser
persona, a set of masking modes, and optional runtime overrides. `seleniumbase-rs`
translates the parts it can apply directly into `BrowserConfig`, command-line
flags, CDP calls, and window settings.

## Supported fields

The `ProfileParams` type mirrors a common `POST /profile/create` body:

- `name`, `browser_type` (`chromium` or `firefox`), `os_type`
- `folder_id`, `tags`, `notes`, `times`
- `core_version`, `core_minor_version`, `auto_update_core`
- `parameters.flags` — masking mode for WebRTC, audio, fonts, geolocation,
  graphics, navigator, ports, proxy, screen, timezone, canvas noise, QUIC, and
  startup behavior.
- `parameters.fingerprint` — explicit values for navigator, localization,
  timezone, graphics, WebRTC, media devices, screen, geolocation, ports, fonts,
  and extra command-line parameters.
- `parameters.storage` — local vs cloud storage options.
- `parameters.proxy` — HTTP/HTTPS/SOCKS proxy with optional credentials and
  traffic saving.
- `parameters.custom_start_urls` — up to 5 URLs to open on launch.

> **Backward compatibility:** The parser still accepts the legacy strings
> `mimic` and `stealthfox`, but they are treated as aliases for `chromium` and
> `firefox`. New profiles should use the generic names.

## Masking modes

Flags control how a fingerprint dimension is handled. Each mode has a concrete
meaning and a matching JSON string value:

| Mode | String | What it does | Example use case |
|---|---|---|---|
| `Natural` | `"natural"` | Use the browser's real value. | Trust the host for audio, media devices, or fonts. |
| `Mask` | `"mask"` | Apply a generic, deterministic spoofed value. | Hide the real WebRTC IP policy, screen size, or timezone. |
| `Custom` | `"custom"` | Use the explicit value supplied in `fingerprint.*`. | Set a specific `user_agent`, `screen` resolution, `proxy`, or `geolocation`. |
| `Disabled` | `"disabled"` | Turn the feature off entirely. | Disable WebRTC, block QUIC, or leave proxy unconfigured. |

### Concrete mask-mode examples

#### Custom user agent + platform (`navigator_masking: custom`)

```json
{
  "name": "custom-ua-profile",
  "browser_type": "chromium",
  "os_type": "windows",
  "parameters": {
    "flags": { "navigator_masking": "custom" },
    "fingerprint": {
      "navigator": {
        "user_agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36",
        "platform": "Win32",
        "hardware_concurrency": 8,
        "device_memory": 8
      }
    }
  }
}
```

When `navigator_masking` is `"custom"`, the launcher passes the supplied
`user_agent` to `--user-agent` and the `platform` value into the evasion
bootstrap.

#### Custom screen + geolocation (`screen_masking` / `geolocation_masking`)

```json
{
  "name": "berlin-desktop",
  "browser_type": "chromium",
  "os_type": "windows",
  "parameters": {
    "flags": {
      "screen_masking": "custom",
      "geolocation_masking": "custom",
      "timezone_masking": "mask"
    },
    "fingerprint": {
      "screen": { "width": 1920, "height": 1080, "pixel_ratio": 1 },
      "geolocation": { "latitude": 52.52, "longitude": 13.405, "accuracy": 100 },
      "timezone": { "zone": "Europe/Berlin" }
    }
  }
}
```

`screen_masking: custom` applies the exact resolution. `geolocation_masking:
custom` emits `Emulation.setGeolocationOverride` with the supplied coordinates.
`timezone_masking: mask` picks a timezone that matches the geolocation.

## Literal value shortcuts

For convenience, most masking flags also accept a concrete literal string
instead of a mode keyword. When a literal value is supplied, the parser sets the
corresponding mode to `Custom` and writes the value into the fingerprint so it
is applied automatically.

| Flag | Literal format | Example |
|---|---|---|
| `screen_masking` | `WIDTHxHEIGHTxDEPTH` | `"1920x1080x24"` |
| `geolocation_masking` | `lat,lon,altitude` | `"40.7128,-74.0060,10.5"` |
| `graphics_masking` | `vendor~renderer` | `"Intel Inc.~Intel(R) Iris(R) Xe Graphics"` |
| `navigator_masking` | full UA string | `"Mozilla/5.0 (Windows NT 10.0; Win64; x64)..."` |
| `localization_masking` | locale / accept-language | `"en-US,en;q=0.9"` |
| `timezone_masking` | IANA zone | `"America/New_York"` |
| `fonts_masking` | comma-separated fonts | `"Arial,Calibri,Consolas"` |
| `media_devices_masking` | `kind:deviceId:label\|...` | `"audioinput:default:Mic1\|videoinput:default:Cam1"` |
| `ports_masking` | `block:port,port,...` or list | `"block:80,443,3000"` |
| `proxy_masking` | proxy URL | `"socks5://user:pass@host:1080"` |
| `webrtc_masking` | `public:IP\|local:IP` | `"public:172.56.21.89\|local:10.0.0.5"` |

### Full literal-value profile

```json
{
  "name": "US_Ecom_Buyer_01",
  "browser_type": "chromium",
  "folder_id": "folder_88a2b1",
  "os_type": "windows",
  "core_version": 133,
  "parameters": {
    "flags": {
      "audio_masking": "noise",
      "fonts_masking": "Arial,Helvetica,Open Sans,Roboto,Segoe UI,Times New Roman",
      "geolocation_masking": "40.7128,-74.0060,10.5",
      "geolocation_popup": "allow",
      "graphics_masking": "Intel Inc.~Intel(R) Iris(R) Xe Graphics~WebGL 2.0",
      "graphics_noise": "medium",
      "localization_masking": "en-US,en;q=0.9",
      "media_devices_masking": "audioinput:default:Mic1|videoinput:default:Cam1",
      "navigator_masking": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
      "ports_masking": "block:80,443,3000,8080,9222",
      "proxy_masking": "socks5://user123:pass456@192.168.1.50:1080",
      "screen_masking": "1920x1080x24",
      "quic_mode": "disabled",
      "timezone_masking": "America/New_York",
      "webrtc_masking": "public:192.168.1.50|local:10.0.0.5",
      "canvas_noise": "random"
    }
  }
}
```

The parser extracts every literal value above and populates the matching
`Fingerprint` fields, so the stealth bootstrap and launch args can use them
without any extra code.

#### Custom proxy (`proxy_masking: custom`)

```json
{
  "name": "proxy-profile",
  "browser_type": "chromium",
  "parameters": {
    "flags": { "proxy_masking": "custom" },
    "proxy": {
      "type": "http",
      "host": "proxy.example.com",
      "port": 8080,
      "username": "alice",
      "password": "secret",
      "save_traffic": false
    }
  }
}
```

`proxy_masking: custom` turns the `parameters.proxy` block into
`--proxy-server=http://alice:secret@proxy.example.com:8080`. Use
`proxy_masking: disabled` to leave proxy configuration empty.

#### WebRTC disabled / masked

```json
{
  "parameters": {
    "flags": { "webrtc_masking": "mask" }
  }
}
```

`webrtc_masking: mask` forces `--force-webrtc-ip-handling-policy=disable_non_proxied_udp`
so public IP leakage is reduced. `webrtc_masking: disabled` leaves the browser's
default policy in place.

To spoof specific public/local IPs, use the literal `public:IP|local:IP` format
or the `fingerprint.webrtc.public_ip` field:

```json
{
  "parameters": {
    "flags": { "webrtc_masking": "public:172.56.21.89|local:10.0.0.5" },
    "fingerprint": {
      "webrtc": { "public_ip": "172.56.21.89" }
    }
  }
}
```

## Mapping to SeleniumBase concepts

Not every anti-detect flag has a direct WebDriver/Chrome capability. The Rust
port applies what it can and preserves the rest as metadata:

| Profile field | Applied via |
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
are stored in the profile and exposed to the stealth bootstrap, custom CDP
scripts, extensions, or future anti-detect injection features.

## Programmatic usage

```rust
use seleniumbase_rs::profile_payloads::ProfileParams;
use serde_json::json;

let raw = json!({
    "name": "custom-profile",
    "browser_type": "chromium",
    "os_type": "windows",
    "parameters": {
        "flags": {
            "webrtc_masking": "mask",
            "navigator_masking": "custom",
            "screen_masking": "custom"
        },
        "fingerprint": {
            "navigator": {
                "user_agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 ...",
                "platform": "Win32",
                "hardware_concurrency": 8
            },
            "screen": { "width": 1920, "height": 1080, "pixel_ratio": 1 }
        },
        "storage": { "is_local": true }
    }
});

let params: ProfileParams = serde_json::from_value(raw).unwrap();
let config = params.to_browser_config("http://localhost:4444");

let mut sb = seleniumbase_rs::BaseCase::new(config).await.unwrap();
params.apply_runtime_overrides(&mut sb).await.unwrap();
```

## Tauri profile-manager app

The `examples/tauri-profile-manager` app accepts a profile JSON payload in the
**Import profile JSON** section. It converts the payload into a local profile
and launches it with the converted `BrowserConfig`. Profiles imported this way
show an `external` badge in the profile list.

## Further reading

- [BrowserLeaks](https://browserleaks.com/)
