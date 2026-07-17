use std::path::PathBuf;

pub fn make_recorder_file(rec_file: &str) -> std::io::Result<PathBuf> {
    let path = PathBuf::from(rec_file);
    let content = r#"// Recorded SeleniumBase Rust test
use seleniumbase_rs::{BaseCase, BrowserConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut sb = BaseCase::new(BrowserConfig::default()).await?;
    sb.open("https://example.com").await?;
    sb.quit().await?;
    Ok(())
}
"#;
    std::fs::write(&path, content)?;
    Ok(path)
}
