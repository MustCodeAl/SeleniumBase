//! Convenience macros for writing SeleniumBase Rust tests.
//!
//! These macros reduce boilerplate for common patterns such as constructing a
//! selector or declaring an async browser test that creates and tears down a
//! [`BaseCase`].

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
///
/// The body must return `Result<(), seleniumbase_rs::SeleniumBaseError>` so that
/// the `?` operator works and the final result can be checked before quitting.
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

/// A short-hand macro for asserting that an element is visible.
///
/// # Examples
///
/// ```ignore
/// use seleniumbase_rs::assert_visible;
///
/// assert_visible!(sb, "#success", "success message should be visible");
/// ```
#[macro_export]
macro_rules! assert_visible {
    ($sb:expr, $selector:expr $(, $msg:expr)?) => {
        $sb.assert_element_visible($selector).await $(.expect($msg))?
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
}
