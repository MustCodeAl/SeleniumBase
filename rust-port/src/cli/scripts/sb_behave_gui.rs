use std::io::{self, Write};
use std::path::Path;

pub fn run_gui() {
    println!("SeleniumBase BDD Feature Generator");
    println!("==================================");

    if !Path::new("features").exists() {
        println!("Creating features/ directory...");
        let _ = std::fs::create_dir_all("features");
    }

    let feature_name = prompt("Feature file name (without .feature): ");
    let feature_title = prompt("Feature title: ");
    let as_a = prompt("As a: ");
    let i_want = prompt("I want: ");
    let so_that = prompt("So that: ");

    let mut feature_content = format!(
        "Feature: {title}\n  As a {as_a}\n  I want {i_want}\n  So that {so_that}\n\n",
        title = feature_title,
        as_a = as_a,
        i_want = i_want,
        so_that = so_that
    );

    let scenario_count = prompt("How many scenarios? ");
    let scenario_count: usize = scenario_count.trim().parse().unwrap_or(1);

    for i in 1..=scenario_count {
        let scenario_name = prompt(&format!("Scenario {} name: ", i));
        feature_content.push_str(&format!("  Scenario: {}\n", scenario_name));

        loop {
            let step = prompt("  Add a step (Given/When/Then/And/But) or 'done': ");
            let step = step.trim();
            if step.eq_ignore_ascii_case("done") {
                break;
            }
            feature_content.push_str(&format!("    {}\n", step));
        }
        feature_content.push('\n');
    }

    let file_path = format!("features/{}.feature", feature_name.trim());
    match std::fs::write(&file_path, feature_content) {
        Ok(_) => println!("Created BDD feature file: {}", file_path),
        Err(e) => eprintln!("Failed to write feature file {}: {}", file_path, e),
    }

    let readme_path = "features/README.md";
    let readme = r#"# BDD Features

This directory contains Gherkin feature files for behavior-driven development tests.

You can run these features with a Rust BDD framework such as `cucumber`:

```toml
[dev-dependencies]
cucumber = "0.21"
```

See the `cucumber` crate documentation for wiring step definitions to these features.
"#;
    let _ = std::fs::write(readme_path, readme);
}

fn prompt(message: &str) -> String {
    print!("{}", message);
    let _ = io::stdout().flush();
    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input);
    input.trim().to_owned()
}
