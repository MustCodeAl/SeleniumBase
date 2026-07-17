use crate::plugins::base_plugin::SeleniumBasePlugin;

/// Plugin that logs test activity to stdout (S3 upload can be wired in later).
pub struct S3LoggingPlugin;

impl S3LoggingPlugin {
    pub fn new() -> Self {
        Self
    }
}

impl SeleniumBasePlugin for S3LoggingPlugin {
    fn on_start(&mut self) {
        println!("[S3LoggingPlugin] test session started");
    }

    fn before_command(&mut self, name: &str, target: &str, value: &str) {
        println!("[S3LoggingPlugin] before {} target={} value={}", name, target, value);
    }

    fn after_command(&mut self, name: &str, target: &str, value: &str, passed: bool) {
        println!(
            "[S3LoggingPlugin] after {} target={} value={} passed={}",
            name, target, value, passed
        );
    }

    fn on_stop(&mut self) {
        println!("[S3LoggingPlugin] test session stopped");
    }
}

impl Default for S3LoggingPlugin {
    fn default() -> Self {
        Self::new()
    }
}
