//! Binary-level anti-detection patching for Chromium drivers and engines.
//!
//! Chromedriver leaks several identifying markers into the page (the `cdc_`
//! variables, `__webdriver` globals, etc.). The helpers here patch those
//! markers directly in the driver binary before it is launched, so the browser
//! never injects them in the first place.
//!
//! Typical usage:
//!
//! ```ignore
//! use seleniumbase_rs::stealth::patcher::{ChromedriverPatcher, EnginePatch};
//!
//! ChromedriverPatcher::new("chromedriver")
//!     .patch(EnginePatch::all())?;
//! ```

use crate::error::SeleniumBaseError;
use rand::RngExt;
use regex::bytes::Regex;
use std::fs;
use std::path::{Path, PathBuf};

/// Kinds of binary patches that can be applied to a Chromium driver.
#[derive(Clone, Debug, Default)]
pub struct EnginePatch {
    /// Replace `cdc_` property assignments with spaces.
    pub scrub_cdc_props: bool,
    /// Replace quoted `$cdc_` prefix strings with random ids.
    pub randomize_cdc_prefix: bool,
    /// Replace `__webdriver`, `__selenium`, `__driver` markers.
    pub scrub_webdriver_markers: bool,
    /// Replace `{window.cdc_...;}` blocks.
    pub replace_cdc_blocks: bool,
    /// Create a `.orig` backup before patching.
    pub backup: bool,
}

impl EnginePatch {
    /// A conservative set of patches with backup enabled.
    pub fn balanced() -> Self {
        Self {
            scrub_cdc_props: true,
            randomize_cdc_prefix: true,
            scrub_webdriver_markers: true,
            replace_cdc_blocks: false,
            backup: true,
        }
    }

    /// All supported patches with backup enabled.
    pub fn all() -> Self {
        Self {
            scrub_cdc_props: true,
            randomize_cdc_prefix: true,
            scrub_webdriver_markers: true,
            replace_cdc_blocks: true,
            backup: true,
        }
    }

    /// No backup; useful in CI where the binary is ephemeral.
    pub fn no_backup() -> Self {
        Self {
            backup: false,
            ..Self::all()
        }
    }
}

/// Fluent patcher for a chromedriver-style binary.
#[derive(Clone, Debug)]
pub struct ChromedriverPatcher<P: AsRef<Path>> {
    path: P,
}

impl<P: AsRef<Path>> ChromedriverPatcher<P> {
    /// Wraps the path to a chromedriver binary.
    pub fn new(path: P) -> Self {
        Self { path }
    }

    /// Path to the binary being patched.
    pub fn path(&self) -> &Path {
        self.path.as_ref()
    }

    /// Default backup path (`<binary>.orig`).
    pub fn backup_path(&self) -> PathBuf {
        self.path.as_ref().with_extension("orig")
    }

    /// Applies `spec` to the binary in-place.
    pub fn patch(&self, spec: EnginePatch) -> Result<(), SeleniumBaseError> {
        let path = self.path.as_ref();
        if spec.backup {
            let backup = self.backup_path();
            fs::copy(path, &backup).map_err(|e| {
                SeleniumBaseError::InvalidConfig(format!(
                    "failed to back up chromedriver to {}: {}",
                    backup.display(),
                    e
                ))
            })?;
        }

        let mut content = fs::read(path).map_err(|e| {
            SeleniumBaseError::InvalidConfig(format!(
                "failed to read chromedriver {}: {}",
                path.display(),
                e
            ))
        })?;

        if spec.scrub_cdc_props {
            content = scrub_cdc_property_assignments(content);
        }
        if spec.randomize_cdc_prefix {
            content = randomize_cdc_string_prefix(content);
        }
        if spec.scrub_webdriver_markers {
            content = scrub_automation_markers(content);
        }
        if spec.replace_cdc_blocks {
            content = replace_cdc_blocks(content);
        }

        fs::write(path, content).map_err(|e| {
            SeleniumBaseError::InvalidConfig(format!(
                "failed to write patched chromedriver {}: {}",
                path.display(),
                e
            ))
        })?;
        Ok(())
    }

    /// Restores the binary from the `.orig` backup if it exists.
    pub fn restore(&self) -> Result<(), SeleniumBaseError> {
        let backup = self.backup_path();
        if !backup.exists() {
            return Err(SeleniumBaseError::InvalidConfig(
                "no .orig backup found to restore".to_owned(),
            ));
        }
        fs::copy(&backup, self.path.as_ref()).map_err(|e| {
            SeleniumBaseError::InvalidConfig(format!(
                "failed to restore chromedriver from {}: {}",
                backup.display(),
                e
            ))
        })?;
        Ok(())
    }

    /// Returns true if known automation markers are still present.
    pub fn needs_patch(&self) -> Result<bool, SeleniumBaseError> {
        let content = fs::read(self.path.as_ref()).map_err(|e| {
            SeleniumBaseError::InvalidConfig(format!(
                "failed to read chromedriver {}: {}",
                self.path.as_ref().display(),
                e
            ))
        })?;
        Ok(has_automation_markers(&content))
    }
}

fn has_automation_markers(content: &[u8]) -> bool {
    static PATTERNS: std::sync::OnceLock<Vec<Regex>> = std::sync::OnceLock::new();
    let patterns = PATTERNS.get_or_init(|| {
        vec![
            Regex::new(r"window\.cdc_[a-zA-Z0-9]{22}_").unwrap(),
            Regex::new(r#"['\"]?\$cdc_[a-zA-Z0-9]{22}_['\"]?"#).unwrap(),
            Regex::new(r"__webdriver|__selenium|__driver").unwrap(),
            Regex::new(r"\{window\.cdc.*?;\}").unwrap(),
        ]
    });
    patterns.iter().any(|re| re.is_match(content))
}

fn scrub_cdc_property_assignments(mut content: Vec<u8>) -> Vec<u8> {
    let re = Regex::new(
        r"window\.cdc_[a-zA-Z0-9]{22}_(Array|Promise|Symbol|Object|Proxy|JSON|Window)\s*=\s*window\.(Array|Promise|Symbol|Object|Proxy|JSON|Window);",
    )
    .unwrap();
    content = re
        .replace_all(&content, |caps: &regex::bytes::Captures| {
            vec![b' '; caps[0].len()]
        })
        .into_owned();

    let re = Regex::new(
        r"window\.cdc_[a-zA-Z0-9]{22}_(Array|Promise|Symbol|Object|Proxy|JSON|Window)\s*\|\|",
    )
    .unwrap();
    content = re
        .replace_all(&content, |caps: &regex::bytes::Captures| {
            vec![b' '; caps[0].len()]
        })
        .into_owned();
    content
}

fn randomize_cdc_string_prefix(mut content: Vec<u8>) -> Vec<u8> {
    let re = Regex::new(r#"['\"]\$cdc_[a-zA-Z0-9]{22}_['\"];"#).unwrap();
    content = re
        .replace_all(&content, |caps: &regex::bytes::Captures| {
            let full = &caps[0];
            let quote = full[0];
            let inner_len = full.len().saturating_sub(3); // two quotes + semicolon
            let mut rng = rand::rng();
            let ran_len = rng.random_range(6..=inner_len.max(6));
            let chars: Vec<u8> = (0..ran_len)
                .map(|_| rng.random_range(b'a'..=b'z'))
                .collect();

            let mut out = Vec::with_capacity(full.len());
            out.push(quote);
            out.extend(chars);
            out.push(quote);
            out.push(b';');
            out.extend(std::iter::repeat_n(
                b'\n',
                inner_len.saturating_sub(ran_len),
            ));
            out
        })
        .into_owned();
    content
}

fn scrub_automation_markers(mut content: Vec<u8>) -> Vec<u8> {
    let re = Regex::new(r"__webdriver|__selenium|__driver").unwrap();
    content = re
        .replace_all(&content, |caps: &regex::bytes::Captures| {
            vec![b' '; caps[0].len()]
        })
        .into_owned();
    content
}

fn replace_cdc_blocks(mut content: Vec<u8>) -> Vec<u8> {
    let re = Regex::new(r"\{window\.cdc.*?;\}").unwrap();
    content = re
        .replace_all(&content, |caps: &regex::bytes::Captures| {
            let mut out = b"{console.log(\"chromedriver is undetectable!\")}".to_vec();
            if out.len() < caps[0].len() {
                out.extend(vec![b' '; caps[0].len() - out.len()]);
            } else {
                out.truncate(caps[0].len());
            }
            out
        })
        .into_owned();
    content
}

/// Convenience one-shot patcher preserving the old API.
pub fn patch_chromedriver<P: AsRef<Path>>(path: P) -> Result<(), SeleniumBaseError> {
    ChromedriverPatcher::new(path).patch(EnginePatch::all())
}

/// Returns additional Chromium command-line flags that reduce engine-level
/// automation fingerprints. These complement the JS evasions applied at
/// runtime.
pub fn engine_spoofing_args() -> Vec<String> {
    vec![
        "--disable-blink-features=AutomationControlled".to_owned(),
        "--disable-features=IsolateOrigins,site-per-process,PrivacySandboxSettings4,InterestFeedContentSuggestions,FedCm,WebRtcHideLocalIpsWithMdns".to_owned(),
        "--disable-component-extensions-with-background-pages".to_owned(),
        "--disable-background-networking".to_owned(),
        "--disable-background-timer-throttling".to_owned(),
        "--disable-backgrounding-occluded-windows".to_owned(),
        "--disable-renderer-backgrounding".to_owned(),
        "--disable-client-side-phishing-detection".to_owned(),
        "--disable-default-apps".to_owned(),
        "--disable-hang-monitor".to_owned(),
        "--disable-popup-blocking".to_owned(),
        "--disable-prompt-on-repost".to_owned(),
        "--disable-sync".to_owned(),
        "--disable-translate".to_owned(),
        "--metrics-recording-only".to_owned(),
        "--no-first-run".to_owned(),
        "--no-default-browser-check".to_owned(),
        "--no-pings".to_owned(),
        "--password-store=basic".to_owned(),
        "--use-mock-keychain".to_owned(),
        "--force-webrtc-ip-handling-policy=disable_non_proxied_udp".to_owned(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn patches_cdc_marker() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("chromedriver");
        let marker = b"window.cdc_abcdef1234567890abcdef_Array = window.Array;";
        fs::write(&path, marker).unwrap();

        patch_chromedriver(&path).unwrap();

        let patched = fs::read(&path).unwrap();
        let patched_str = String::from_utf8_lossy(&patched);
        assert!(!patched_str.contains("cdc_abcdef1234567890abcdef_Array"));
    }

    #[test]
    fn patcher_detects_markers() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("chromedriver");
        fs::write(
            &path,
            b"window.cdc_abcdef1234567890abcdef_Array = window.Array;",
        )
        .unwrap();

        let patcher = ChromedriverPatcher::new(&path);
        assert!(patcher.needs_patch().unwrap());

        patcher.patch(EnginePatch::balanced()).unwrap();
        assert!(!patcher.needs_patch().unwrap());
    }

    #[test]
    fn patcher_creates_backup_and_restores() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("chromedriver");
        let original = b"window.cdc_abcdef1234567890abcdef_Array = window.Array;";
        fs::write(&path, original).unwrap();

        let patcher = ChromedriverPatcher::new(&path);
        patcher.patch(EnginePatch::all()).unwrap();
        assert!(patcher.backup_path().exists());

        patcher.restore().unwrap();
        let restored = fs::read(&path).unwrap();
        assert_eq!(&restored[..], original);
    }

    #[test]
    fn engine_args_include_automation_switch() {
        let args = engine_spoofing_args();
        assert!(args.contains(&"--disable-blink-features=AutomationControlled".to_owned()));
        assert!(args.iter().any(|a| a.contains("PrivacySandboxSettings4")));
    }
}
