use crate::browser::downloader::download_chrome_driver;

pub async fn install_drivers() {
    if let Err(e) = download_chrome_driver().await {
        eprintln!("Failed to install driver: {}", e);
    } else {
        println!("Drivers installed successfully!");
    }
}
