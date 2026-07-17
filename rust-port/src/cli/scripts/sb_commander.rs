use std::process::{Command, Output};

pub fn run_commander() -> std::io::Result<Output> {
    commander("echo SeleniumBase commander is ready")
}

pub fn commander(command: &str) -> std::io::Result<Output> {
    Command::new("sh").arg("-c").arg(command).output()
}
