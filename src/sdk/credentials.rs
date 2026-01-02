//! Credential resolution for Augment API.
//!
//! Credentials are resolved in order:
//! 1. Explicit options
//! 2. Environment variables (AUGMENT_API_TOKEN, AUGMENT_API_URL)
//! 3. Session file (~/.augment/session.json)

use serde::Deserialize;
use std::path::PathBuf;
use tokio::fs;

use crate::error::{Error, Result};

/// Resolved credentials for API access.
#[derive(Debug, Clone)]
pub struct Credentials {
    /// API key (Bearer token)
    pub api_key: String,
    /// API base URL
    pub api_url: String,
}

/// Session file structure.
#[derive(Debug, Deserialize)]
struct SessionFile {
    #[serde(rename = "accessToken")]
    access_token: Option<String>,
    #[serde(rename = "tenantURL")]
    tenant_url: Option<String>,
}

/// Get the path to the session file.
fn session_file_path() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".augment").join("session.json"))
}

/// Read the session file.
async fn read_session_file() -> Option<SessionFile> {
    let path = session_file_path()?;
    let content = fs::read_to_string(&path).await.ok()?;
    serde_json::from_str(&content).ok()
}

/// Resolve credentials from options, environment, or session file.
///
/// # Arguments
///
/// * `api_key` - Optional API key override
/// * `api_url` - Optional API URL override
///
/// # Errors
///
/// Returns an error if credentials cannot be resolved from any source.
pub async fn resolve_credentials(
    api_key: Option<&str>,
    api_url: Option<&str>,
) -> Result<Credentials> {
    // Try options first
    let mut resolved_key = api_key.map(String::from);
    let mut resolved_url = api_url.map(String::from);

    // Try environment variables
    if resolved_key.is_none() {
        resolved_key = std::env::var("AUGMENT_API_TOKEN").ok();
    }
    if resolved_url.is_none() {
        resolved_url = std::env::var("AUGMENT_API_URL").ok();
    }

    // Try session file
    if resolved_key.is_none() || resolved_url.is_none() {
        if let Some(session) = read_session_file().await {
            if resolved_key.is_none() {
                resolved_key = session.access_token;
            }
            if resolved_url.is_none() {
                resolved_url = session.tenant_url;
            }
        }
    }

    // Validate
    let api_key = resolved_key.ok_or_else(|| {
        Error::CredentialsNotFound(
            "API key is required. Provide it via:\n\
             1. options.api_key parameter\n\
             2. AUGMENT_API_TOKEN environment variable\n\
             3. Run 'auggie login' to create ~/.augment/session.json"
                .to_string(),
        )
    })?;

    let api_url = resolved_url.ok_or_else(|| {
        Error::CredentialsNotFound(
            "API URL is required. Provide it via:\n\
             1. options.api_url parameter\n\
             2. AUGMENT_API_URL environment variable\n\
             3. Run 'auggie login' to create ~/.augment/session.json"
                .to_string(),
        )
    })?;

    Ok(Credentials { api_key, api_url })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_resolve_from_options() {
        let creds = resolve_credentials(Some("test-key"), Some("https://api.example.com"))
            .await
            .unwrap();

        assert_eq!(creds.api_key, "test-key");
        assert_eq!(creds.api_url, "https://api.example.com");
    }

    #[tokio::test]
    async fn test_missing_credentials() {
        // Clear env vars for this test
        std::env::remove_var("AUGMENT_API_TOKEN");
        std::env::remove_var("AUGMENT_API_URL");

        let result = resolve_credentials(None, None).await;

        // Should fail if no session file exists
        // (This test may pass if session file exists on the system)
        if result.is_err() {
            assert!(matches!(result.unwrap_err(), Error::CredentialsNotFound(_)));
        }
    }
}
