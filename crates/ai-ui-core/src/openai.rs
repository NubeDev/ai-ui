//! Reference [`Provider`] implementation: passthrough to any OpenAI-compatible
//! HTTP API. Behind `feature = "openai-proxy"`.

use std::sync::Arc;

use ai_ui_types::ChatMessage;
use futures::stream::{StreamExt, TryStreamExt};
use serde_json::json;

use crate::provider::{ChatStream, Provider, ProviderContext, ProviderError, SseChunk};

/// OpenAI-compatible reference provider. Cheap to clone.
#[derive(Clone)]
pub struct OpenAiProvider {
    inner: Arc<OpenAiInner>,
}

struct OpenAiInner {
    http: reqwest::Client,
    api_key: String,
    base_url: String,
    model: String,
}

impl OpenAiProvider {
    pub fn new(api_key: impl Into<String>, base_url: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            inner: Arc::new(OpenAiInner {
                http: reqwest::Client::new(),
                api_key: api_key.into(),
                base_url: base_url.into(),
                model: model.into(),
            }),
        }
    }

    /// Build from env: `OPENAI_API_KEY` (required), `OPENAI_BASE_URL`
    /// (default `https://api.openai.com/v1`), `OPENAI_MODEL` (default
    /// `gpt-4o-mini`).
    pub fn from_env() -> Result<Self, std::env::VarError> {
        let api_key = std::env::var("OPENAI_API_KEY")?;
        let base_url = std::env::var("OPENAI_BASE_URL")
            .unwrap_or_else(|_| "https://api.openai.com/v1".into());
        let model = std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o-mini".into());
        Ok(Self::new(api_key, base_url, model))
    }
}

impl Provider for OpenAiProvider {
    fn stream_chat(&self, ctx: ProviderContext, messages: Vec<ChatMessage>) -> ChatStream {
        let inner = self.inner.clone();
        let stream = async_stream::try_stream! {
            let mut payload_messages = vec![json!({
                "role": "system",
                "content": ctx.system_prompt,
            })];
            for m in &messages {
                payload_messages.push(json!({
                    "role": m.role,
                    "content": m.content,
                }));
            }

            let url = format!("{}/chat/completions", inner.base_url);
            let resp = inner.http
                .post(&url)
                .bearer_auth(&inner.api_key)
                .json(&json!({
                    "model": inner.model,
                    "messages": payload_messages,
                    "stream": true,
                }))
                .send()
                .await
                .map_err(|e| ProviderError::Unavailable(e.to_string()))?;

            if !resp.status().is_success() {
                let status = resp.status().as_u16();
                let body = resp.text().await.unwrap_or_default();
                Err(ProviderError::Upstream { status, body })?;
                return;
            }

            // Upstream is already framed `data: {...}\n\n`. We need to strip
            // the framing so the route handler can re-frame uniformly.
            let mut bytes_stream = resp.bytes_stream();
            let mut buf = Vec::<u8>::new();
            while let Some(chunk) = bytes_stream.next().await {
                let chunk = chunk.map_err(|e| ProviderError::Other(e.to_string()))?;
                buf.extend_from_slice(&chunk);
                // Split on `\n\n` boundaries; emit one SseChunk per SSE event.
                while let Some(pos) = find_double_newline(&buf) {
                    let event = buf.drain(..pos + 2).collect::<Vec<u8>>();
                    let event_str = std::str::from_utf8(&event).unwrap_or("");
                    for line in event_str.lines() {
                        if let Some(payload) = line.strip_prefix("data: ") {
                            let payload = payload.trim();
                            if payload.is_empty() { continue; }
                            yield SseChunk(bytes::Bytes::copy_from_slice(payload.as_bytes()));
                        }
                    }
                }
            }
        };
        Box::pin(stream.into_stream().map(|r| r))
    }
}

fn find_double_newline(buf: &[u8]) -> Option<usize> {
    buf.windows(2).position(|w| w == b"\n\n")
}
