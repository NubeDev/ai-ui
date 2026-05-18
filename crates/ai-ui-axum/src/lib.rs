//! Axum routes for ai-ui.
//!
//! Three router constructors:
//! - [`chat_routes`] — `POST /api/chat`
//! - [`push_routes`] — `POST /api/push`, `GET /api/events`
//! - [`manifest_routes`] — `GET /api/skills`, `GET /api/components`
//!
//! Each returns an `axum::Router` already typed with [`AiUiState`], so the
//! consumer just `.merge`s them into their own router.
//!
//! Skip this crate entirely if you don't need HTTP — `ai-ui-core` gives you
//! the [`Provider`](ai_ui_core::Provider) trait and the prompt assembler
//! directly, ready to drive from Tauri IPC, a CLI, or any other transport.

use std::convert::Infallible;
use std::time::Duration;

use axum::{
    extract::State,
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse, Response,
    },
    routing::{get, post},
    Json, Router,
};
use futures::{stream, StreamExt};
use serde_json::json;
use tokio_stream::wrappers::BroadcastStream;

use ai_ui_core::{AiUiState, ProviderContext, ProviderError};
use ai_ui_types::{ChatRequest, ComponentManifest, PushEvent, PushRequest};

/// `POST /api/chat`.
pub fn chat_routes(state: AiUiState) -> Router {
    Router::new()
        .route("/api/chat", post(chat_handler))
        .with_state(state)
}

/// `POST /api/push` and `GET /api/events`.
pub fn push_routes(state: AiUiState) -> Router {
    Router::new()
        .route("/api/push", post(push_handler))
        .route("/api/events", get(events_handler))
        .with_state(state)
}

/// `GET /api/skills` and `GET /api/components`.
pub fn manifest_routes(state: AiUiState) -> Router {
    Router::new()
        .route("/api/skills", get(skills_handler))
        .route("/api/components", get(components_handler))
        .with_state(state)
}

// ---------------------------------------------------------------------------
// /api/chat
// ---------------------------------------------------------------------------

async fn chat_handler(
    State(state): State<AiUiState>,
    Json(payload): Json<ChatRequest>,
) -> Response {
    let prompt = if payload.skills.is_empty() {
        state.prompt().build()
    } else {
        ai_ui_prompt::PromptBuilder::new()
            .components(state.manifest().clone())
            .skills_subset(state.skills(), &payload.skills)
            .build()
    };

    let ctx = ProviderContext {
        system_prompt: prompt,
    };
    let upstream = state.provider().stream_chat(ctx, payload.messages);

    // Wrap each chunk in SSE framing; errors mid-stream go out as a `data:` line
    // so the client surfaces them instead of silently seeing the conn close.
    let sse_stream = upstream.map(|chunk_result| -> Result<axum::body::Bytes, Infallible> {
        match chunk_result {
            Ok(chunk) => {
                let mut buf = bytes::BytesMut::with_capacity(chunk.0.len() + 8);
                buf.extend_from_slice(b"data: ");
                buf.extend_from_slice(&chunk.0);
                buf.extend_from_slice(b"\n\n");
                Ok(buf.freeze())
            }
            Err(e) => {
                let err = json!({ "error": format!("{e}") });
                let body = format!("data: {}\n\n", err);
                Ok(axum::body::Bytes::from(body))
            }
        }
    });

    let body = axum::body::Body::from_stream(sse_stream);
    Response::builder()
        .header("Content-Type", "text/event-stream")
        .header("Cache-Control", "no-cache, no-transform")
        .header("Connection", "keep-alive")
        .body(body)
        .unwrap()
}

/// Newtype so we can put `IntoResponse` on a `ProviderError` despite the
/// orphan rule (the trait + the error type both live in foreign crates).
pub struct ProviderErrorResponse(pub ProviderError);

impl From<ProviderError> for ProviderErrorResponse {
    fn from(e: ProviderError) -> Self {
        Self(e)
    }
}

impl IntoResponse for ProviderErrorResponse {
    fn into_response(self) -> Response {
        let status = match &self.0 {
            ProviderError::Unavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            ProviderError::Upstream { status, .. } => {
                StatusCode::from_u16(*status).unwrap_or(StatusCode::BAD_GATEWAY)
            }
            ProviderError::Other(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, self.0.to_string()).into_response()
    }
}

// ---------------------------------------------------------------------------
// /api/push  +  /api/events
// ---------------------------------------------------------------------------

async fn push_handler(
    State(state): State<AiUiState>,
    Json(payload): Json<PushRequest>,
) -> Response {
    let n = state.broadcast(payload.event);
    Json(json!({ "delivered_to": n })).into_response()
}

async fn events_handler(
    State(state): State<AiUiState>,
) -> Sse<impl futures::Stream<Item = Result<Event, Infallible>>> {
    let rx = state.subscribe();
    let stream = BroadcastStream::new(rx)
        .filter_map(|res| async move { res.ok() })
        .map(|event: PushEvent| {
            let data = serde_json::to_string(&event).unwrap_or_else(|_| "{}".into());
            Ok::<_, Infallible>(Event::default().event("push").data(data))
        });
    let stream =
        stream::once(async { Ok::<_, Infallible>(Event::default().event("ready").data("{}")) })
            .chain(stream);
    Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    )
}

// ---------------------------------------------------------------------------
// /api/skills  +  /api/components
// ---------------------------------------------------------------------------

async fn skills_handler(State(state): State<AiUiState>) -> Json<serde_json::Value> {
    let summaries: Vec<_> = state
        .skills()
        .skills()
        .iter()
        .map(|s| s.manifest.summary())
        .collect();
    Json(json!({ "skills": summaries }))
}

async fn components_handler(State(state): State<AiUiState>) -> Json<ComponentManifest> {
    Json(state.manifest().clone())
}
