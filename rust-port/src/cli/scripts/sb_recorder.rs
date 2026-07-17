use crate::api::recorder::ActionRecorder;
use std::io::{self, Write};

pub fn start_recorder() {
    println!("Starting SeleniumBase action recorder...");
    println!("Type actions as: click <selector>");
    println!("Type 'save <filename>' to export, or 'quit' to exit.");

    let mut recorder = ActionRecorder::default();
    loop {
        print!("> ");
        let _ = io::stdout().flush();
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            break;
        }
        let input = input.trim();
        if input.eq_ignore_ascii_case("quit") {
            break;
        }
        if let Some(path) = input.strip_prefix("save ") {
            let script = recorder.to_rust_script();
            match std::fs::write(path, script) {
                Ok(_) => println!("Saved recorded test to {}", path),
                Err(e) => eprintln!("Failed to save: {}", e),
            }
            break;
        }
        let parts: Vec<&str> = input.splitn(3, ' ').collect();
        if parts.len() >= 2 {
            let action = parts[0];
            let target = parts[1];
            let value = parts.get(2).copied();
            recorder.record(action, Some(target), value);
            println!("Recorded: {} {}", action, target);
        }
    }
}
