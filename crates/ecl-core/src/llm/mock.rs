//! Mock LLM provider for testing.

use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::provider::{
    CompletionRequest, CompletionResponse, CompletionStream, LlmProvider, StopReason, TokenUsage,
};
use crate::Result;

/// Mock LLM provider that returns canned responses.
///
/// Useful for testing without making actual API calls.
#[derive(Clone)]
pub struct MockLlmProvider {
    responses: Arc<Mutex<MockResponses>>,
}

struct MockResponses {
    canned: Vec<String>,
    index: usize,
}

impl MockLlmProvider {
    /// Creates a new mock provider with canned responses.
    ///
    /// Responses are returned in order. After all responses are used,
    /// the provider cycles back to the first response.
    ///
    /// # Examples
    ///
    /// ```
    /// use ecl_core::llm::MockLlmProvider;
    ///
    /// let provider = MockLlmProvider::new(vec![
    ///     "First response".to_string(),
    ///     "Second response".to_string(),
    /// ]);
    /// ```
    pub fn new(responses: Vec<String>) -> Self {
        Self {
            responses: Arc::new(Mutex::new(MockResponses {
                canned: responses,
                index: 0,
            })),
        }
    }

    /// Creates a mock provider with a single response.
    pub fn with_response(response: impl Into<String>) -> Self {
        Self::new(vec![response.into()])
    }
}

#[async_trait]
impl LlmProvider for MockLlmProvider {
    async fn complete(&self, _request: CompletionRequest) -> Result<CompletionResponse> {
        let mut responses = self.responses.lock().await;

        // Get current response
        let content = responses.canned[responses.index].clone();

        // Advance to next response (cycling)
        responses.index = (responses.index + 1) % responses.canned.len();

        Ok(CompletionResponse {
            content,
            tokens_used: TokenUsage {
                input: 10, // Mock values
                output: 20,
            },
            stop_reason: StopReason::EndTurn,
        })
    }

    async fn complete_streaming(&self, _request: CompletionRequest) -> Result<CompletionStream> {
        // Streaming not implemented for mock
        Err(crate::Error::llm(
            "Streaming not supported in mock provider",
        ))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::llm::Message;

    #[tokio::test]
    async fn test_mock_provider_single_response() {
        let provider = MockLlmProvider::with_response("Test response");

        let request = CompletionRequest::new(vec![Message::user("Hello")]);

        let response = provider.complete(request).await.unwrap();
        assert_eq!(response.content, "Test response");
    }

    #[tokio::test]
    async fn test_mock_provider_multiple_responses() {
        let provider = MockLlmProvider::new(vec![
            "First".to_string(),
            "Second".to_string(),
            "Third".to_string(),
        ]);

        let request = CompletionRequest::new(vec![Message::user("Test")]);

        assert_eq!(
            provider.complete(request.clone()).await.unwrap().content,
            "First"
        );
        assert_eq!(
            provider.complete(request.clone()).await.unwrap().content,
            "Second"
        );
        assert_eq!(
            provider.complete(request.clone()).await.unwrap().content,
            "Third"
        );
        // Cycles back
        assert_eq!(
            provider.complete(request.clone()).await.unwrap().content,
            "First"
        );
    }

    #[tokio::test]
    async fn test_mock_provider_clone() {
        let provider = MockLlmProvider::with_response("Shared");
        let provider2 = provider.clone();

        let request = CompletionRequest::new(vec![Message::user("Test")]);

        // Both providers share the same state
        provider.complete(request.clone()).await.unwrap();
        // Would cycle if shared, but we have only one response so it's the same
        let response = provider2.complete(request).await.unwrap();
        assert_eq!(response.content, "Shared");
    }
}
