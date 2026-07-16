use serde_json::json;

use crate::cdp::CdpClient;
use crate::error::SeleniumBaseError;

// A robust stealth patch set for UC mode.
const UC_STEALTH_SCRIPT: &str = r#"
Object.defineProperty(navigator, 'webdriver', { get: () => undefined });
Object.defineProperty(navigator, 'languages', { get: () => ['en-US', 'en'] });
Object.defineProperty(navigator, 'plugins', { get: () => [1, 2, 3, 4, 5] });
if (!window.chrome) {
  window.chrome = { runtime: {} };
}
// Remove cdc_ properties if they exist
let objectToInspect = window;
let cdc_props = [];
while (objectToInspect !== null) {
  cdc_props = cdc_props.concat(Object.getOwnPropertyNames(objectToInspect));
  objectToInspect = Object.getPrototypeOf(objectToInspect);
}
cdc_props.filter(i => i.match(/^[a-z]{3}_[a-z]{22}_.*/i)).forEach(p => delete window[p]);
"#;

pub async fn apply_uc_stealth(cdp: &CdpClient) -> Result<(), SeleniumBaseError> {
    cdp.add_init_script(UC_STEALTH_SCRIPT).await?;
    Ok(())
}

pub async fn override_user_agent(
    cdp: &CdpClient,
    user_agent: &str,
    locale: Option<&str>,
) -> Result<(), SeleniumBaseError> {
    let mut params = json!({ "userAgent": user_agent });
    if let Some(locale_value) = locale {
        params["acceptLanguage"] = json!(locale_value);
    }
    cdp.execute_with_params("Network.setUserAgentOverride", params)
        .await?;
    Ok(())
}
