use std::collections::HashMap;

/// A step handler that returns Ok(()) on success.
pub type StepHandler = Box<dyn Fn(&str) -> Result<(), String>>;

/// Registry mapping step patterns to handlers.
pub struct StepRegistry {
    handlers: HashMap<String, StepHandler>,
}

impl StepRegistry {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    /// Register an exact step string with a handler.
    pub fn register(&mut self, pattern: impl Into<String>, handler: StepHandler) {
        self.handlers.insert(pattern.into(), handler);
    }

    /// Find and run the handler for a step.
    pub fn run(&self, step: &str) -> Result<(), String> {
        let trimmed = step.trim();
        for (pattern, handler) in &self.handlers {
            if pattern == trimmed {
                return handler(trimmed);
            }
        }
        Err(format!("no handler registered for step: {}", step))
    }

    pub fn has_handler(&self, step: &str) -> bool {
        self.handlers.contains_key(step.trim())
    }
}

impl Default for StepRegistry {
    fn default() -> Self {
        Self::new()
    }
}
