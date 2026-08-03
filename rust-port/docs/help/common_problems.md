# Common Problems

## Detected as a bot despite UC mode

UC mode removes many fingerprints, but detection is layered. Check each defense:

1. **Patch the chromedriver binary.** Use
   [`ChromedriverPatcher`](../tutorials/binary_patching.md) or
   `cargo run --bin sbase -- patch-chromedriver --path /path/to/chromedriver`.
   Verify with `needs_patch()`.
2. **Add engine spoofing args.** Pass `engine_spoofing_args()` through
   `BrowserConfig::with_extra_args` or `StealthOptions::extra_args`.
3. **Use a coherent fingerprint.** A Windows user agent with a macOS WebGL
   vendor or a mobile screen on a desktop platform is suspicious. Build a
   [`Fingerprint`](../tutorials/fingerprint_stealth.md) and keep the values
   consistent.
4. **Check WebGL / geolocation consistency.** Spoofed WebGL vendor/renderer and
   geolocation should match the IP address and timezone.
5. **Run headful or use `--headless=new`.** Some sites test for the legacy
   headless implementation.

## chromedriver executable not found

`BaseCase::new` fails with a WebDriver or connection error when chromedriver is
missing or not running:

```bash
which chromedriver
# empty? download it:
cargo run --bin sbase -- install chromedriver
```

Alternatively, download a matching version from the
[Chrome for Testing dashboard](https://googlechromelabs.github.io/chrome-for-testing/)
and pass the path explicitly:

```bash
cargo run --bin sbase -- --webdriver http://localhost:9515 open https://example.com
```

If you want SeleniumBase to launch the driver for you, make sure
`auto_start_driver: true` is set in `BrowserConfig` or `sbase_config.toml`.

## Headless detected

Switch to headful mode for sites that refuse legacy headless:

```rust
let config = BrowserConfig {
    headless: false,
    ..BrowserConfig::default().with_mode(DriverMode::Uc)
};
```

Or use `--headless=new` via `StealthOptions` or `extra_args`. Combine this with
a `Fingerprint` and binary patching.

## cargo publish fails with a git dependency

The optional `playwright` feature depends on `rustwright` from GitHub:

```toml
rustwright = { git = "https://github.com/Skyvern-AI/rustwright", optional = true }
```

`cargo publish` rejects git dependencies. Publish only the default feature set,
or pin `rustwright` to a crates.io release before publishing. The default build
does not pull `rustwright`.

## Playwright / rustwright feature not compiling

Enable the feature explicitly and ensure the git dependency can be fetched:

```bash
cargo build --features playwright
```

If compilation fails because of network access to GitHub, vendor the dependency
or use a crates.io mirror. The `playwright` feature is disabled by default, so a
plain `cargo build` does not require `rustwright`.

## MCP server not listing tools

The `seleniumbase-mcp` binary only exists when the `mcp-server` feature is
enabled:

```bash
cargo build --bin seleniumbase-mcp --features mcp-server
```

The server communicates over stdio. Configure your MCP client with the absolute
path to the built binary and an empty argument list. Listing tools does not
require a running browser; the session starts lazily on the first browser tool
call.

## Macro not found

Macros are `#[macro_export]` and live at the crate root, not under
`seleniumbase_rs::macros`:

```rust
use seleniumbase_rs::{sb_open, sb_click, sb_test, selector};
```

Action macros expand `.await` internally, so they must be called inside an
`async` block or function. `sb_test!` generates a `#[tokio::test]` wrapper for
you.
