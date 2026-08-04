# SeleniumBase for Rust

This book explains how to build browser automation and end-to-end tests with
`seleniumbase-rs`. The crate combines an ergonomic `BaseCase` API with Rust's
type system, async runtime, test ecosystem, and native deployment model.

Start with [Getting Started](tutorials/getting_started.md). If you are moving
an existing suite, use [Migrate Python Tests](python-migration.md).

## What is included

- WebDriver, CDP, UC, and optional Playwright browser modes.
- A 200+ method `BaseCase` API covering actions, waits, assertions, downloads,
  screenshots, PDF tools, Shadow DOM, frames, windows, cookies, storage, MFA/TOTP,
  and JS execution.
- Recorder, Gherkin, Selenium IDE, Python migration, and low-code JSON scenario
  runners.
- A Rust-native test lifecycle that attempts asynchronous browser cleanup.
- A pluggable stealth evasion registry with 20+ built-in providers:
  navigator, WebGL, fonts, plugins, permissions, battery, canvas noise,
  WebRTC, media devices, tracker blocking, window geometry, and more.
- Native-level spoofing via CDP `setUserAgentOverride` / `setLocaleOverride`
  and binary patching for Chrome and Chromedriver.
- Multilogin-style browser profile payloads with concrete masking values for
  screen, geolocation, timezone, fonts, WebGL, WebRTC, proxy, and ports.
- Built-in fingerprint presets including `chrome_windows`, `firefox_windows`,
  `safari_macos`, `edge_windows`, `linux_desktop`, `android_chrome`, and
  `ios_mobile_safari`.
- Twelve-Factor runtime configuration through `SB_*` environment variables,
  structured tracing, and graceful shutdown.
- Static HTML checks with stable rule IDs and source selectors.
- Shell completions, admin commands (`sbase doctor`, `sbase patch-chrome`),
  and an optional Model Context Protocol server.

## Current status

The initial feature-port plan is complete. The crate builds, passes clippy,
and tests cleanly on stable Rust with all feature flags enabled. Work is
pushed to:

- Monorepo feature branch: `MustCodeAl/SeleniumBase/mustcodeal-rust-port`
- Crate-only orphan branch: `MustCodeAl/seleniumbase-rs/main`

`cargo publish --dry-run` is blocked only by the `rustwright` git dependency
(tag `v0.2.0` is newer than crates.io `0.1.1`); an upstream release is needed
before the crate can be published.

## Build this book

Install [mdBook](https://rust-lang.github.io/mdBook/guide/installation.html),
then run:

```bash
mdbook serve --open
```

