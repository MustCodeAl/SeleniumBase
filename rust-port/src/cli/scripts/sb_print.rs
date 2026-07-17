use std::fs;

pub fn print_file(filename: &str) {
    match fs::read_to_string(filename) {
        Ok(content) => println!("{}", content),
        Err(e) => eprintln!("Failed to read file {}: {}", filename, e),
    }
}
