//! HTTP client for Augment backend API.
//!
//! This module provides the low-level HTTP client for communicating
//! with the Augment backend API, including SSE streaming support.

use reqwest::{Client, Response};
use serde::de::DeserializeOwned;
use serde::Serialize;
use uuid::Uuid;

use crate::error::{Error, Result};
use crate::sdk::retry::{retry_api, BackoffParams};
use crate::sdk::types::*;
use crate::VERSION;

/// User agent string for API requests.
fn user_agent() -> String {
    format!("augment.sdk.context/{} (rust)", VERSION)
}

/// API client for Augment backend.
#[derive(Debug, Clone)]
pub struct ApiClient {
    client: Client,
    api_url: String,
    api_key: String,
    session_id: String,
    debug: bool,
}

impl ApiClient {
    /// Create a new API client.
    pub fn new(api_url: String, api_key: String, debug: bool) -> Result<Self> {
        let client = Client::builder()
            .user_agent(user_agent())
            .timeout(std::time::Duration::from_secs(300))
            .build()
            .map_err(|e| Error::Internal(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            client,
            api_url,
            api_key,
            session_id: Uuid::new_v4().to_string(),
            debug,
        })
    }

    /// Get the API URL.
    pub fn api_url(&self) -> &str {
        &self.api_url
    }

    /// Make an authenticated API request.
    async fn request<T: Serialize, R: DeserializeOwned>(
        &self,
        endpoint: &str,
        body: &T,
    ) -> Result<R> {
        let url = format!("{}/{}", self.api_url.trim_end_matches('/'), endpoint);
        let request_id = Uuid::new_v4().to_string();

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("X-Request-Session-Id", &self.session_id)
            .header("X-Request-Id", &request_id)
            .json(body)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Handle API response, extracting errors.
    async fn handle_response<R: DeserializeOwned>(&self, response: Response) -> Result<R> {
        let status = response.status();

        if !status.is_success() {
            let status_text = status.canonical_reason().unwrap_or("Unknown");
            let body = response.text().await.unwrap_or_default();
            return Err(Error::api(status.as_u16(), status_text, body));
        }

        response
            .json()
            .await
            .map_err(|e| Error::Internal(format!("Failed to parse response: {}", e)))
    }

    /// Make an API request with retry logic.
    async fn call_api_with_retry<T: Serialize + Clone, R: DeserializeOwned>(
        &self,
        endpoint: &str,
        body: &T,
    ) -> Result<R> {
        let params = BackoffParams::default();
        let body = body.clone();

        retry_api(
            || async { self.request(endpoint, &body).await },
            &params,
            self.debug,
        )
        .await
    }

    // ===== API Endpoints =====

    /// Find which blobs are unknown or not indexed.
    pub async fn find_missing(&self, blob_names: Vec<String>) -> Result<FindMissingResponse> {
        let request = FindMissingRequest {
            mem_object_names: blob_names,
        };
        self.call_api_with_retry("find-missing", &request).await
    }

    /// Upload blobs in a batch.
    pub async fn batch_upload(&self, blobs: Vec<BlobEntry>) -> Result<BatchUploadResponse> {
        let request = BatchUploadRequest { blobs };
        self.call_api_with_retry("batch-upload", &request).await
    }

    /// Create a checkpoint of the current blob state.
    pub async fn checkpoint_blobs(
        &self,
        checkpoint_id: Option<String>,
        added_blobs: Vec<BlobInfo>,
        deleted_blobs: Vec<String>,
    ) -> Result<CheckpointBlobsResponse> {
        let request = CheckpointBlobsRequest {
            blobs: CheckpointBlobs {
                checkpoint_id,
                added_blobs,
                deleted_blobs,
            },
        };
        self.call_api_with_retry("checkpoint-blobs", &request).await
    }

    /// Perform semantic codebase retrieval.
    pub async fn agent_codebase_retrieval(
        &self,
        query: &str,
        blobs: Blobs,
        max_output_length: Option<usize>,
    ) -> Result<CodebaseRetrievalResponse> {
        let mut request = CodebaseRetrievalRequest {
            information_request: query.to_string(),
            blobs,
            dialog: vec![],
            max_output_length: None,
        };

        if let Some(len) = max_output_length {
            request.max_output_length = Some(len);
        }

        self.call_api_with_retry("agents/codebase-retrieval", &request)
            .await
    }

    /// Chat with the AI using SSE streaming.
    pub async fn chat_stream(&self, prompt: &str, blobs: Blobs) -> Result<String> {
        let url = format!("{}/chat-stream", self.api_url.trim_end_matches('/'));
        let request_id = Uuid::new_v4().to_string();

        let request = ChatStreamRequest {
            prompt: prompt.to_string(),
            blobs,
            dialog: vec![],
        };

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Accept", "text/event-stream")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("X-Request-Session-Id", &self.session_id)
            .header("X-Request-Id", &request_id)
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let status_text = status.canonical_reason().unwrap_or("Unknown");
            let body = response.text().await.unwrap_or_default();
            return Err(Error::api(status.as_u16(), status_text, body));
        }

        // Parse SSE stream and collect response
        let body = response.text().await?;
        let mut result = String::new();

        for line in body.lines() {
            if line.starts_with("data: ") {
                let data = &line[6..];
                if data == "[DONE]" {
                    break;
                }
                if let Ok(event) = serde_json::from_str::<ChatStreamEvent>(data) {
                    if let Some(content) = event.content {
                        result.push_str(&content);
                    }
                }
            }
        }

        Ok(result)
    }
}
