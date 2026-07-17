# SeleniumBase for Rust

A Rust port of SeleniumBase using `thirtyfour` as the underlying WebDriver engine.

It provides API parity with the Python SeleniumBase library for core DOM
interactions, plus stealth modes, CDP integrations, and a command-line helper.

### Key Features Ported

- **BaseCase API**: 200+ actions, waits, assertions, and DOM manipulation methods.
  - Basic interactions: `open`, `click`, `type_text`, `submit`, `clear`, `hover`
  - Advanced interactions: `double_click`, `context_click`, `slow_click`, `drag_and_drop`
  - JS interactions: `js_click`, `js_type`, `execute_script`, `execute_async_script`
  - Selectors & Shadow DOM: `find_element`, `find_elements`, `shadow_click`, `shadow_type`, `shadow_get_text`
  - Attributes/Properties: `get_attribute`, `get_property`, `set_attribute`, `remove_attribute`
  - Windows & Frames: `switch_to_frame`, `switch_to_default_content`, `switch_to_window`, `switch_to_new_window`, `maximize_window`
  - Navigation: `go_back`, `go_forward`, `refresh`
  - Scrolling: `scroll_to`, `scroll_to_top`, `scroll_to_bottom`
  - Alerts: `accept_alert`, `dismiss_alert`, `type_alert_text`
  - Cookies & Storage: `get_cookie`, `add_cookie`, `delete_all_cookies`, `set_local_storage_item`, `clear_local_storage`, `remove_local_storage_item`
  - Uploads: `choose_file`
  - MFA/TOTP: `get_totp_code`
  - PDF: `save_as_pdf`, `get_pdf_text`, `assert_pdf_text`
  - HTML parsing: `soup_find`, `soup_find_all`, `get_beautiful_soup_object`
- **Driver Modes (`BrowserConfig`)**: `WebDriver`, `Cdp`, `Uc` (Undetected Chromedriver).
- **Stealth Integrations**:
  - UC mode (Chromium flags + `navigator.webdriver` evasion).
  - CDC stealth binary patcher to strip hardcoded signatures from the `chromedriver` executable.
- **CDP Automation**:
  - Raw CDP commands: `execute_cdp`, `execute_cdp_with_params`.
  - High-level CDP page API: `cdp_open`, `cdp_click`, `cdp_type`, `cdp_get_text`, `cdp_evaluate`, `cdp_screenshot`.
  - Standalone async CDP driver (`CdpDriver`) for direct WebSocket control.
  - Network & Cache: `set_network_conditions`, `clear_browser_cache`.
- **Playwright Mode** (optional `playwright` feature): Bypass bot-detection using a Playwright-backed driver.
- **GUI Automation**: OS-level mouse and keyboard control via `enigo` (`gui_click`, `gui_type`, `gui_key_sequence`).
- **Native Dialogs**: Cross-platform message/confirm/input dialogs via `rfd` (`dialog`, `dialog_confirm`, `dialog_input`).
- **HTML Inspector**: Lint-like checks for missing alt text, empty links, duplicate ids, skipped headings, and landmarks.
- **MasterQA**: Hybrid manual testing session with Markdown test-case reports.
- **Global Config**: Load `sbase_config.toml` / `.sbase_config` with env overrides.
- **Commander TUI**: Terminal UI for browsing and running tests (`sbase commander`).
- **Recorder CLI**: Capture interactions and generate Rust tests (`sbase recorder`).
- **Cloud Integrations**: Upload artifacts to S3, Azure Blob Storage, or Google Cloud Storage behind feature flags (`--features s3/azure/gcp`).
- **CI/CD & Docker**: Ready-to-use GitHub Actions workflows and a `Dockerfile`.
- **Test Artifacts**: `save_screenshot_to_logs()`, `save_page_source_to_logs()`.
- **Low-Code Runner**: JSON scenario execution with an HTML dashboard.
- **Action Recorder**: Captures browser interactions and compiles them into a JSON scenario or a standalone Rust script.
- **Interactive CLI (`sbase`)**: Execute single commands directly from the terminal with `--headless`, `--mobile`, `--proxy`, `--proxy-pac-url`, `--user-data-dir`, `--extension-dir`, `--reuse-session`, and `-n` threads.

## Documentation

- [Getting Started](./docs/tutorials/getting_started.md)
- [Undetected (UC) Mode](./docs/tutorials/uc_mode.md)
- [CLI Usage](./docs/tutorials/cli_usage.md)
- [API Reference](./docs/tutorials/api_reference.md)

## Quick start

```bash
cd rust-port
cargo run --bin sbase -- --cdp open https://seleniumbase.io
```

The command expects a running WebDriver endpoint at `http://localhost:4444`.
Override it with `--webdriver` when needed.

### Smoke test with assertion

```bash
cargo run --bin sbase -- --uc smoke https://seleniumbase.io --title-contains SeleniumBase
```

### Run raw CDP command

```bash
cargo run --bin sbase -- --cdp cdp --cmd Browser.getVersion
cargo run --bin sbase -- --cdp cdp --cmd Network.setCacheDisabled --params '{"cacheDisabled":true}'
```

### Save artifacts

```bash
cargo run --bin sbase -- screenshot
cargo run --bin sbase -- save-source
```

### Assertions and Waits from CLI

```bash
cargo run --bin sbase -- open https://seleniumbase.io
cargo run --bin sbase -- assert-element --css "body"
cargo run --bin sbase -- wait-for-text --css "body" --text "SeleniumBase" --timeout 15
```

### CDC Stealth Binary Patcher

You can patch your downloaded `chromedriver` executable directly to remove hardcoded CDC variables and signatures (matching SeleniumBase Python `undetected-chromedriver` patches):

```bash
cargo run --bin sbase -- patch-chromedriver --path /path/to/chromedriver
```

### CDP page automation

```rust
use seleniumbase_rs::{BaseCase, BrowserConfig, DriverMode};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut sb = BaseCase::new(BrowserConfig {
        mode: DriverMode::Cdp,
        ..Default::default()
    }).await?;
    sb.cdp_open("https://seleniumbase.io").await?;
    let text = sb.cdp_get_text("h1").await?;
    println!("{text}");
    sb.quit().await?;
    Ok(())
}
```

### Shadow DOM piercing

```rust
sb.shadow_click("my-app ::shadow button").await?;
sb.shadow_type("my-app ::shadow input", "hello").await?;
```

### Native dialogs

```rust
sb.show_message("Welcome", "Welcome to the SeleniumBase Rust demo!");
if sb.show_confirm("Continue?", "Do you want to continue?") {
    let result = sb.show_prompt("Your name", "What is your name?", Some("guest"));
    let name = result.text.unwrap_or_else(|| "guest".to_string());
    sb.show_message("Hello", &format!("Hello, {name}!"));
}
```

### HTML Inspector

```rust
let inspection = sb.inspect_html().await?;
assert!(inspection.is_clean(), "{inspection:?}");
```

### GUI automation

```rust
sb.gui_click_x_y(100, 200)?;
sb.gui_write("hello")?;
sb.gui_press_keys(&["command", "a"])?;
```

### Run JSON scenario and generate dashboard

```bash
cargo run --bin sbase -- run-scenario --file ./scenario.json
```

Example `scenario.json`:

```json
{
  "name": "basic_flow",
  "steps": [
    {"action": "open", "url": "https://seleniumbase.io"},
    {"action": "assert_element", "css": "body"},
    {"action": "wait_for_text", "css": "body", "text": "SeleniumBase", "timeout": 15}
  ]
}
```

### Cloud artifact uploads

Upload screenshots or logs to S3, Azure Blob Storage, or Google Cloud Storage.

```bash
# S3
export AWS_ACCESS_KEY_ID=...
export AWS_SECRET_ACCESS_KEY=...
cargo run --example cloud_upload --features s3

# Azure Blob Storage (SAS URL with write permission)
export AZURE_BLOB_URL="https://myaccount.blob.core.windows.net/mycontainer/myblob?sv=...&sig=..."
cargo run --example cloud_upload --features azure

# Google Cloud Storage
export GCS_ACCESS_TOKEN=$(gcloud auth print-access-token)
cargo run --example cloud_upload --features gcp
```

See [`examples/cloud_upload.rs`](./examples/cloud_upload.rs) for the full snippet.

### Playwright mode (optional feature)

Playwright-backed stealth mode is available behind the `playwright` feature. The
upstream `playwright` crate (`0.0.20`) downloads a native driver during its build
script; the hosted driver URL is currently unreachable (HTTP 404), so the
feature may fail to build on hosts without a cached driver. The feature is left
disabled by default so the main build stays green:

```bash
cargo run --example playwright_mode --features playwright
```

If the driver download fails, use the default CDP or UC modes instead.

### Commander TUI

Browse and run tests or examples interactively:

```bash
cargo run --bin sbase -- commander
```

Use `↑/↓` or `j/k` to navigate, `Enter` to run the selected item, `/` or `f` to
filter, `r` to refresh, and `q` to quit.
