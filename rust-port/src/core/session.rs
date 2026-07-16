use std::path::Path;
use std::time::{Duration, Instant};

use serde_json::Value;
use thirtyfour::common::capabilities::chromium::ChromiumLikeCapabilities;
use thirtyfour::extensions::cdp::NetworkConditions;
use thirtyfour::prelude::{By, DesiredCapabilities, WebDriver, WebElement};

use crate::cdp::CdpClient;
use crate::config::{Browser, BrowserConfig};
use crate::error::SeleniumBaseError;
use crate::uc;

pub struct BrowserSession {
    driver: WebDriver,
    cdp: Option<CdpClient>,
}

impl BrowserSession {
    pub async fn connect(config: BrowserConfig) -> Result<Self, SeleniumBaseError> {
        validate_mode_support(&config)?;
        let driver = connect_driver(&config).await?;
        let cdp = if config.is_cdp_enabled() {
            Some(CdpClient::from_handle(driver.handle.clone()))
        } else {
            None
        };

        let session = Self { driver, cdp };
        session.initialize_mode(&config).await?;
        Ok(session)
    }

    pub async fn goto(&mut self, url: &str) -> Result<(), SeleniumBaseError> {
        self.driver.goto(url).await?;
        Ok(())
    }

    pub async fn back(&self) -> Result<(), SeleniumBaseError> {
        self.driver.back().await?;
        Ok(())
    }

    pub async fn forward(&self) -> Result<(), SeleniumBaseError> {
        self.driver.forward().await?;
        Ok(())
    }

    pub async fn refresh(&self) -> Result<(), SeleniumBaseError> {
        self.driver.refresh().await?;
        Ok(())
    }

    pub async fn current_title(&mut self) -> Result<String, SeleniumBaseError> {
        let title = self.driver.title().await?;
        Ok(title)
    }

    pub async fn current_url(&mut self) -> Result<String, SeleniumBaseError> {
        let url = self.driver.current_url().await?;
        Ok(url.as_str().to_owned())
    }

    pub async fn click(&mut self, locator: By) -> Result<(), SeleniumBaseError> {
        let element = self.driver.find(locator).await?;
        element.click().await?;
        Ok(())
    }

    pub async fn type_text(&mut self, locator: By, text: &str) -> Result<(), SeleniumBaseError> {
        let element = self.driver.find(locator).await?;
        element.clear().await?;
        element.send_keys(text).await?;
        Ok(())
    }

    pub async fn clear(&mut self, locator: By) -> Result<(), SeleniumBaseError> {
        let element = self.driver.find(locator).await?;
        element.clear().await?;
        Ok(())
    }

    pub async fn submit(&mut self, locator: By) -> Result<(), SeleniumBaseError> {
        let element = self.driver.find(locator).await?;
        let element_json = element.to_json()?;
        // Form submit via script
        self.driver.execute("arguments[0].closest('form').submit()", vec![element_json]).await?;
        Ok(())
    }

    pub async fn text(&mut self, locator: By) -> Result<String, SeleniumBaseError> {
        let element = self.driver.find(locator).await?;
        Ok(element.text().await?)
    }

    pub async fn hover(&mut self, locator: By) -> Result<(), SeleniumBaseError> {
        let element = self.driver.find(locator).await?;
        self.driver
            .action_chain()
            .move_to_element_center(&element)
            .perform()
            .await?;
        Ok(())
    }

    pub async fn select_option_by_text(
        &mut self,
        locator: By,
        text: &str,
    ) -> Result<(), SeleniumBaseError> {
        let element = self.driver.find(locator).await?;
        let select = thirtyfour::components::SelectElement::new(&element).await?;
        select.select_by_visible_text(text).await?;
        Ok(())
    }

    pub async fn select_option_by_value(
        &mut self,
        locator: By,
        value: &str,
    ) -> Result<(), SeleniumBaseError> {
        let element = self.driver.find(locator).await?;
        let select = thirtyfour::components::SelectElement::new(&element).await?;
        select.select_by_value(value).await?;
        Ok(())
    }

    pub async fn switch_to_frame(&mut self, locator: By) -> Result<(), SeleniumBaseError> {
        let element = self.driver.find(locator).await?;
        element.enter_frame().await?;
        Ok(())
    }

    pub async fn switch_to_default_content(&mut self) -> Result<(), SeleniumBaseError> {
        self.driver.enter_default_frame().await?;
        Ok(())
    }

    pub async fn drag_and_drop(
        &mut self,
        source_locator: By,
        target_locator: By,
    ) -> Result<(), SeleniumBaseError> {
        let source = self.driver.find(source_locator).await?;
        let target = self.driver.find(target_locator).await?;
        self.driver
            .action_chain()
            .drag_and_drop_element(&source, &target)
            .perform()
            .await?;
        Ok(())
    }

    pub async fn page_source(&self) -> Result<String, SeleniumBaseError> {
        Ok(self.driver.source().await?)
    }

    pub async fn find(&mut self, locator: By) -> Result<WebElement, SeleniumBaseError> {
        let element = self.driver.find(locator).await?;
        Ok(element)
    }

    pub async fn find_all(&self, locator: By) -> Result<Vec<WebElement>, SeleniumBaseError> {
        Ok(self.driver.find_all(locator).await?)
    }

    pub async fn element_present(&self, locator: By) -> Result<bool, SeleniumBaseError> {
        let elements = self.find_all(locator).await?;
        Ok(!elements.is_empty())
    }

    pub async fn wait_for_element(
        &self,
        locator: By,
        timeout_secs: u64,
    ) -> Result<WebElement, SeleniumBaseError> {
        let deadline = Instant::now() + Duration::from_secs(timeout_secs);
        loop {
            if let Ok(element) = self.driver.find(locator.clone()).await {
                return Ok(element);
            }
            if Instant::now() >= deadline {
                return Err(SeleniumBaseError::AssertionFailed(
                    "timed out waiting for element".to_owned(),
                ));
            }
            tokio::time::sleep(Duration::from_millis(250)).await;
        }
    }

    pub async fn get_attribute(&mut self, locator: By, attribute_name: &str) -> Result<Option<String>, SeleniumBaseError> {
        let element = self.driver.find(locator).await?;
        Ok(element.attr(attribute_name).await?)
    }

    pub async fn get_property(&mut self, locator: By, property_name: &str) -> Result<Option<String>, SeleniumBaseError> {
        let element = self.driver.find(locator).await?;
        Ok(element.prop(property_name).await?)
    }

    pub async fn wait_for_element_visible(
        &self,
        locator: By,
        timeout_secs: u64,
    ) -> Result<WebElement, SeleniumBaseError> {
        let deadline = Instant::now() + Duration::from_secs(timeout_secs);
        loop {
            if let Ok(element) = self.driver.find(locator.clone()).await {
                if element.is_displayed().await.unwrap_or(false) {
                    return Ok(element);
                }
            }
            if Instant::now() >= deadline {
                return Err(SeleniumBaseError::AssertionFailed(
                    "timed out waiting for element to be visible".to_owned(),
                ));
            }
            tokio::time::sleep(Duration::from_millis(250)).await;
        }
    }

    pub async fn wait_for_element_absent(
        &self,
        locator: By,
        timeout_secs: u64,
    ) -> Result<(), SeleniumBaseError> {
        let deadline = Instant::now() + Duration::from_secs(timeout_secs);
        loop {
            match self.driver.find_all(locator.clone()).await {
                Ok(elements) if elements.is_empty() => return Ok(()),
                Err(_) => return Ok(()),
                _ => {}
            }
            if Instant::now() >= deadline {
                return Err(SeleniumBaseError::AssertionFailed(
                    "timed out waiting for element to be absent".to_owned(),
                ));
            }
            tokio::time::sleep(Duration::from_millis(250)).await;
        }
    }

    pub async fn wait_for_text(
        &self,
        locator: By,
        expected_substring: &str,
        timeout_secs: u64,
    ) -> Result<(), SeleniumBaseError> {
        let deadline = Instant::now() + Duration::from_secs(timeout_secs);
        loop {
            if let Ok(element) = self.driver.find(locator.clone()).await {
                let text = element.text().await?;
                if text.contains(expected_substring) {
                    return Ok(());
                }
            }
            if Instant::now() >= deadline {
                return Err(SeleniumBaseError::AssertionFailed(format!(
                    "timed out waiting for text '{expected_substring}'"
                )));
            }
            tokio::time::sleep(Duration::from_millis(250)).await;
        }
    }

    pub async fn execute_script(&self, script: &str) -> Result<Value, SeleniumBaseError> {
        let ret = self.driver.execute(script, Vec::new()).await?;
        Ok(ret.json().clone())
    }

    pub async fn screenshot(&self, path: &Path) -> Result<(), SeleniumBaseError> {
        self.driver.screenshot(path).await?;
        Ok(())
    }

    pub async fn screenshot_as_png(&self) -> Result<Vec<u8>, SeleniumBaseError> {
        Ok(self.driver.screenshot_as_png().await?)
    }

    pub async fn activate_cdp_mode(&self) -> Result<(), SeleniumBaseError> {
        let cdp = self.cdp_client()?;
        cdp.enable_default_domains().await?;
        Ok(())
    }

    pub async fn execute_cdp(&self, method: &str) -> Result<Value, SeleniumBaseError> {
        let cdp = self.cdp_client()?;
        cdp.execute(method).await
    }

    pub async fn execute_cdp_with_params(
        &self,
        method: &str,
        params: Value,
    ) -> Result<Value, SeleniumBaseError> {
        let cdp = self.cdp_client()?;
        cdp.execute_with_params(method, params).await
    }

    pub async fn clear_browser_cache(&self) -> Result<(), SeleniumBaseError> {
        let cdp = self.cdp_client()?;
        cdp.clear_cache().await
    }

    pub async fn clear_browser_cookies(&self) -> Result<(), SeleniumBaseError> {
        let cdp = self.cdp_client()?;
        cdp.clear_cookies().await
    }

    pub async fn get_cookies(&self) -> Result<Value, SeleniumBaseError> {
        let cdp = self.cdp_client()?;
        cdp.get_cookies().await
    }

    pub async fn cdp_mouse_click(&self, x: f64, y: f64) -> Result<(), SeleniumBaseError> {
        let cdp = self.cdp_client()?;
        cdp.mouse_click(x, y).await
    }

    pub async fn cdp_type_text(&self, text: &str) -> Result<(), SeleniumBaseError> {
        let cdp = self.cdp_client()?;
        cdp.keyboard_insert_text(text).await
    }

    pub async fn set_network_conditions(
        &self,
        conditions: &NetworkConditions,
    ) -> Result<(), SeleniumBaseError> {
        let cdp = self.cdp_client()?;
        cdp.set_network_conditions(conditions).await
    }

    pub async fn enable_uc_mode(&self, config: &BrowserConfig) -> Result<(), SeleniumBaseError> {
        let cdp = self.cdp_client()?;
        cdp.enable_default_domains().await?;
        uc::apply_uc_stealth(cdp).await?;
        if let Some(user_agent) = config.user_agent.as_deref() {
            uc::override_user_agent(cdp, user_agent, config.locale.as_deref()).await?;
        }
        // Also patch the current document for pages already loaded.
        let _ = self
            .driver
            .execute(
                "Object.defineProperty(navigator,'webdriver',{get:()=>undefined});",
                Vec::new(),
            )
            .await?;
        Ok(())
    }

    pub async fn quit(self) -> Result<(), SeleniumBaseError> {
        self.driver.quit().await?;
        Ok(())
    }

    async fn initialize_mode(&self, config: &BrowserConfig) -> Result<(), SeleniumBaseError> {
        if config.is_cdp_enabled() {
            self.activate_cdp_mode().await?;
        }
        if config.is_uc_enabled() {
            self.enable_uc_mode(config).await?;
        }
        Ok(())
    }

    fn cdp_client(&self) -> Result<&CdpClient, SeleniumBaseError> {
        self.cdp.as_ref().ok_or_else(|| {
            SeleniumBaseError::Unsupported(
                "CDP is unavailable for this session. Enable mode cdp or uc.".to_owned(),
            )
        })
    }
}

fn validate_mode_support(config: &BrowserConfig) -> Result<(), SeleniumBaseError> {
    if config.is_cdp_enabled()
        && !matches!(
            config.browser,
            Browser::Chrome | Browser::Chromium | Browser::Edge
        )
    {
        return Err(SeleniumBaseError::InvalidConfig(
            "CDP/UC mode requires a Chromium-based browser (chrome/chromium/edge)".to_owned(),
        ));
    }
    Ok(())
}

async fn connect_driver(config: &BrowserConfig) -> Result<WebDriver, SeleniumBaseError> {
    match config.browser {
        Browser::Chrome | Browser::Chromium => {
            let mut caps = DesiredCapabilities::chrome();
            apply_chromium_capabilities(&mut caps, config)?;
            Ok(WebDriver::new(&config.webdriver_url, caps).await?)
        }
        Browser::Edge => {
            let mut caps = DesiredCapabilities::edge();
            apply_chromium_capabilities(&mut caps, config)?;
            Ok(WebDriver::new(&config.webdriver_url, caps).await?)
        }
        Browser::Firefox => {
            let mut caps = DesiredCapabilities::firefox();
            if config.headless {
                caps.add_arg("-headless")?;
            }
            Ok(WebDriver::new(&config.webdriver_url, caps).await?)
        }
    }
}

fn apply_chromium_capabilities<C: ChromiumLikeCapabilities>(
    caps: &mut C,
    config: &BrowserConfig,
) -> Result<(), SeleniumBaseError> {
    caps.add_arg("--disable-gpu")?;
    caps.add_arg("--window-size=1280,720")?;
    if config.headless {
        caps.add_arg("--headless=new")?;
    }
    if config.ad_block {
        caps.add_arg("--blink-settings=imagesEnabled=false")?;
    }
    if let Some(locale) = config.locale.as_deref() {
        caps.add_arg(&format!("--lang={locale}"))?;
    }
    if let Some(user_agent) = config.user_agent.as_deref() {
        caps.add_arg(&format!("--user-agent={user_agent}"))?;
    }
    if config.is_uc_enabled() {
        caps.add_arg("--disable-blink-features=AutomationControlled")?;
        caps.add_arg("--disable-infobars")?;
        caps.add_arg("--disable-popup-blocking")?;
        caps.add_exclude_switch("enable-automation")?;
        caps.add_experimental_option("useAutomationExtension", false)?;
    }
    Ok(())
}
