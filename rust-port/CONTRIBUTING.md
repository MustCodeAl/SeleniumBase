# Contributing to SeleniumBase-rs

Thank you for helping make `seleniumbase-rs` a powerful, idiomatic Rust browser-automation framework. This guide covers how to set up the project, where things live, and how to extend the library in common ways.

## Table of Contents

1. [Quick Start](#quick-start)
2. [Project Layout](#project-layout)
3. [Code Style](#code-style)
4. [Adding a Stealth Evasion Provider](#adding-a-stealth-evasion-provider)
5. [Adding a Macro](#adding-a-macro)
6. [Adding an MCP Tool](#adding-an-mcp-tool)
7. [Writing Documentation](#writing-documentation)
8. [Testing](#testing)
9. [Opening a Pull Request](#opening-a-pull-request)
10. [Licensing](#licensing)

## Quick Start

```bash
# Clone the workspace
git clone https://github.com/MustCodeAl/SeleniumBase.git
cd SeleniumBase/rust-port

# Run the default test suite
cargo test

# Run with all optional features enabled
cargo test --features s3,azure,gcp,playwright,mcp-server

# Check formatting and clippy
cargo fmt --all -- --check
cargo clippy --all-targets --features s3,azure,gcp,playwright,mcp-server -- -D warnings
```

## Project Layout

```text
rust-port/
├── src/
│   ├── lib.rs                 # Crate root and public re-exports
│   ├── api/                   # BaseCase implementation split by topic
│   │   ├── base_case.rs       # Core driver/session management
│   │   ├── base_case_impls/   # Focused helper groups (assertions, mouse, etc.)
│   │   ├── chart.rs           # Chart generation
│   │   ├── tour.rs            # Guided tour themes
│   │   ├── traits.rs          # Capability traits (BrowserApi, ElementApi, ...)
│   │   └── ...
│   ├── browser/               # BrowserConfig, session startup, capabilities
│   ├── stealth/               # Anti-detection layer
│   │   ├── fingerprint.rs     # Fingerprint / StealthFlags structs + coherence validation
│   │   ├── humanize.rs        # Bézier mouse paths + keystroke timing
│   │   ├── patcher.rs         # Chromedriver binary patching
│   │   ├── providers/         # Evasion provider plugin architecture
│   │   │   ├── mod.rs         # EvasionProvider trait + EvasionRegistry
│   │   │   └── builtin.rs     # 25 built-in anti-detection providers
│   │   └── ...
│   ├── profile_payloads/      # External anti-detect profile JSON parsing
│   ├── macros.rs              # Public macros
│   ├── bin/mcp_server.rs      # SeleniumBase MCP server
│   ├── cli/                   # sbase command-line tool
│   ├── behave/                # Gherkin/BDD runner
│   ├── utilities/             # Python importer, IDE helpers, etc.
│   └── utils/                 # Low-level utilities (selectors, XPath, ...)
├── examples/                  # Runnable examples
├── docs/                      # mdBook sources and help pages
├── tests/                     # Additional integration tests
└── book.toml                  # mdBook configuration
```

## Code Style

- Follow the Rust style enforced by `cargo fmt`.
- Keep `cargo clippy --all-targets --features s3,azure,gcp,playwright,mcp-server -- -D warnings` clean.
- Prefer explicit types on public APIs; this helps Copilot and other agents infer intent.
- Use `tracing` for logs instead of `println!` in library code.
- Avoid unsafe code unless absolutely necessary and clearly documented.
- Keep functions small and focused. If a `BaseCase` helper grows beyond ~50 lines, consider adding it to the appropriate `base_case_impls` module.
- Do not reference third-party anti-detect product names. Use generic terms like "anti-detect profile" or "external profile payload".

## Adding a Stealth Evasion Provider

The stealth layer is built around a provider registry so new evasions can be added without touching the core `Fingerprint` generator. `evasions::bootstrap_script(fp)` is a thin wrapper over the registry.

1. Open `src/stealth/providers/builtin.rs` and implement the `EvasionProvider` trait:

```rust
use seleniumbase_rs::stealth::providers::{EvasionContext, EvasionProvider};
use seleniumbase_rs::stealth::fingerprint::masked;

pub struct MyEvasionProvider;

impl EvasionProvider for MyEvasionProvider {
    fn name(&self) -> &str {
        "my_evasion" // stable, unique, snake_case
    }

    fn priority(&self) -> i32 {
        // Lower runs earlier. native_toString is 5; navigator props ~30;
        // late/self-defense providers are 110+.
        100
    }

    fn applies(&self, fp: &crate::stealth::fingerprint::Fingerprint) -> bool {
        // Gate on a StealthFlags field so users can opt out.
        masked(fp.flags.navigator_masking)
    }

    fn script(&self, ctx: &EvasionContext) -> Option<String> {
        // Build a self-contained snippet. Escape user-controlled strings
        // with EvasionContext::escape before interpolation.
        let value = EvasionContext::escape(
            ctx.fingerprint.vendor.as_deref().unwrap_or("Example"),
        );
        Some(format!("(function() {{ /* use '{value}' */ }})();"))
    }
}
```

2. Register it in `all()` at the bottom of `builtin.rs` (keep the list sorted by priority for readability).

3. Wrap replaced natives so `Function.prototype.toString` still reports `[native code]`:

```js
obj.method = (window.__sbNative || function(f){return f;})(patched, 'method');
```

The `native_to_string` provider (priority 5) installs `window.__sbNative` before any other provider runs.

4. Use `ctx.seed` for anything that must be deterministic per session (e.g. canvas/audio noise). `ctx.seed` comes from `Fingerprint::seed_value()`.

5. Add a unit test in the `builtin` test module asserting the snippet contains the expected markers and that `applies` is correctly gated.

6. Update `docs/tutorials/fingerprint_stealth.md` to document the new provider.

### Adding a new masking flag

If your provider needs its own toggle, add a field to `StealthFlags` in `fingerprint.rs`. Annotate it `#[serde(default)]`, set it in `balanced()` and `all_custom()`, and document it in the stealth tutorial's flag table.

## Adding a Macro

Public macros live in `src/macros.rs` and are re-exported at the crate root.

1. Add your macro using `#[macro_export]`.
2. Write a small compile-time unit test under the macro definition.
3. Update `docs/tutorials/macros.md` with a usage example.
4. Update the macro table in `README.md` if one exists.

Example pattern:

```rust
#[macro_export]
macro_rules! sb_focus {
    ($sb:expr, $css:expr) => {{
        $sb.focus($css)
    }};
}
```

## Adding an MCP Tool

The MCP server in `src/bin/mcp_server.rs` uses the `rmcp` crate.

1. Add the tool schema to the `tools()` function. Keep schemas minimal and typed.
2. Add a handler branch in `call_tool()`.
3. Add a unit test in the same file or in `tests/` that exercises the tool via an in-memory client.
4. Document the tool in `docs/help/mcp_server.md` and the README tool table.

## Writing Documentation

- All public items should have doc comments (`///`).
- Examples in doc comments are run as doctests when possible; use `no_run` or `ignore` for snippets that need a browser.
- User-facing tutorials go in `docs/tutorials/`.
- Reference/help pages go in `docs/help/`.
- After adding a page, add it to `docs/SUMMARY.md` so the mdBook includes it.
- Build the book locally before pushing:

```bash
mdbook build
```

## Testing

- Add unit tests for pure logic (selectors, fingerprint generation, macros, providers).
- Use `#[tokio::test]` for async helpers that do not require a browser.
- Browser-backed tests should live in `examples/` or behind a feature flag so CI stays fast.
- Run the full verification matrix before opening a PR:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --features s3,azure,gcp,playwright,mcp-server -- -D warnings
cargo test --features mcp-server
mdbook build
```

## Opening a Pull Request

1. Branch from the current feature branch (`mustcodeal-rust-port`) unless told otherwise.
2. Keep commits focused and atomic; include the standard `Co-authored-by: Copilot App <223556219+Copilot@users.noreply.github.com>` trailer only if Copilot assisted.
3. Fill out the PR description with:
   - What changed and why.
   - Which verification commands passed.
   - Any breaking API changes.
4. Ensure CI is green.

## Licensing

By contributing, you agree that your contributions will be licensed under the same license as the project (MIT).
