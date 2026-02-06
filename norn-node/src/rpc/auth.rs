/// API key configuration for RPC authentication.
#[derive(Debug, Clone)]
pub struct RpcAuthConfig {
    /// If set, mutation methods require this API key via `Authorization: Bearer <key>`.
    pub api_key: Option<String>,
}

impl RpcAuthConfig {
    /// Create a new auth config with no authentication required.
    pub fn open() -> Self {
        Self { api_key: None }
    }

    /// Create a new auth config requiring the given API key.
    pub fn with_key(key: String) -> Self {
        Self { api_key: Some(key) }
    }

    /// Check if a provided bearer token matches the configured API key.
    /// Returns true if no API key is configured (open access) or if the token matches.
    pub fn check(&self, bearer_token: Option<&str>) -> bool {
        match &self.api_key {
            None => true,
            Some(key) => bearer_token == Some(key.as_str()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_allows_all() {
        let auth = RpcAuthConfig::open();
        assert!(auth.check(None));
        assert!(auth.check(Some("anything")));
    }

    #[test]
    fn test_with_key_requires_match() {
        let auth = RpcAuthConfig::with_key("my-secret-key".to_string());
        assert!(auth.check(Some("my-secret-key")));
        assert!(!auth.check(Some("wrong-key")));
        assert!(!auth.check(None));
    }

    #[test]
    fn test_empty_key_still_requires_match() {
        let auth = RpcAuthConfig::with_key(String::new());
        assert!(auth.check(Some("")));
        assert!(!auth.check(Some("notempty")));
        assert!(!auth.check(None));
    }
}
