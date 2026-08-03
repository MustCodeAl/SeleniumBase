# Macros

`seleniumbase-rs` ships a small `macros` module that reduces boilerplate for
the most common testing tasks.

```rust
use seleniumbase_rs::macros::{selector, sb_test, assert_visible};
```

## `selector!`

Builds a `Selector` at compile time. It is easier to read than nested
constructors and works well in tables of test data.

```rust
use seleniumbase_rs::macros::selector;
use seleniumbase_rs::Selector;

let s = selector!(css, "#submit");
assert_eq!(s, Selector::Css("#submit"));

let s = selector!(xpath, "//button[text()='Go']");
let s = selector!(id, "username");
let s = selector!(link, "Home");
let s = selector!(partial_link, "Terms");
```

## `sb_test!`

Generates a `#[tokio::test]` async function that creates a `BaseCase`, runs your
body, and always calls `quit()` before returning. This avoids leaked browser
processes when an assertion fails.

```rust
use seleniumbase_rs::macros::sb_test;

sb_test!(visit_example, {
    sb.open("https://example.com").await?;
    sb.assert_title("Example Domain").await?;
});
```

The macro expands to:

```rust
#[tokio::test]
async fn visit_example() -> Result<(), Box<dyn std::error::Error>> {
    let mut sb = seleniumbase_rs::BaseCase::new(
        seleniumbase_rs::BrowserConfig::default()
    ).await?;
    let result = async {
        sb.open("https://example.com").await?;
        sb.assert_title("Example Domain").await?;
        Ok::<(), Box<dyn std::error::Error>>(())
    }.await;
    let _ = sb.quit().await;
    result
}
```

You can supply a custom config by ending the first argument with a trailing
expression:

```rust
sb_test!(stealth_example, seleniumbase_rs::BrowserConfig::default()
    .with_mode(seleniumbase_rs::DriverMode::Uc), {
    sb.open("https://example.com").await?;
});
```

## `assert_visible!`

Asserts that an element is present and visible, with a clearer panic message
than a manual chain.

```rust
use seleniumbase_rs::macros::assert_visible;

sb_test!(homepage_has_cta, {
    sb.open("https://example.com").await?;
    assert_visible!(sb, selector!(css, ".cta-button"));
});
```

The macro delegates to `BaseCase::assert_element_visible`, so it respects the
same wait and retry semantics as the rest of the API.
