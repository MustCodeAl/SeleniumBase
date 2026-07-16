use serde::Deserialize;

use crate::dashboard::RunSummary;
use crate::error::SeleniumBaseError;
use crate::fixtures::base_case::BaseCase;

#[derive(Debug, Deserialize)]
pub struct Scenario {
    pub name: String,
    pub steps: Vec<ScenarioStep>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum ScenarioStep {
    Open {
        url: String,
    },
    Click {
        css: String,
    },
    TypeText {
        css: String,
        text: String,
    },
    AssertText {
        css: String,
        contains: String,
    },
    AssertElement {
        css: String,
    },
    WaitForText {
        css: String,
        text: String,
        timeout: u64,
    },
    Sleep {
        seconds: f64,
    },
    Hover {
        css: String,
    },
    HoverAndClick {
        hover_css: String,
        click_css: String,
    },
    SelectOptionByText {
        css: String,
        text: String,
    },
    SelectOptionByValue {
        css: String,
        value: String,
    },
    DragAndDrop {
        source_css: String,
        target_css: String,
    },
    SwitchToFrame {
        css: String,
    },
    SwitchToDefaultContent,
    Clear {
        css: String,
    },
    ClickLinkText {
        text: String,
    },
    Submit {
        css: String,
    },
}

pub async fn run_scenario(
    sb: &mut BaseCase,
    scenario: &Scenario,
) -> Result<RunSummary, SeleniumBaseError> {
    let mut passed_steps = 0_usize;
    let mut errors = Vec::new();

    for step in &scenario.steps {
        let result = match step {
            ScenarioStep::Open { url } => sb.open(url).await,
            ScenarioStep::Click { css } => sb.click(css).await,
            ScenarioStep::TypeText { css, text } => sb.type_text(css, text).await,
            ScenarioStep::AssertText { css, contains } => sb.assert_text(css, contains).await,
            ScenarioStep::AssertElement { css } => sb.assert_element(css).await,
            ScenarioStep::WaitForText { css, text, timeout } => {
                sb.wait_for_text(css, text, *timeout).await
            }
            ScenarioStep::Sleep { seconds } => {
                sb.sleep(*seconds).await;
                Ok(())
            }
            ScenarioStep::Hover { css } => sb.hover(css).await,
            ScenarioStep::HoverAndClick {
                hover_css,
                click_css,
            } => sb.hover_and_click(hover_css, click_css).await,
            ScenarioStep::SelectOptionByText { css, text } => {
                sb.select_option_by_text(css, text).await
            }
            ScenarioStep::SelectOptionByValue { css, value } => {
                sb.select_option_by_value(css, value).await
            }
            ScenarioStep::DragAndDrop {
                source_css,
                target_css,
            } => sb.drag_and_drop(source_css, target_css).await,
            ScenarioStep::SwitchToFrame { css } => sb.switch_to_frame(css).await,
            ScenarioStep::SwitchToDefaultContent => sb.switch_to_default_content().await,
            ScenarioStep::Clear { css } => sb.clear(css).await,
            ScenarioStep::ClickLinkText { text } => sb.click_link_text(text).await,
            ScenarioStep::Submit { css } => sb.submit(css).await,
        };

        match result {
            Ok(()) => passed_steps += 1,
            Err(e) => errors.push(e.to_string()),
        }
    }

    let failed_steps = scenario.steps.len().saturating_sub(passed_steps);
    Ok(RunSummary {
        scenario_name: scenario.name.clone(),
        total_steps: scenario.steps.len(),
        passed_steps,
        failed_steps,
        errors,
    })
}
