use std::process::Command;

pub fn run_tests() {
    println!("Running SeleniumBase Rust tests...");
    let output = Command::new("cargo").arg("test").output();
    match output {
        Ok(out) => {
            println!("{}", String::from_utf8_lossy(&out.stdout));
            eprintln!("{}", String::from_utf8_lossy(&out.stderr));
        }
        Err(e) => eprintln!("Failed to run tests: {}", e),
    }
}
