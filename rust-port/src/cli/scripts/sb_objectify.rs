use std::fs::File;
use std::io::Write;

pub fn objectify_page() {
    let mut file = File::create("page_object.rs").unwrap();
    let content = r#"use seleniumbase_rs::{BaseCase, SeleniumBaseError};

pub struct LoginPage<'a> {
    sb: &'a mut BaseCase,
}

impl<'a> LoginPage<'a> {
    pub fn new(sb: &'a mut BaseCase) -> Self {
        Self { sb }
    }

    pub async fn open(&mut self) -> Result<(), SeleniumBaseError> {
        self.sb.open("https://example.com/login").await
    }
}
"#;
    file.write_all(content.as_bytes()).unwrap();
    println!("Generated page_object.rs");
}
