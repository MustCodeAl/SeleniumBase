# Macros

`seleniumbase-rs` exports a set of convenience macros from the crate root. They
reduce boilerplate for selectors, test setup, common interactions, and
assertions. Because the action macros expand `.await` internally, use them
inside an `async` function or closure.

```rust
use seleniumbase_rs::{
    assert_visible, fingerprint, sb_assert_text, sb_assert_title, sb_assert_url,
    sb_click, sb_hover, sb_js, sb_open, sb_quit, sb_scroll_to, sb_select,
    sb_screenshot, sb_test, sb_type, sb_wait_for, selector, uc_config,
};
```

## `selector!`

Builds a [`Selector`](crate::Selector) variant at compile time.

```rust
use seleniumbase_rs::{selector, Selector};

let s = selector!(css, "#submit");
assert_eq!(s, Selector::Css("#submit"));

let s = selector!(xpath, "//button[text()='Go']");
let s = selector!(id, "username");
let s = selector!(link, "Home");
let s = selector!(partial_link, "Terms");
```

## `sb_test!`

Generates a `#[tokio::test]` async function that creates a [`BaseCase`](crate::BaseCase),
runs the supplied closure, and always calls `quit()` before returning. The test
fails if the closure returns an `Err`.

```rust,ignore
use seleniumbase_rs::sb_test;

sb_test!(visit_example, seleniumbase_rs::BrowserConfig::default(), |sb| {
    sb.open("https://example.com").await?;
    sb.assert_title("Example Domain").await?;
    Ok(())
});
```

The closure receives `&mut BaseCase` and must return `Result<(), E>` where `E`
converts into `SeleniumBaseError` (for example, via `?` on crate methods).

## `uc_config!`

Returns a [`BrowserConfig`](crate::BrowserConfig) with UC mode enabled.

```rust
use seleniumbase_rs::uc_config;

let config = uc_config!();
```

## `fingerprint!`

Returns a built-in [`Fingerprint`](crate::Fingerprint) preset.

```rust
use seleniumbase_rs::fingerprint;

let fp = fingerprint!(windows);
let fp = fingerprint!(macos);
let fp = fingerprint!(android);
```

## Action macros

These macros call the corresponding [`BaseCase`](crate::BaseCase) method and
await it. An optional trailing message is passed to `.expect(...)`.

| Macro | Calls |
|---|---|
| `sb_open!(sb, url)` | `sb.open(url).await` |
| `sb_click!(sb, selector)` | `sb.click(selector).await` |
| `sb_type!(sb, selector, text)` | `sb.type_text(selector, text).await` |
| `sb_hover!(sb, selector)` | `sb.hover(selector).await` |
| `sb_scroll_to!(sb, selector)` | `sb.scroll_to(selector).await` |
| `sb_wait_for!(sb, selector)` | `sb.wait_for_element_visible(selector).await` |
| `sb_select!(sb, selector, text)` | `sb.select_option_by_text(selector, text).await` |
| `sb_assert_text!(sb, selector, expected)` | `sb.assert_text(selector, expected).await` |
| `sb_assert_title!(sb, expected)` | `sb.assert_title_contains(expected).await` |
| `sb_assert_url!(sb, expected)` | `sb.assert_url_contains(expected).await` |
| `sb_screenshot!(sb, path)` | `sb.save_screenshot_to_path(path).await` |
| `sb_js!(sb, script)` | `sb.execute_script(script).await` |
| `sb_quit!(sb)` | `sb.quit().await` |
| `assert_visible!(sb, selector)` | `sb.assert_element_visible(selector).await` |

```rust,ignore
use seleniumbase_rs::{
    assert_visible, sb_click, sb_open, sb_quit, sb_screenshot, sb_test,
    sb_type, selector,
};

sb_test!(login_with_macros, seleniumbase_rs::BrowserConfig::default(), |sb| {
    sb_open!(sb, "https://example.com/login");
    sb_type!(sb, "#username", "alice");
    sb_type!(sb, "#password", "secret");
    sb_click!(sb, "#submit");
    assert_visible!(sb, "#dashboard");
    sb_screenshot!(sb, "dashboard.png");
    sb_quit!(sb);
    Ok(())
});
```

## When to use macros

Use macros for:

* Quick, linear test scripts where the shorter syntax improves readability.
* Compile-time selectors that never change.
* One-liner actions that would otherwise be dominated by `.await?` noise.

Prefer the explicit method API when you need fine-grained error handling,
custom timeouts, or non-trivial control flow.
