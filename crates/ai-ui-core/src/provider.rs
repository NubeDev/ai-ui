//! The provider abstraction — one method, async, streams SSE chunks.

use std::pin::Pin;

use ai_ui_types::ChatMessage;
use futures::Stream;

/// One SSE chunk as it will be written to the wire. The consumer should
/// produce chunks that match the OpenAI Chat Completions streaming format
/// (`data: {...}\n\n` framing handled by the server), because the frontend's
/// OpenUI adapter parses that format.
///
/// Producing the OpenAI-compatible JSON is the *provider's* job — the server
/// only wraps it with the SSE framing.
#[derive(Debug, Clone)]
pub struct SseChunk(pub bytes::Bytes);

impl SseChunk {
    /// Convenience: build a chunk from a `serde_json::Value` that represents
    /// one OpenAI streaming chunk (the bit between `data: ` and `\n\n`).
    pub fn from_json(value: &serde_json::Value) -> Self {
        let s = serde_json::to_string(value).expect("Value is always serializable");
        Self(bytes::Bytes::from(s))
    }

    /// The terminator chunk: `[DONE]`. Most providers will want to emit this
    /// last so the OpenAI-compatible client closes the stream cleanly.
    pub fn done() -> Self {
        Self(bytes::Bytes::from_static(b"[DONE]"))
    }
}

/// Errors a provider may return before it starts streaming.
#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("provider unavailable: {0}")]
    Unavailable(String),
    #[error("upstream returned {status}: {body}")]
    Upstream { status: u16, body: String },
    #[error("provider error: {0}")]
    Other(String),
}

/// Context the provider receives alongside the messages.
#[derive(Debug, Clone)]
pub struct ProviderContext {
    /// The fully-assembled system prompt (base + components + skills).
    pub system_prompt: String,
}

/// Boxed stream type — most providers will return a different concrete stream
/// type, so the trait object pins them to a uniform shape.
pub type ChatStream = Pin<Box<dyn Stream<Item = Result<SseChunk, ProviderError>> + Send>>;

/// The one trait a consumer implements.
///
/// Implementations should be cheap to clone (e.g. an `Arc` inside) because
/// the chat route hands a clone to each request.
pub trait Provider: Send + Sync + 'static {
    /// Run a chat request and return a stream of SSE chunks.
    ///
    /// The provider is responsible for:
    /// - Prepending the `ctx.system_prompt` to the conversation appropriately
    ///   for its API (a `role: system` message, a CLI flag, etc.).
    /// - Emitting OpenAI-compatible streaming JSON chunks.
    /// - Emitting a final `[DONE]` chunk via [`SseChunk::done`].
    fn stream_chat(&self, ctx: ProviderContext, messages: Vec<ChatMessage>) -> ChatStream;
}
