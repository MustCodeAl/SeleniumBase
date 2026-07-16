use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::artifacts::{artifact_path, ensure_latest_logs_dir};
use crate::config::BrowserConfig;
use crate::core::selectors::Selector;
use crate::core::session::BrowserSession;
use crate::error::SeleniumBaseError;
use crate::recorder::{ActionRecorder, RecordedAction};
use serde_json::Value;
use thirtyfour::extensions::cdp::NetworkConditions;

pub struct BaseCase {
    session: BrowserSession,
    recorder: Arc<Mutex<ActionRecorder>>,
}

impl BaseCase {
    pub async fn new(config: BrowserConfig) -> Result<Self, SeleniumBaseError> {
        let session = BrowserSession::connect(config).await?;
        Ok(Self {
            session,
            recorder: Arc::new(Mutex::new(ActionRecorder::default())),
        })
    }

    pub async fn open(&mut self, url: &str) -> Result<(), SeleniumBaseError> {
        self.record("open", Some(url), None);
        self.session.goto(url).await
    }

    pub async fn refresh(&self) -> Result<(), SeleniumBaseError> {
        self.session.refresh().await
    }

    pub async fn go_back(&self) -> Result<(), SeleniumBaseError> {
        self.session.back().await
    }

    pub async fn go_forward(&self) -> Result<(), SeleniumBaseError> {
        self.session.forward().await
    }

    pub async fn click(&mut self, css: &str) -> Result<(), SeleniumBaseError> {
        let by = Selector::Css(css).to_by()?;
        self.record("click", Some(css), None);
        self.session.click(by).await
    }

    pub async fn type_text(&mut self, css: &str, text: &str) -> Result<(), SeleniumBaseError> {
        let by = Selector::Css(css).to_by()?;
        self.record("type_text", Some(css), Some(text));
        self.session.type_text(by, text).await
    }

    pub async fn clear(&mut self, css: &str) -> Result<(), SeleniumBaseError> {
        let by = Selector::Css(css).to_by()?;
        self.record("clear", Some(css), None);
        self.session.clear(by).await
    }

    pub async fn click_link_text(&mut self, link_text: &str) -> Result<(), SeleniumBaseError> {
        let by = thirtyfour::prelude::By::LinkText(link_text.to_owned());
        self.record("click_link_text", Some(link_text), None);
        self.session.click(by).await
    }

    pub async fn submit(&mut self, css: &str) -> Result<(), SeleniumBaseError> {
        let by = Selector::Css(css).to_by()?;
        self.record("submit", Some(css), None);
        self.session.submit(by).await
    }

    pub async fn get_text(&mut self, css: &str) -> Result<String, SeleniumBaseError> {
        let by = Selector::Css(css).to_by()?;
        self.session.text(by).await
    }

    pub async fn hover(&mut self, css: &str) -> Result<(), SeleniumBaseError> {
        let by = Selector::Css(css).to_by()?;
        self.record("hover", Some(css), None);
        self.session.hover(by).await
    }

    pub async fn hover_and_click(
        &mut self,
        hover_css: &str,
        click_css: &str,
    ) -> Result<(), SeleniumBaseError> {
        self.hover(hover_css).await?;
        self.sleep(0.1).await;
        self.click(click_css).await
    }

    pub async fn select_option_by_text(
        &mut self,
        css: &str,
        text: &str,
    ) -> Result<(), SeleniumBaseError> {
        let by = Selector::Css(css).to_by()?;
        self.record("select_option_by_text", Some(css), Some(text));
        self.session.select_option_by_text(by, text).await
    }

    pub async fn select_option_by_value(
        &mut self,
        css: &str,
        value: &str,
    ) -> Result<(), SeleniumBaseError> {
        let by = Selector::Css(css).to_by()?;
        self.record("select_option_by_value", Some(css), Some(value));
        self.session.select_option_by_value(by, value).await
    }

    pub async fn switch_to_frame(&mut self, css: &str) -> Result<(), SeleniumBaseError> {
        let by = Selector::Css(css).to_by()?;
        self.record("switch_to_frame", Some(css), None);
        self.session.switch_to_frame(by).await
    }

    pub async fn switch_to_default_content(&mut self) -> Result<(), SeleniumBaseError> {
        self.record("switch_to_default_content", None, None);
        self.session.switch_to_default_content().await
    }

    pub async fn drag_and_drop(
        &mut self,
        source_css: &str,
        target_css: &str,
    ) -> Result<(), SeleniumBaseError> {
        let source_by = Selector::Css(source_css).to_by()?;
        let target_by = Selector::Css(target_css).to_by()?;
        self.record("drag_and_drop", Some(source_css), Some(target_css));
        self.session.drag_and_drop(source_by, target_by).await
    }

    pub async fn is_element_present(&self, css: &str) -> Result<bool, SeleniumBaseError> {
        let by = Selector::Css(css).to_by()?;
        self.session.element_present(by).await
    }

    pub async fn assert_element(&self, css: &str) -> Result<(), SeleniumBaseError> {
        if self.is_element_present(css).await? {
            return Ok(());
        }
        Err(SeleniumBaseError::AssertionFailed(format!(
            "expected element '{css}' to be present"
        )))
    }

    pub async fn assert_element_absent(&self, css: &str) -> Result<(), SeleniumBaseError> {
        if !self.is_element_present(css).await? {
            return Ok(());
        }
        Err(SeleniumBaseError::AssertionFailed(format!(
            "expected element '{css}' to be absent"
        )))
    }

    pub async fn get_title(&mut self) -> Result<String, SeleniumBaseError> {
        self.session.current_title().await
    }

    pub async fn get_current_url(&mut self) -> Result<String, SeleniumBaseError> {
        self.session.current_url().await
    }

    pub async fn assert_url_contains(&mut self, expected: &str) -> Result<(), SeleniumBaseError> {
        let url = self.get_current_url().await?;
        if url.contains(expected) {
            return Ok(());
        }
        Err(SeleniumBaseError::AssertionFailed(format!(
            "expected URL to contain '{expected}', got '{url}'"
        )))
    }

    pub async fn get_page_source(&self) -> Result<String, SeleniumBaseError> {
        self.session.page_source().await
    }

    pub async fn execute_script(&self, script: &str) -> Result<Value, SeleniumBaseError> {
        self.session.execute_script(script).await
    }

    pub async fn activate_cdp_mode(&self) -> Result<(), SeleniumBaseError> {
        self.session.activate_cdp_mode().await
    }

    pub async fn execute_cdp(&self, method: &str) -> Result<Value, SeleniumBaseError> {
        self.session.execute_cdp(method).await
    }

    pub async fn execute_cdp_with_params(
        &self,
        method: &str,
        params: Value,
    ) -> Result<Value, SeleniumBaseError> {
        self.session.execute_cdp_with_params(method, params).await
    }

    pub async fn clear_browser_cache(&self) -> Result<(), SeleniumBaseError> {
        self.session.clear_browser_cache().await
    }

    pub async fn set_network_conditions(
        &self,
        conditions: &NetworkConditions,
    ) -> Result<(), SeleniumBaseError> {
        self.session.set_network_conditions(conditions).await
    }

    pub async fn wait_for_element(
        &self,
        css: &str,
        timeout_secs: u64,
    ) -> Result<(), SeleniumBaseError> {
        let by = Selector::Css(css).to_by()?;
        self.session.wait_for_element(by, timeout_secs).await?;
        Ok(())
    }

    pub async fn get_attribute(&mut self, css: &str, attribute_name: &str) -> Result<Option<String>, SeleniumBaseError> {
        let by = Selector::Css(css).to_by()?;
        self.session.get_attribute(by, attribute_name).await
    }

    pub async fn get_property(&mut self, css: &str, property_name: &str) -> Result<Option<String>, SeleniumBaseError> {
        let by = Selector::Css(css).to_by()?;
        self.session.get_property(by, property_name).await
    }

    pub async fn wait_for_element_visible(
        &self,
        css: &str,
        timeout_secs: u64,
    ) -> Result<(), SeleniumBaseError> {
        let by = Selector::Css(css).to_by()?;
        self.session.wait_for_element_visible(by, timeout_secs).await?;
        Ok(())
    }

    pub async fn wait_for_element_absent(
        &self,
        css: &str,
        timeout_secs: u64,
    ) -> Result<(), SeleniumBaseError> {
        let by = Selector::Css(css).to_by()?;
        self.session.wait_for_element_absent(by, timeout_secs).await
    }

    pub async fn wait_for_text(
        &self,
        css: &str,
        expected_substring: &str,
        timeout_secs: u64,
    ) -> Result<(), SeleniumBaseError> {
        let by = Selector::Css(css).to_by()?;
        self.session
            .wait_for_text(by, expected_substring, timeout_secs)
            .await
    }

    pub async fn assert_title_contains(&mut self, expected: &str) -> Result<(), SeleniumBaseError> {
        let title = self.get_title().await?;
        if title.contains(expected) {
            return Ok(());
        }
        Err(SeleniumBaseError::AssertionFailed(format!(
            "expected title to contain '{expected}', got '{title}'"
        )))
    }

    pub async fn assert_text(
        &mut self,
        css: &str,
        expected: &str,
    ) -> Result<(), SeleniumBaseError> {
        self.record("assert_text", Some(css), Some(expected));
        let by = Selector::Css(css).to_by()?;
        let text = self.session.text(by).await?;
        if text.contains(expected) {
            return Ok(());
        }
        Err(SeleniumBaseError::AssertionFailed(format!(
            "expected text '{expected}' in element '{css}', got '{text}'"
        )))
    }

    pub async fn assert_exact_text(
        &mut self,
        css: &str,
        expected: &str,
    ) -> Result<(), SeleniumBaseError> {
        let text = self.get_text(css).await?;
        if text == expected {
            return Ok(());
        }
        Err(SeleniumBaseError::AssertionFailed(format!(
            "expected exact text '{expected}' in element '{css}', got '{text}'"
        )))
    }

    pub async fn highlight(&self, css: &str) -> Result<(), SeleniumBaseError> {
        let script = format!(
            "var e=document.querySelector({q}); if(e){{e.style.outline='3px solid magenta'; e.style.outlineOffset='2px';}}",
            q = serde_json::to_string(css).map_err(|e| {
                SeleniumBaseError::InvalidSelector(format!("failed to escape selector: {e}"))
            })?
        );
        self.execute_script(&script).await?;
        Ok(())
    }

    pub async fn post_message(
        &self,
        message: &str,
        duration_secs: u64,
    ) -> Result<(), SeleniumBaseError> {
        let escaped = serde_json::to_string(message).map_err(|e| {
            SeleniumBaseError::InvalidSelector(format!("failed to escape message: {e}"))
        })?;
        let script = format!(
            "(() => {{
                const id='sb-rs-msg';
                let box=document.getElementById(id);
                if(!box) {{
                  box=document.createElement('div');
                  box.id=id;
                  box.style.position='fixed';
                  box.style.top='16px';
                  box.style.right='16px';
                  box.style.zIndex='2147483647';
                  box.style.background='rgba(20,20,20,0.92)';
                  box.style.color='#fff';
                  box.style.padding='10px 14px';
                  box.style.borderRadius='8px';
                  box.style.fontFamily='Arial,sans-serif';
                  box.style.fontSize='14px';
                  document.body.appendChild(box);
                }}
                box.textContent={msg};
                setTimeout(() => {{ if (box && box.parentNode) box.parentNode.removeChild(box); }}, {ms});
            }})();",
            msg = escaped,
            ms = duration_secs.saturating_mul(1000),
        );
        self.execute_script(&script).await?;
        Ok(())
    }

    pub async fn save_screenshot<P: AsRef<Path>>(&self, path: P) -> Result<(), SeleniumBaseError> {
        self.session.screenshot(path.as_ref()).await
    }

    pub async fn save_page_source<P: AsRef<Path>>(&self, path: P) -> Result<(), SeleniumBaseError> {
        let html = self.get_page_source().await?;
        std::fs::write(path.as_ref(), html).map_err(|e| {
            SeleniumBaseError::InvalidConfig(format!(
                "failed to write page source '{}': {e}",
                path.as_ref().display()
            ))
        })?;
        Ok(())
    }

    pub async fn save_screenshot_to_logs(&self) -> Result<PathBuf, SeleniumBaseError> {
        let dir = ensure_latest_logs_dir()?;
        let path = artifact_path(&dir, "screenshot", "png");
        self.save_screenshot(&path).await?;
        Ok(path)
    }

    pub async fn save_page_source_to_logs(&self) -> Result<PathBuf, SeleniumBaseError> {
        let dir = ensure_latest_logs_dir()?;
        let path = artifact_path(&dir, "page_source", "html");
        self.save_page_source(&path).await?;
        Ok(path)
    }

    pub fn recorded_actions(&self) -> Result<Vec<RecordedAction>, SeleniumBaseError> {
        let recorder = self
            .recorder
            .lock()
            .map_err(|_| SeleniumBaseError::Unsupported("recorder mutex poisoned".to_owned()))?;
        Ok(recorder.actions.clone())
    }

    pub fn export_recording_as_rust(&self) -> Result<String, SeleniumBaseError> {
        let recorder = self
            .recorder
            .lock()
            .map_err(|_| SeleniumBaseError::Unsupported("recorder mutex poisoned".to_owned()))?;
        Ok(recorder.to_rust_script())
    }

    pub fn save_recording_to_logs(&self) -> Result<(PathBuf, PathBuf), SeleniumBaseError> {
        let dir = ensure_latest_logs_dir()?;
        let json_path = artifact_path(&dir, "recording", "json");
        let rust_path = artifact_path(&dir, "recording", "rs");

        let actions = self.recorded_actions()?;
        let json_data = serde_json::to_string_pretty(&actions).map_err(|e| {
            SeleniumBaseError::InvalidConfig(format!("failed to serialize recording json: {e}"))
        })?;
        std::fs::write(&json_path, json_data).map_err(|e| {
            SeleniumBaseError::InvalidConfig(format!("failed to write recording json: {e}"))
        })?;

        let rust_script = self.export_recording_as_rust()?;
        std::fs::write(&rust_path, rust_script).map_err(|e| {
            SeleniumBaseError::InvalidConfig(format!("failed to write recording rust file: {e}"))
        })?;

        Ok((json_path, rust_path))
    }

    pub async fn sleep(&self, seconds: f64) {
        let millis = if seconds <= 0.0 {
            0_u64
        } else {
            (seconds * 1000.0) as u64
        };
        tokio::time::sleep(Duration::from_millis(millis)).await;
    }

    pub async fn quit(self) -> Result<(), SeleniumBaseError> {
        self.session.quit().await
    }

    fn record(&self, name: &str, target: Option<&str>, value: Option<&str>) {
        if let Ok(mut recorder) = self.recorder.lock() {
            recorder.record(name, target, value);
        }
    }
}
