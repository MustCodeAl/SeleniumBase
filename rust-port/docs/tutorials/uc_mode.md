# Undetected (UC) Mode Guide

UC mode reduces the fingerprints that sites use to detect automated browsers.

## Enable UC mode

```rust
use seleniumbase_rs::{BaseCase, BrowserConfig, DriverMode};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = BrowserConfig::default()
        .with_mode(DriverMode::Uc);

    let mut sb = BaseCase::new(config).await?;
    sb.open("https://seleniumbase.io/demo_page").await?;

    //navigator.webdriver is masked, plugins are mocked, and WebGL vendor is spoofed.
    let webdriver: bool = sb.execute_script("return navigator.webdriver === undefined").await?;
    println!("webdriver hidden: {}", webdriver);

    sb.quit().await?;
    Ok(())
}
```

## What is patched automatically

- `cdc_` variables injected by chromedriver are removed.
- `navigator.webdriver` is masked.
- `navigator.plugins`, `navigator.languages`, `hardwareConcurrency`, `deviceMemory` are mocked.
- Chrome-only objects (`chrome.app`, `chrome.runtime`, `chrome.csi`, `chrome.loadTimes`) are created.
- WebGL vendor/renderer report a common Intel profile.
- `navigator.permissions.query` returns natural values.
- `enumerateDevices` returns realistic media device labels.
- iframe `contentWindow` patches propagate the mask into nested frames.

## Extra evasions you can apply

```rust
// Spoof timezone and geolocation
sb.set_timezone("America/Los_Angeles").await?;
sb.set_geolocation(34.0522, -118.2437, 100.0).await?;

// Use a custom user agent and locale
let config = BrowserConfig {
    user_agent: Some("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 ...".into()),
    locale: Some("en-US".into()),
    ..BrowserConfig::default().with_mode(DriverMode::Uc)
};

// Obtain a fresh WebDriver session without restarting the driver process
sb.reconnect().await?;
```

## Patch chromedriver binary

You can also patch a downloaded `chromedriver` executable to strip hardcoded CDC signatures:

```bash
cargo run --bin sbase -- patch-chromedriver --path /path/to/chromedriver
```

## When to use UC mode

Use UC mode when:
- A site blocks normal WebDriver traffic.
- You need to interact with cloudflare-like challenges.
- You want the most human-like browser fingerprint possible.
