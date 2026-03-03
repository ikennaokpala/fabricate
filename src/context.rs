use crate::sequence::Sequence;

/// Shared context passed to all factory operations.
///
/// Holds the database pool (when available), sequence generators,
/// and configuration for the factory session.
pub struct FactoryContext {
    /// Auto-incrementing sequences for generating unique values.
    pub sequences: Sequence,

    /// Database connection pool (when using direct DB mode).
    #[cfg(feature = "postgres")]
    pub pool: Option<sqlx::PgPool>,

    /// HTTP client for API-based seeding.
    pub http_client: Option<reqwest::Client>,

    /// Base URL for the backend API (e.g., "http://localhost:8080").
    pub base_url: Option<String>,

    /// Test API key for authenticating with `/__test__/` endpoints.
    pub test_key: String,

    /// Field overrides applied via `.set("field", value)`.
    pub overrides: std::collections::HashMap<String, serde_json::Value>,
}

impl FactoryContext {
    /// Create a new context for HTTP API-based seeding.
    pub fn http(base_url: &str) -> Self {
        Self {
            sequences: Sequence::new(),
            #[cfg(feature = "postgres")]
            pool: None,
            http_client: Some(reqwest::Client::new()),
            base_url: Some(base_url.to_string()),
            test_key: "test-key".to_string(),
            overrides: std::collections::HashMap::new(),
        }
    }

    /// Create a new context for direct database seeding.
    #[cfg(feature = "postgres")]
    pub fn database(pool: sqlx::PgPool) -> Self {
        Self {
            sequences: Sequence::new(),
            pool: Some(pool),
            http_client: None,
            base_url: None,
            test_key: "test-key".to_string(),
            overrides: std::collections::HashMap::new(),
        }
    }

    /// Set the test API key.
    pub fn with_test_key(mut self, key: &str) -> Self {
        self.test_key = key.to_string();
        self
    }

    /// Get the next value for a named sequence.
    pub fn sequence(&mut self, name: &str) -> u64 {
        self.sequences.next(name)
    }

    /// Generate a unique test email.
    pub fn email(&mut self, prefix: &str) -> String {
        self.sequences.email(prefix)
    }

    /// Generate a unique test phone.
    pub fn phone(&mut self) -> String {
        self.sequences.phone()
    }

    /// Generate a unique full name.
    pub fn full_name(&mut self) -> String {
        self.sequences.full_name()
    }

    /// Set a field override.
    pub fn set_override(&mut self, field: &str, value: serde_json::Value) {
        self.overrides.insert(field.to_string(), value);
    }

    /// Get a field override, if set.
    pub fn get_override(&self, field: &str) -> Option<&serde_json::Value> {
        self.overrides.get(field)
    }

    /// Clear all overrides.
    pub fn clear_overrides(&mut self) {
        self.overrides.clear();
    }

    /// Reset sequences and overrides for a fresh session.
    pub fn reset(&mut self) {
        self.sequences.reset();
        self.overrides.clear();
    }

    /// Check backend health by hitting the health endpoint.
    pub async fn health_check(&self) -> crate::Result<bool> {
        let client = self
            .http_client
            .as_ref()
            .ok_or_else(|| crate::Error::Build("No HTTP client configured".into()))?;
        let base = self
            .base_url
            .as_ref()
            .ok_or_else(|| crate::Error::Build("No base URL configured".into()))?;

        let url = format!("{base}/api/v1/health");
        let resp = client
            .get(&url)
            .header("X-Test-Key", &self.test_key)
            .send()
            .await?;

        Ok(resp.status().is_success())
    }

    /// Make an authenticated POST to a `/__test__/` endpoint.
    pub async fn test_post<T: serde::Serialize>(
        &self,
        path: &str,
        body: &T,
    ) -> crate::Result<serde_json::Value> {
        let client = self
            .http_client
            .as_ref()
            .ok_or_else(|| crate::Error::Build("No HTTP client configured".into()))?;
        let base = self
            .base_url
            .as_ref()
            .ok_or_else(|| crate::Error::Build("No base URL configured".into()))?;

        let url = format!("{base}{path}");
        let resp = client
            .post(&url)
            .header("X-Test-Key", &self.test_key)
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(crate::Error::Build(format!(
                "API error {status}: {text}"
            )));
        }

        let json: serde_json::Value = resp.json().await?;
        Ok(json)
    }

    /// Make an authenticated DELETE to a `/__test__/` endpoint.
    pub async fn test_delete(&self, path: &str) -> crate::Result<()> {
        let client = self
            .http_client
            .as_ref()
            .ok_or_else(|| crate::Error::Build("No HTTP client configured".into()))?;
        let base = self
            .base_url
            .as_ref()
            .ok_or_else(|| crate::Error::Build("No base URL configured".into()))?;

        let url = format!("{base}{path}");
        let resp = client
            .post(&url)
            .header("X-Test-Key", &self.test_key)
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({}))
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(crate::Error::Build(format!(
                "API error {status}: {text}"
            )));
        }

        Ok(())
    }
}
