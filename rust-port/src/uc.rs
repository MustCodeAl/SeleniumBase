use serde_json::json;

use crate::cdp::CdpClient;
use crate::error::SeleniumBaseError;

// A conservative stealth patch set that avoids broad prototype rewrites.
const UC_STEALTH_SCRIPT: &str = r#"
Object.defineProperty(navigator, 'webdriver', { get: () => undefined });
Object.defineProperty(navigator, 'languages', { get: () => ['en-US', 'en'] });
Object.defineProperty(navigator, 'plugins', { get: () => [1, 2, 3, 4, 5] });
if (!window.chrome) {
  window.chrome = { runtime: {} };
}
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
