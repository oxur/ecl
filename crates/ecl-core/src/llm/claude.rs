//! Claude API provider implementation.

use async_trait::async_trait;

use super::provider::{
    CompletionRequest, CompletionResponse, CompletionStream, LlmProvider, StopReason, TokenUsage,
};
use crate::{Error, Result};

/// LLM provider using Anthropic's Claude API.
pub struct ClaudeProvider {
    api_key: String,
    model: String,
    client: reqwest::Client,
}

impl ClaudeProvider {
    /// Creates a new Claude provider.
    ///
    /// # Arguments
    ///
    /// * `api_key` - Anthropic API key
    /// * `model` - Model ID (e.g., "claude-sonnet-4-20250514")
    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            model: model.into(),
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl LlmProvider for ClaudeProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        // Build Claude API request
        let mut body = serde_json::json!({
            "model": self.model,
            "max_tokens": request.max_tokens,
            "messages": request.messages,
        });

        if let Some(system) = request.system_prompt {
            body["system"] = serde_json::json!(system);
        }

        if let Some(temp) = request.temperature {
            body["temperature"] = serde_json::json!(temp);
        }

        if !request.stop_sequences.is_empty() {
            body["stop_sequences"] = serde_json::json!(request.stop_sequences);
        }

        // Make API request
        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::llm_with_source("Failed to call Claude API", e))?;

        // Check for errors
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Error::llm(format!(
                "Claude API error {}: {}",
                status, error_text
            )));
        }

        // Parse response
        let response_body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| Error::llm_with_source("Failed to parse Claude response", e))?;

        // Extract content
        let content = response_body["content"][0]["text"]
            .as_str()
            .ok_or_else(|| Error::llm("Missing content in Claude response"))?
            .to_string();

        // Extract token usage
        let usage = response_body["usage"]
            .as_object()
            .ok_or_else(|| Error::llm("Missing usage data in Claude response"))?;

        let input_tokens = usage["input_tokens"]
            .as_u64()
            .ok_or_else(|| Error::llm("Invalid input_tokens"))?;
        let output_tokens = usage["output_tokens"]
            .as_u64()
            .ok_or_else(|| Error::llm("Invalid output_tokens"))?;

        // Extract stop reason
        let stop_reason_str = response_body["stop_reason"]
            .as_str()
            .ok_or_else(|| Error::llm("Missing stop_reason"))?;

        let stop_reason = match stop_reason_str {
            "end_turn" => StopReason::EndTurn,
            "max_tokens" => StopReason::MaxTokens,
            "stop_sequence" => StopReason::StopSequence,
            other => return Err(Error::llm(format!("Unknown stop reason: {}", other))),
        };

        Ok(CompletionResponse {
            content,
            tokens_used: TokenUsage {
                input: input_tokens,
                output: output_tokens,
            },
            stop_reason,
        })
    }

    async fn complete_streaming(&self, _request: CompletionRequest) -> Result<CompletionStream> {
        // Streaming implementation deferred to Phase 3
        Err(Error::llm("Streaming not yet implemented"))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::llm::Message;

    #[test]
    fn test_claude_provider_construction() {
        let provider = ClaudeProvider::new("test-key", "claude-3-opus");
        assert_eq!(provider.api_key, "test-key");
        assert_eq!(provider.model, "claude-3-opus");
    }

    // Integration test (requires API key, run manually)
    #[tokio::test]
    #[ignore]
    #[allow(clippy::expect_used)]
    async fn test_claude_provider_integration() {
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .expect("ANTHROPIC_API_KEY must be set for integration tests");

        let provider = ClaudeProvider::new(api_key, "claude-sonnet-4-20250514");

        let request = CompletionRequest::new(vec![Message::user("Say hello")]).with_max_tokens(100);

        let response = provider.complete(request).await.unwrap();

        assert!(!response.content.is_empty());
        assert!(response.tokens_used.output > 0);
    }
}
