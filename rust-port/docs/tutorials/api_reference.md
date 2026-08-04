# BaseCase API Reference

`BaseCase` is the main entry point for writing browser automation in
`seleniumbase-rs`. It wraps a WebDriver/CDP session, provides a fluent API for
navigation, interaction, waits, assertions, and exposes advanced helpers for
stealth, CDP, tours, and screenshots.

This page is a high-level reference. For exact signatures and generic bounds,
build the rustdocs with:

```bash
cargo doc --no-deps --open
```

## What you will learn

- The lifecycle methods for creating and closing sessions.
- Common navigation, interaction, and query methods.
- Wait and assertion patterns.
- CDP/UC helpers and when to use them.
- Where to find rustdocs for deeper detail.

## Lifecycle

| Method | Description |
|--------|-------------|
| `BaseCase::new(config).await` | Creates and connects a browser session from a `BrowserConfig`. |
| `BaseCase::without_session(config)` | Creates a `BaseCase` without connecting a driver (used by some runners). |
| `sb.quit().await` | Closes the browser and ends the session. |
| `sb.reconnect().await` | Replaces the WebDriver session while keeping the driver process. |
| `sb.restart().await` | Quits and restarts the browser with the same config. |

Always call `quit()` in a `Drop` guard, `sb_test!` macro, or `finally` block so
browser processes do not leak when a test fails.

## Navigation

| Method | Description |
|--------|-------------|
| `open(url)` | Navigates to `url`. |
| `go_back()` | Browser back. |
| `go_forward()` | Browser forward. |
| `refresh_page()` | Reloads the page. |
| `get_current_url()` | Returns the current URL. |
| `get_title()` | Returns the page title. |
| `get_html_source()` | Returns the full page source. |

## Interaction

| Method | Description |
|--------|-------------|
| `click(selector)` | Clicks the element. |
| `double_click(selector)` | Double-clicks the element. |
| `context_click(selector)` | Right-clicks the element. |
| `type_text(selector, text)` | Clears and types. |
| `add_text(selector, text)` | Appends text. |
| `clear(selector)` | Clears the element. |
| `submit(selector)` | Submits the parent form. |
| `hover(selector)` | Hovers over the element. |
| `drag_and_drop(src, dst)` | Drags one element onto another. |
| `select_option_by_text(selector, text)` | Selects by visible text. |
| `select_option_by_value(selector, value)` | Selects by value. |
| `select_option_by_index(selector, index)` | Selects by zero-based index. |
| `js_click(selector)` | Clicks via JavaScript. |
| `js_type(selector, text)` | Types via JavaScript. |
| `highlight_click(selector)` | Highlights then clicks. |
| `highlight_type(selector, text)` | Highlights then types. |

All `selector` arguments accept the same formats described in the
[Selectors guide](./selectors.md).

## Waits

| Method | Description |
|--------|-------------|
| `wait_for_element_present(selector, timeout)` | Waits for element in DOM. |
| `wait_for_element_visible(selector, timeout)` | Waits for element visible. |
| `wait_for_element_not_visible(selector, timeout)` | Waits for element invisible. |
| `wait_for_element_clickable(selector, timeout)` | Waits for element clickable. |
| `wait_for_element_absent(selector, timeout)` | Waits for element removed. |
| `wait_for_text_visible(text, selector, timeout)` | Waits for text to appear. |
| `wait_for_ready_state_complete()` | Waits for `document.readyState === 'complete'`. |
| `set_time_limit(seconds)` | Caps wait timeouts globally. |

See [Waits and Assertions](./waits_assertions.md) for detailed examples.

## Assertions

| Method | Description |
|--------|-------------|
| `assert_title(expected)` | Asserts page title equals `expected`. |
| `assert_text_visible(text, selector)` | Asserts text visible. |
| `assert_text_not_visible(text, selector)` | Asserts text not visible. |
| `assert_element_visible(selector)` | Asserts element is visible. |
| `assert_element_not_visible(selector)` | Asserts element is not visible. |
| `assert_attribute(selector, attr, value)` | Asserts attribute value. |
| `assert_url_contains(fragment)` | Asserts current URL contains fragment. |
| `assert_no_404_errors()` | Fails if any same-origin link returns HTTP 404. |

## Queries

| Method | Description |
|--------|-------------|
| `is_element_present(selector)` | Returns true if in DOM. |
| `is_element_visible(selector)` | Returns true if visible. |
| `is_element_enabled(selector)` | Returns true if enabled. |
| `is_element_selected(selector)` | Returns true if selected. |
| `is_text_visible(text, selector)` | Returns true if text visible. |
| `get_text(selector)` | Returns element text. |
| `get_attribute(selector, attr)` | Returns attribute value. |
| `get_property(selector, prop)` | Returns property value. |
| `get_shadow_root(selector)` | Returns the shadow root. |
| `find_element(selector)` | Returns a `WebElement` handle. |
| `find_elements(selector)` | Returns all matching `WebElement` handles. |

## Windows and frames

| Method | Description |
|--------|-------------|
| `switch_to_frame(selector)` | Enters a frame. |
| `switch_to_default_content()` | Returns to top document. |
| `switch_to_parent_frame()` | Goes to parent frame. |
| `switch_to_window(handle)` | Switches to a window. |
| `switch_to_new_window()` | Opens and switches to new window. |
| `close_window()` | Closes current window. |
| `maximize_window()` | Maximizes window. |
| `minimize_window()` | Minimizes window. |
| `set_window_size(w, h)` | Resizes window. |
| `set_window_position(x, y)` | Moves window. |
| `set_window_rect(x, y, w, h)` | Sets position and size. |

## Cookies and storage

| Method | Description |
|--------|-------------|
| `save_cookies(path)` | Saves cookies to JSON. |
| `load_cookies(path)` | Loads cookies from JSON. |
| `set_local_storage_item(key, value)` | Sets localStorage item. |
| `get_local_storage_item(key)` | Gets localStorage item. |
| `remove_local_storage_item(key)` | Removes localStorage item. |
| `clear_local_storage()` | Clears localStorage. |

## Screenshots

| Method | Description |
|--------|-------------|
| `save_screenshot(path)` | Writes screenshot to the logs directory. |
| `save_screenshot_to_path(path)` | Writes screenshot to an arbitrary path. |
| `screenshot_as_png()` | Returns the current page screenshot as PNG bytes. |
| `check_window(name, level)` | Compares a screenshot to a stored baseline. |

## CDP / UC helpers

| Method | Description |
|--------|-------------|
| `activate_cdp_mode()` | Enables CDP domains. |
| `execute_cdp(method)` | Sends a CDP command. |
| `execute_cdp_with_params(method, params)` | Sends a CDP command with params. |
| `cdp_mouse_click(x, y)` | CDP mouse click. |
| `cdp_type_text(text)` | CDP text insert. |
| `cdp_click_element(selector)` | CDP click at element center. |
| `clear_browser_cache()` | Clears cache. |
| `clear_browser_cookies()` | Clears cookies. |
| `get_cookies()` | Returns cookies as JSON. |
| `set_network_conditions(conditions)` | Network throttling. |
| `set_timezone(id)` | Sets timezone. |
| `set_geolocation(lat, lon, acc)` | Sets geolocation. |
| `uc_click(selector)` | Stealth click with delay. |
| `uc_type(selector, text)` | Stealth type with delay. |
| `human_click(selector)` | Human-like click. |
| `human_type(selector, text)` | Human-like type. |

See the [CDP Mode](./cdp_mode.md) and [UC Mode](./uc_mode.md) tutorials for
end-to-end examples.

## Tours, presentations, charts

| Method | Description |
|--------|-------------|
| `create_tour(name)` | Creates a tour with the default theme. |
| `create_tour_with_theme(name, theme)` | Creates a tour with a specific `TourTheme`. |
| `create_shepherd_tour(name)` / `create_introjs_tour(name)` / `create_driverjs_tour(name)` / `create_bootstrap_tour(name)` / `create_hopscotch_tour(name)` | Convenience themed constructors. |
| `add_tour_step(message, target)` | Adds a tour step; `target` is an optional CSS selector. |
| `play_tour()` / `start_tour()` | Plays the tour in the current page. |
| `export_tour(path)` | Exports tour HTML. |
| `create_presentation(name)` | Creates HTML presentation. |
| `add_presentation_slide(html)` | Adds a slide. |
| `save_presentation(path)` | Saves presentation. |
| `create_pie_chart(name)` / `create_bar_chart(name)` / ... | Creates a chart. |
| `add_data_point(label, value)` | Adds data to the current chart. |
| `save_chart(path)` | Saves chart HTML. |

See the [Tours](./tours.md) and [Charts](./charts.md) pages for details.

## Recorder

| Method | Description |
|--------|-------------|
| `activate_recorder()` | Injects the browser-side action recorder. |
| `recorded_actions()` | Returns actions captured so far. |
| `export_recording_as_rust()` | Returns recorded actions as Rust source. |
| `save_recording_to_logs()` | Saves the recording as JSON and Rust source. |
| `save_recorded_actions(path)` | Saves recorded actions to a JSON file. |

## Common error handling pattern

Most `BaseCase` methods return `Result<T, SeleniumBaseError>`. Use `?` inside an
async function or the `sb_test!` macro:

```rust
use seleniumbase_rs::{BaseCase, BrowserConfig};

async fn login(sb: &mut BaseCase) -> Result<(), seleniumbase_rs::SeleniumBaseError> {
    sb.open("https://example.com/login").await?;
    sb.type_text("#username", "alice").await?;
    sb.type_text("#password", "secret").await?;
    sb.click("#submit").await?;
    sb.assert_element_visible("#dashboard").await?;
    Ok(())
}
```

## Further reading

- [Selectors](./selectors.md)
- [Waits and Assertions](./waits_assertions.md)
- [CDP Mode](./cdp_mode.md)
- [UC Mode](./uc_mode.md)
- [Macros](./macros.md)

For exact signatures, run `cargo doc --no-deps --open`.
