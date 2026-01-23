//! Retry wrapper for LLM providers.

use async_trait::async_trait;
use backon::{ExponentialBuilder, Retryable};
use std::sync::Arc;
use std::time::Duration;

use super::provider::{CompletionRequest, CompletionResponse, CompletionStream, LlmProvider};
use crate::{Error, Result};

/// Wraps an LLM provider with retry logic.
pub struct RetryWrapper {
    inner: Arc<dyn LlmProvider>,
    max_attempts: u32,
    initial_delay: Duration,
    max_delay: Duration,
}

impl RetryWrapper {
    /// Creates a new retry wrapper with default settings.
    ///
    /// Default settings:
    /// - Max attempts: 3
    /// - Initial delay: 1 second
    /// - Max delay: 10 seconds
    /// - Multiplier: 2.0 (exponential backoff)
    pub fn new(provider: Arc<dyn LlmProvider>) -> Self {
        Self {
            inner: provider,
            max_attempts: 3,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(10),
        }
    }

    /// Sets the maximum number of attempts.
    pub fn with_max_attempts(mut self, max_attempts: u32) -> Self {
        self.max_attempts = max_attempts;
        self
    }

    /// Sets the initial delay between retries.
    pub fn with_initial_delay(mut self, delay: Duration) -> Self {
        self.initial_delay = delay;
        self
    }

    /// Sets the maximum delay between retries.
    pub fn with_max_delay(mut self, delay: Duration) -> Self {
        self.max_delay = delay;
        self
    }

    /// Determines if an error should be retried.
    fn should_retry(error: &Error) -> bool {
        error.is_retryable()
    }
}

#[async_trait]
impl LlmProvider for RetryWrapper {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        let backoff = ExponentialBuilder::default()
            .with_min_delay(self.initial_delay)
            .with_max_delay(self.max_delay)
            .with_max_times(self.max_attempts as usize);

        let provider = self.inner.clone();
        let request_clone = request.clone();

        // Use backon for retry logic
        (|| async { provider.complete(request_clone.clone()).await })
            .retry(backoff)
            .when(Self::should_retry)
            .await
    }

    async fn complete_streaming(&self, request: CompletionRequest) -> Result<CompletionStream> {
        // Streaming with retry is complex, defer to Phase 3
        self.inner.complete_streaming(request).await
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::llm::MockLlmProvider;

    #[tokio::test]
    async fn test_retry_wrapper_success() {
        let mock = Arc::new(MockLlmProvider::with_response("Success"));
        let retry = RetryWrapper::new(mock);

        let request = CompletionRequest::new(vec![crate::llm::Message::user("Test")]);
        let response = retry.complete(request).await.unwrap();

        assert_eq!(response.content, "Success");
    }

    #[test]
    fn test_retry_wrapper_builder() {
        let mock = Arc::new(MockLlmProvider::with_response("Test"));
        let retry = RetryWrapper::new(mock)
            .with_max_attempts(5)
            .with_initial_delay(Duration::from_millis(500))
            .with_max_delay(Duration::from_secs(30));

        assert_eq!(retry.max_attempts, 5);
        assert_eq!(retry.initial_delay, Duration::from_millis(500));
        assert_eq!(retry.max_delay, Duration::from_secs(30));
    }
}
