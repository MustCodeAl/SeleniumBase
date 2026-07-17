use std::fs::File;
use std::io::Write;

pub fn make_presentation(filename: &str) {
    let mut file = File::create(filename).unwrap();
    let content = r#"<!DOCTYPE html>
<html>
<head>
    <title>SeleniumBase Presentation</title>
    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/reveal.js@4/dist/reveal.css">
</head>
<body>
    <div class="reveal">
        <div class="slides">
            <section><h1>SeleniumBase Rust</h1></section>
            <section><h2>Features</h2><ul><li>Stealth</li><li>CDP</li><li>CLI</li></ul></section>
        </div>
    </div>
    <script src="https://cdn.jsdelivr.net/npm/reveal.js@4/dist/reveal.js"></script>
    <script>Reveal.initialize();</script>
</body>
</html>"#;
    file.write_all(content.as_bytes()).unwrap();
    println!("Created presentation at {}", filename);
}
