//! Convenience macros for writing SeleniumBase Rust tests.
//!
//! These macros reduce boilerplate for common patterns such as constructing a
//! selector, driving a [`BaseCase`], or building a [`Fingerprint`]. They are
//! intended to be used inside `async` test functions (for example those created
//! with [`sb_test!`]).

/// Builds a [`Selector`](crate::Selector) variant at compile time.
///
/// # Examples
///
/// ```ignore
/// use seleniumbase_rs::{selector, Selector};
///
/// let s = selector!(css, "#login");
/// assert_eq!(s, Selector::Css("#login"));
///
/// let s = selector!(xpath, "//button[text()='Go']");
/// let s = selector!(link, "Click here");
/// let s = selector!(partial_link, "here");
/// let s = selector!(id, "username");
/// ```
#[macro_export]
macro_rules! selector {
    (css, $value:expr) => {
        $crate::Selector::Css($value)
    };
    (xpath, $value:expr) => {
        $crate::Selector::XPath($value)
    };
    (id, $value:expr) => {
        $crate::Selector::Id($value)
    };
    (link, $value:expr) => {
        $crate::Selector::LinkText($value)
    };
    (partial_link, $value:expr) => {
        $crate::Selector::PartialLinkText($value)
    };
}

/// Declares an async browser test with automatic `BaseCase` setup and teardown.
///
/// The macro expands to a `#[tokio::test]` async function that creates a
/// [`BaseCase`](crate::BaseCase) from the supplied [`BrowserConfig`](crate::BrowserConfig),
/// runs the provided body, and finally calls [`quit`](crate::BaseCase::quit)
/// even when assertions fail.
///
/// # Examples
///
/// ```ignore
/// use seleniumbase_rs::{sb_test, BrowserConfig};
///
/// sb_test!(login_flow, BrowserConfig::default(), |sb| {
///     sb.open("https://example.com/login").await?;
///     sb.type_text("#user", "alice").await?;
///     sb.click("#submit").await?;
///     Ok(())
/// });
/// ```
#[macro_export]
macro_rules! sb_test {
    ($name:ident, $config:expr, |$sb:ident| $body:expr) => {
        #[tokio::test]
        async fn $name() {
            let mut $sb = $crate::BaseCase::new($config)
                .await
                .expect("failed to create BaseCase");
            let result: Result<(), $crate::SeleniumBaseError> = async { $body }.await;
            $sb.quit().await.expect("failed to quit BaseCase");
            result.expect("test failed");
        }
    };
}

/// Opens a URL inside an async test.
///
/// ```ignore
/// sb_open!(sb, "https://example.com");
/// ```
#[macro_export]
macro_rules! sb_open {
    ($sb:expr, $url:expr $(, $msg:expr)?) => {
        $sb.open($url).await $(.expect($msg))?
    };
}

/// Clicks an element inside an async test.
///
/// ```ignore
/// sb_click!(sb, "#submit");
/// ```
#[macro_export]
macro_rules! sb_click {
    ($sb:expr, $selector:expr $(, $msg:expr)?) => {
        $sb.click($selector).await $(.expect($msg))?
    };
}

/// Types text into an element inside an async test.
///
/// ```ignore
/// sb_type!(sb, "#username", "alice");
/// ```
#[macro_export]
macro_rules! sb_type {
    ($sb:expr, $selector:expr, $text:expr $(, $msg:expr)?) => {
        $sb.type_text($selector, $text).await $(.expect($msg))?
    };
}

/// Hovers over an element inside an async test.
#[macro_export]
macro_rules! sb_hover {
    ($sb:expr, $selector:expr $(, $msg:expr)?) => {
        $sb.hover($selector).await $(.expect($msg))?
    };
}

/// Scrolls to an element inside an async test.
#[macro_export]
macro_rules! sb_scroll_to {
    ($sb:expr, $selector:expr $(, $msg:expr)?) => {
        $sb.scroll_to($selector).await $(.expect($msg))?
    };
}

/// Waits for an element to be visible.
#[macro_export]
macro_rules! sb_wait_for {
    ($sb:expr, $selector:expr $(, $msg:expr)?) => {
        $sb.wait_for_element_visible($selector).await $(.expect($msg))?
    };
}

/// Selects an `<option>` by visible text.
#[macro_export]
macro_rules! sb_select {
    ($sb:expr, $selector:expr, $text:expr $(, $msg:expr)?) => {
        $sb.select_option_by_text($selector, $text).await $(.expect($msg))?
    };
}

/// Asserts that an element contains the expected text.
#[macro_export]
macro_rules! sb_assert_text {
    ($sb:expr, $selector:expr, $expected:expr $(, $msg:expr)?) => {
        $sb.assert_text($selector, $expected).await $(.expect($msg))?
    };
}

/// Asserts that the page title contains the expected text.
#[macro_export]
macro_rules! sb_assert_title {
    ($sb:expr, $expected:expr $(, $msg:expr)?) => {
        $sb.assert_title_contains($expected).await $(.expect($msg))?
    };
}

/// Asserts that the current URL contains the expected substring.
#[macro_export]
macro_rules! sb_assert_url {
    ($sb:expr, $expected:expr $(, $msg:expr)?) => {
        $sb.assert_url_contains($expected).await $(.expect($msg))?
    };
}

/// Takes a screenshot and saves it to the supplied path.
#[macro_export]
macro_rules! sb_screenshot {
    ($sb:expr, $path:expr $(, $msg:expr)?) => {
        $sb.save_screenshot_to_path($path).await $(.expect($msg))?
    };
}

/// Executes JavaScript and returns the result.
#[macro_export]
macro_rules! sb_js {
    ($sb:expr, $script:expr $(, $msg:expr)?) => {
        $sb.execute_script($script).await $(.expect($msg))?
    };
}

/// Quits the browser session.
#[macro_export]
macro_rules! sb_quit {
    ($sb:expr $(, $msg:expr)?) => {
        $sb.quit().await $(.expect($msg))?
    };
}

/// A short-hand macro for asserting that an element is visible.
#[macro_export]
macro_rules! assert_visible {
    ($sb:expr, $selector:expr $(, $msg:expr)?) => {
        $sb.assert_element_visible($selector).await $(.expect($msg))?
    };
}

/// Builds a [`Fingerprint`](crate::Fingerprint) preset.
///
/// # Examples
///
/// ```ignore
/// use seleniumbase_rs::fingerprint;
///
/// let fp = fingerprint!(windows);
/// let fp = fingerprint!(macos);
/// let fp = fingerprint!(android);
/// ```
#[macro_export]
macro_rules! fingerprint {
    (windows) => {
        $crate::Fingerprint::windows_desktop()
    };
    (macos) => {
        $crate::Fingerprint::macos_desktop()
    };
    (android) => {
        $crate::Fingerprint::android_mobile()
    };
}

/// Builds a [`BrowserConfig`](crate::BrowserConfig) with UC mode enabled.
///
/// ```ignore
/// use seleniumbase_rs::uc_config;
///
/// let config = uc_config!();
/// ```
#[macro_export]
macro_rules! uc_config {
    () => {
        $crate::BrowserConfig::default().with_mode($crate::DriverMode::Uc)
    };
}

#[cfg(test)]
mod tests {
    use crate::Selector;

    #[test]
    fn selector_macro_css() {
        assert_eq!(selector!(css, "#id"), Selector::Css("#id"));
    }

    #[test]
    fn selector_macro_xpath() {
        assert_eq!(
            selector!(xpath, "//div[@class='x']"),
            Selector::XPath("//div[@class='x']")
        );
    }

    #[test]
    fn selector_macro_link_text() {
        assert_eq!(selector!(link, "Home"), Selector::LinkText("Home"));
    }

    #[test]
    fn selector_macro_partial_link_text() {
        assert_eq!(
            selector!(partial_link, "Hom"),
            Selector::PartialLinkText("Hom")
        );
    }

    #[test]
    fn selector_macro_id() {
        assert_eq!(selector!(id, "user"), Selector::Id("user"));
    }

    #[test]
    fn fingerprint_preset_macro() {
        let fp = fingerprint!(windows);
        assert_eq!(fp.os_type, crate::stealth::fingerprint::OsType::Windows);
        assert!(fp.user_agent.as_ref().unwrap().contains("Windows"));
    }
}
