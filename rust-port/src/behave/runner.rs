use crate::behave::helper::parse_scenarios;
use crate::behave::steps::StepRegistry;
use std::fs;
use std::path::Path;

/// Result of running one scenario.
#[derive(Debug, Clone, PartialEq)]
pub struct ScenarioResult {
    pub name: String,
    pub passed: bool,
    pub messages: Vec<String>,
}

/// Run all scenarios in a feature file using the provided step registry.
pub fn run_feature_file<P: AsRef<Path>>(
    path: P,
    registry: &StepRegistry,
) -> Result<Vec<ScenarioResult>, std::io::Error> {
    let content = fs::read_to_string(path)?;
    let lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
    let scenarios = parse_scenarios(&lines);
    let mut results = Vec::new();
    for (name, steps) in scenarios {
        let mut messages = Vec::new();
        let mut passed = true;
        for step in steps {
            match registry.run(&step) {
                Ok(()) => {}
                Err(e) => {
                    passed = false;
                    messages.push(format!("{} -> {}", step, e));
                }
            }
        }
        results.push(ScenarioResult {
            name,
            passed,
            messages,
        });
    }
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_feature_file() {
        let tmp = std::env::temp_dir().join("sb_feature_test.feature");
        fs::write(
            &tmp,
            "Feature: test\nScenario: pass\nGiven one\nThen two\n",
        )
        .unwrap();
        let mut registry = StepRegistry::new();
        registry.register("Given one", Box::new(|_| Ok(())));
        registry.register("Then two", Box::new(|_| Ok(())));
        let results = run_feature_file(&tmp, &registry).unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].passed);
        let _ = fs::remove_file(&tmp);
    }
}
