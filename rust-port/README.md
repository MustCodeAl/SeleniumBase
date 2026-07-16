## SeleniumBase Rust Port (CDP/UC Foundation)

This directory contains a Rust SeleniumBase edition foundation with `thirtyfour`
as the engine and an initial SeleniumBase-style surface.

It provides:

- `BaseCase` API with actions, waits, assertions, highlighting, JS execution, hover, select options, and frame switching.
- `DriverMode::{WebDriver, Cdp, Uc}` in `BrowserConfig`.
- CDP command execution (`execute_cdp`, `execute_cdp_with_params`).
- CDP network/cache helpers (`set_network_conditions`, `clear_browser_cache`).
- UC-style stealth bootstrap (Chromium flags + `navigator.webdriver` hardening script).
- SeleniumBase-style artifact helpers:
  - `save_screenshot_to_logs()` to `./latest_logs/*.png`
  - `save_page_source_to_logs()` to `./latest_logs/*.html`
- Recorder + scenario runner foundation:
  - action recording export to JSON and Rust script
  - JSON scenario execution with HTML dashboard output
- `sbase` CLI with `--cdp` and `--uc` modes.

### Quick start

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
```

### Run raw CDP command with params

```bash
cargo run --bin sbase -- --cdp cdp --cmd Network.setCacheDisabled --params '{"cacheDisabled":true}'
```

### Save artifacts

```bash
cargo run --bin sbase -- screenshot
cargo run --bin sbase -- save-source
```

### assertion and wait helpers from CLI

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
