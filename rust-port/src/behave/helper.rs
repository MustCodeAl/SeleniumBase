use std::fs;
use std::path::Path;

/// Read a Gherkin feature file and return its lines.
pub fn read_feature_file<P: AsRef<Path>>(path: P) -> Result<Vec<String>, std::io::Error> {
    let content = fs::read_to_string(path)?;
    Ok(content.lines().map(|l| l.to_string()).collect())
}

/// Extract scenarios from a feature file as (name, step_lines).
pub fn parse_scenarios(lines: &[String]) -> Vec<(String, Vec<String>)> {
    let mut scenarios = Vec::new();
    let mut current_name = String::new();
    let mut current_steps = Vec::new();
    for line in lines {
        let trimmed = line.trim();
        if trimmed.starts_with("Scenario:") || trimmed.starts_with("Scenario Outline:") {
            if !current_name.is_empty() {
                scenarios.push((current_name.clone(), current_steps.clone()));
            }
            current_name = trimmed
                .trim_start_matches("Scenario Outline:")
                .trim_start_matches("Scenario:")
                .trim()
                .to_string();
            current_steps.clear();
        } else if trimmed.starts_with("Given ")
            || trimmed.starts_with("When ")
            || trimmed.starts_with("Then ")
            || trimmed.starts_with("And ")
            || trimmed.starts_with("But ")
        {
            current_steps.push(trimmed.to_string());
        }
    }
    if !current_name.is_empty() {
        scenarios.push((current_name, current_steps));
    }
    scenarios
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_scenarios() {
        let lines = vec![
            "Feature: login".into(),
            "Scenario: valid login".into(),
            "Given user is on login page".into(),
            "When user enters credentials".into(),
            "Then user is logged in".into(),
        ];
        let scenarios = parse_scenarios(&lines);
        assert_eq!(scenarios.len(), 1);
        assert_eq!(scenarios[0].0, "valid login");
        assert_eq!(scenarios[0].1.len(), 3);
    }
}
