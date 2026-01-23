//! LLM provider abstraction.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::Result;

/// Abstraction over LLM providers (Claude, GPT, etc.).
///
/// This trait allows swapping LLM backends without changing workflow code.
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Completes a prompt and returns the full response.
    ///
    /// This is a blocking call that waits for the entire response.
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse>;

    /// Completes a prompt with streaming response.
    ///
    /// Returns a stream of response chunks as they arrive.
    async fn complete_streaming(&self, request: CompletionRequest) -> Result<CompletionStream>;
}

/// A request to complete a prompt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    /// System prompt (context/instructions)
    pub system_prompt: Option<String>,

    /// Conversation messages
    pub messages: Vec<Message>,

    /// Maximum tokens to generate
    pub max_tokens: u32,

    /// Temperature (0.0 = deterministic, 1.0 = creative)
    pub temperature: Option<f32>,

    /// Stop sequences
    pub stop_sequences: Vec<String>,
}

impl CompletionRequest {
    /// Creates a new completion request with default settings.
    pub fn new(messages: Vec<Message>) -> Self {
        Self {
            system_prompt: None,
            messages,
            max_tokens: 1024,
            temperature: None,
            stop_sequences: Vec::new(),
        }
    }

    /// Sets the system prompt.
    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }

    /// Sets the maximum tokens.
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    /// Sets the temperature.
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Adds a stop sequence.
    pub fn with_stop_sequence(mut self, sequence: impl Into<String>) -> Self {
        self.stop_sequences.push(sequence.into());
        self
    }
}

/// A message in the conversation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    /// Role of the message sender
    pub role: Role,

    /// Message content
    pub content: String,
}

impl Message {
    /// Creates a user message.
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: content.into(),
        }
    }

    /// Creates an assistant message.
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: content.into(),
        }
    }
}

/// Role of a message sender.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// User message
    User,
    /// Assistant message
    Assistant,
}

/// Response from an LLM completion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    /// Generated content
    pub content: String,

    /// Token usage statistics
    pub tokens_used: TokenUsage,

    /// Why the model stopped generating
    pub stop_reason: StopReason,
}

/// Token usage statistics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenUsage {
    /// Input tokens consumed
    pub input: u64,

    /// Output tokens generated
    pub output: u64,
}

impl TokenUsage {
    /// Total tokens used (input + output).
    pub fn total(&self) -> u64 {
        self.input + self.output
    }
}

/// Reason why the model stopped generating.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum StopReason {
    /// Reached the end of the response naturally
    EndTurn,

    /// Hit the maximum token limit
    MaxTokens,

    /// Encountered a stop sequence
    StopSequence,
}

/// Streaming response from an LLM completion.
///
/// This is a placeholder for now; full implementation in Phase 3.
pub struct CompletionStream {
    // Future: implement streaming using tokio::sync::mpsc or similar
    _private: (),
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_message_constructors() {
        let user_msg = Message::user("Hello");
        assert_eq!(user_msg.role, Role::User);
        assert_eq!(user_msg.content, "Hello");

        let asst_msg = Message::assistant("Hi there");
        assert_eq!(asst_msg.role, Role::Assistant);
        assert_eq!(asst_msg.content, "Hi there");
    }

    #[test]
    fn test_completion_request_builder() {
        let request = CompletionRequest::new(vec![Message::user("Test")])
            .with_system_prompt("You are helpful")
            .with_max_tokens(2048)
            .with_temperature(0.7)
            .with_stop_sequence("\n\n");

        assert_eq!(request.system_prompt, Some("You are helpful".to_string()));
        assert_eq!(request.max_tokens, 2048);
        assert_eq!(request.temperature, Some(0.7));
        assert_eq!(request.stop_sequences, vec!["\n\n"]);
    }

    #[test]
    fn test_token_usage_total() {
        let usage = TokenUsage {
            input: 100,
            output: 200,
        };
        assert_eq!(usage.total(), 300);
    }

    #[test]
    fn test_message_serialization() {
        let msg = Message::user("test content");
        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: Message = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, deserialized);
    }
}
