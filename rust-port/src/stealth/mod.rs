//! Stealth and anti-detection support: CDP client wrappers, undetected-chrome
//! evasions, chromedriver patching, fingerprint profiles, and injected
//! JavaScript helpers.

pub mod cdp;
pub mod dprocess;
pub mod evasions;
pub mod fingerprint;
pub mod js;
pub mod options;
pub mod patcher;
pub mod reactor;
pub mod uc;

pub use fingerprint::{Fingerprint, StealthFlags};
