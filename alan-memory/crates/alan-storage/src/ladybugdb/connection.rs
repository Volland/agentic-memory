/// Placeholder for a LadybugDB connection.
/// Actual LadybugDB bindings will be added later.
#[derive(Debug, Clone)]
pub struct LadybugDbConnection {
    path: String,
}

impl LadybugDbConnection {
    /// Create a new connection handle (no actual connection is opened yet).
    pub fn new(path: impl Into<String>) -> Self {
        Self { path: path.into() }
    }

    /// Returns the configured database path.
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Placeholder: always returns false until real bindings are added.
    pub fn is_connected(&self) -> bool {
        false
    }
}
