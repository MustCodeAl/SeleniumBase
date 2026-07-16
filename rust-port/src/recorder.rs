use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct RecordedAction {
    pub name: String,
    pub target: Option<String>,
    pub value: Option<String>,
    pub timestamp_ms: u128,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct ActionRecorder {
    pub actions: Vec<RecordedAction>,
}

impl ActionRecorder {
    pub fn record(&mut self, name: &str, target: Option<&str>, value: Option<&str>) {
        let timestamp_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        self.actions.push(RecordedAction {
            name: name.to_owned(),
            target: target.map(ToOwned::to_owned),
            value: value.map(ToOwned::to_owned),
            timestamp_ms,
        });
    }

    pub fn to_rust_script(&self) -> String {
        let mut out = String::from(
            "use seleniumbase_rs::{BaseCase, BrowserConfig};\n\n#[tokio::main]\nasync fn main() -> Result<(), Box<dyn std::error::Error>> {\n    let mut sb = BaseCase::new(BrowserConfig::default()).await?;\n",
        );
        for action in &self.actions {
            match action.name.as_str() {
                "open" => {
                    if let Some(url) = action.target.as_deref() {
                        out.push_str(&format!("    sb.open({:?}).await?;\n", url));
                    }
                }
                "click" => {
                    if let Some(css) = action.target.as_deref() {
                        out.push_str(&format!("    sb.click({:?}).await?;\n", css));
                    }
                }
                "type_text" => {
                    if let (Some(css), Some(value)) =
                        (action.target.as_deref(), action.value.as_deref())
                    {
                        out.push_str(&format!(
                            "    sb.type_text({:?}, {:?}).await?;\n",
                            css, value
                        ));
                    }
                }
                "assert_text" => {
                    if let (Some(css), Some(value)) =
                        (action.target.as_deref(), action.value.as_deref())
                    {
                        out.push_str(&format!(
                            "    sb.assert_text({:?}, {:?}).await?;\n",
                            css, value
                        ));
                    }
                }
                "hover" => {
                    if let Some(css) = action.target.as_deref() {
                        out.push_str(&format!("    sb.hover({:?}).await?;\n", css));
                    }
                }
                "select_option_by_text" => {
                    if let (Some(css), Some(value)) =
                        (action.target.as_deref(), action.value.as_deref())
                    {
                        out.push_str(&format!(
                            "    sb.select_option_by_text({:?}, {:?}).await?;\n",
                            css, value
                        ));
                    }
                }
                "select_option_by_value" => {
                    if let (Some(css), Some(value)) =
                        (action.target.as_deref(), action.value.as_deref())
                    {
                        out.push_str(&format!(
                            "    sb.select_option_by_value({:?}, {:?}).await?;\n",
                            css, value
                        ));
                    }
                }
                "switch_to_frame" => {
                    if let Some(css) = action.target.as_deref() {
                        out.push_str(&format!("    sb.switch_to_frame({:?}).await?;\n", css));
                    }
                }
                "switch_to_default_content" => {
                    out.push_str("    sb.switch_to_default_content().await?;\n");
                }
                "drag_and_drop" => {
                    if let (Some(css), Some(value)) =
                        (action.target.as_deref(), action.value.as_deref())
                    {
                        out.push_str(&format!(
                            "    sb.drag_and_drop({:?}, {:?}).await?;\n",
                            css, value
                        ));
                    }
                }
                "wait_for_element_visible" => {
                    if let Some(css) = action.target.as_deref() {
                        out.push_str(&format!(
                            "    sb.wait_for_element_visible({:?}, 10).await?;\n",
                            css
                        ));
                    }
                }
                "wait_for_element_absent" => {
                    if let Some(css) = action.target.as_deref() {
                        out.push_str(&format!(
                            "    sb.wait_for_element_absent({:?}, 10).await?;\n",
                            css
                        ));
                    }
                }
                "clear" => {
                    if let Some(css) = action.target.as_deref() {
                        out.push_str(&format!("    sb.clear({:?}).await?;\n", css));
                    }
                }
                "submit" => {
                    if let Some(css) = action.target.as_deref() {
                        out.push_str(&format!("    sb.submit({:?}).await?;\n", css));
                    }
                }
                "click_link_text" => {
                    if let Some(text) = action.target.as_deref() {
                        out.push_str(&format!("    sb.click_link_text({:?}).await?;\n", text));
                    }
                }

                "accept_alert" => {
                    out.push_str("    sb.accept_alert().await?;
");
                }
                "dismiss_alert" => {
                    out.push_str("    sb.dismiss_alert().await?;
");
                }
                "type_alert_text" => {
                    if let Some(text) = action.target.as_deref() {
                        out.push_str(&format!("    sb.type_alert_text({:?}).await?;
", text));
                    }
                }
                "clear_local_storage" => {
                    out.push_str("    sb.clear_local_storage().await?;
");
                }
                "remove_local_storage_item" => {
                    if let Some(key) = action.target.as_deref() {
                        out.push_str(&format!("    sb.remove_local_storage_item({:?}).await?;
", key));
                    }
                }
                "set_local_storage_item" => {
                    if let (Some(key), Some(value)) = (action.target.as_deref(), action.value.as_deref()) {
                        out.push_str(&format!("    sb.set_local_storage_item({:?}, {:?}).await?;
", key, value));
                    }
                }
                "switch_to_window" => {
                    if let Some(handle) = action.target.as_deref() {
                        out.push_str(&format!("    sb.switch_to_window({:?}).await?;
", handle));
                    }
                }

                "double_click" => {
                    if let Some(css) = action.target.as_deref() {
                        out.push_str(&format!("    sb.double_click({:?}).await?;
", css));
                    }
                }
                "context_click" => {
                    if let Some(css) = action.target.as_deref() {
                        out.push_str(&format!("    sb.context_click({:?}).await?;
", css));
                    }
                }
                "maximize_window" => {
                    out.push_str("    sb.maximize_window().await?;
");
                }
                "scroll_to_bottom" => {
                    out.push_str("    sb.scroll_to_bottom().await?;
");
                }
                "scroll_to_top" => {
                    out.push_str("    sb.scroll_to_top().await?;
");
                }
                "scroll_to" => {
                    if let Some(css) = action.target.as_deref() {
                        out.push_str(&format!("    sb.scroll_to({:?}).await?;
", css));
                    }
                }

                _ => {}
            }
        }
        out.push_str("    sb.quit().await?;\n    Ok(())\n}\n");
        out
    }
}
