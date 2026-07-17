use std::fs::File;
use std::io::Write;

pub fn make_recorder_file(rec_file: &str) {
    let mut file = File::create(rec_file).unwrap();
    let content = r#"// Recorded SeleniumBase Rust test
use seleniumbase_rs::{BaseCase, BrowserConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut sb = BaseCase::new(BrowserConfig::default()).await?;
    sb.open("https://example.com").await?;
    // TODO: add recorded actions
    sb.quit().await?;
    Ok(())
}
"#;
    file.write_all(content.as_bytes()).unwrap();
    println!("Created recorder file at {}", rec_file);
}
