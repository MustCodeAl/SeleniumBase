use std::fs::File;
use std::io::{self, Cursor};
use std::path::PathBuf;

use reqwest::Client;
use zip::ZipArchive;

use crate::error::SeleniumBaseError;

/// Downloads and extracts the latest chromedriver for the current platform
pub async fn download_chrome_driver() -> Result<(), SeleniumBaseError> {
    let client = Client::new();
    
    // Determine platform
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    
    let platform = match (os, arch) {
        ("macos", "aarch64") => "mac-arm64",
        ("macos", "x86_64") => "mac-x64",
        ("linux", _) => "linux64",
        ("windows", "x86_64") => "win64",
        ("windows", "x86") => "win32",
        _ => return Err(SeleniumBaseError::Unsupported(format!("Unsupported platform: {}-{}", os, arch))),
    };

    println!("Fetching latest Chrome for Testing (CfT) version info...");
    
    // Fetch latest version info
    let version_url = "https://googlechromelabs.github.io/chrome-for-testing/last-known-good-versions-with-downloads.json";
    let resp = client.get(version_url).send().await
        .map_err(|e| SeleniumBaseError::Unsupported(format!("Failed to fetch version info: {}", e)))?;
        
    let json: serde_json::Value = resp.json().await
        .map_err(|e| SeleniumBaseError::Unsupported(format!("Failed to parse version JSON: {}", e)))?;
        
    let stable_info = &json["channels"]["Stable"];
    let version = stable_info["version"].as_str().unwrap_or("unknown");
    
    println!("Latest stable Chrome version: {}", version);
    
    // Find the correct download URL for this platform
    let downloads = &stable_info["downloads"]["chromedriver"];
    let mut download_url = None;
    
    if let Some(arr) = downloads.as_array() {
        for dl in arr {
            if dl["platform"].as_str() == Some(platform) {
                download_url = dl["url"].as_str();
                break;
            }
        }
    }
    
    let url = download_url.ok_or_else(|| {
        SeleniumBaseError::Unsupported(format!("No chromedriver download found for platform: {}", platform))
    })?;

    println!("Downloading chromedriver from {}...", url);
    
    // Download the ZIP file
    let resp = client.get(url).send().await
        .map_err(|e| SeleniumBaseError::Unsupported(format!("Failed to download chromedriver: {}", e)))?;
        
    let bytes = resp.bytes().await
        .map_err(|e| SeleniumBaseError::Unsupported(format!("Failed to read downloaded bytes: {}", e)))?;
        
    println!("Extracting chromedriver...");
    
    // Create downloaded_drivers directory if it doesn't exist
    let dest_dir = PathBuf::from("downloaded_drivers");
    if !dest_dir.exists() {
        std::fs::create_dir_all(&dest_dir)
            .map_err(|e| SeleniumBaseError::Unsupported(format!("Failed to create drivers directory: {}", e)))?;
    }
    
    // Extract the ZIP archive
    let reader = Cursor::new(bytes);
    let mut archive = ZipArchive::new(reader)
        .map_err(|e| SeleniumBaseError::Unsupported(format!("Failed to read ZIP archive: {}", e)))?;
        
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)
            .map_err(|e| SeleniumBaseError::Unsupported(format!("Failed to read file in ZIP: {}", e)))?;
            
        let outpath = match file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue,
        };
        
        // We only care about the actual chromedriver executable, not the folder structure
        let file_name = outpath.file_name().unwrap_or_default().to_string_lossy();
        if file_name == "chromedriver" || file_name == "chromedriver.exe" {
            let mut dest_path = dest_dir.clone();
            dest_path.push(file_name.as_ref());
            
            let mut outfile = File::create(&dest_path)
                .map_err(|e| SeleniumBaseError::Unsupported(format!("Failed to create output file {:?}: {}", dest_path, e)))?;
                
            io::copy(&mut file, &mut outfile)
                .map_err(|e| SeleniumBaseError::Unsupported(format!("Failed to extract file: {}", e)))?;
                
            // Set executable permissions on Unix
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = std::fs::metadata(&dest_path)
                    .map_err(|e| SeleniumBaseError::Unsupported(format!("Failed to read metadata: {}", e)))?
                    .permissions();
                perms.set_mode(0o755);
                std::fs::set_permissions(&dest_path, perms)
                    .map_err(|e| SeleniumBaseError::Unsupported(format!("Failed to set permissions: {}", e)))?;
            }
            
            println!("Successfully installed to {:?}", dest_path);
            return Ok(());
        }
    }
    
    Err(SeleniumBaseError::Unsupported("chromedriver executable not found in ZIP archive".to_string()))
}
