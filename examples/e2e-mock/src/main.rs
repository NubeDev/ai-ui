//! End-to-end test binary with a built-in mock provider.
//!
//! No API keys, no claude binary. Streams a fixed set of OpenAI-compatible
//! chunks containing a one-line OpenUI Lang reply. Used to smoke every
//! route in `ai-ui-axum` without any external network or CLI dep.

use std::path::PathBuf;

use ai_ui_axum as router;
use ai_ui_core::{AiUiState, ChatStream, Provider, ProviderContext, SseChunk};
use ai_ui_skills::SkillRegistry;
use ai_ui_types::ChatMessage;
use axum::Router;
use serde_json::json;
use tower_http::cors::CorsLayer;

/// A provider that ignores the input and streams a tiny fixed reply.
#[derive(Clone)]
struct MockProvider;

impl Provider for MockProvider {
    fn stream_chat(&self, _ctx: ProviderContext, _messages: Vec<ChatMessage>) -> ChatStream {
        let id = format!("chatcmpl-{}", uuid::Uuid::new_v4());
        let stream = async_stream::stream! {
            for delta in ["```openui\n", "root = TextContent(\"hello from mock\")\n", "```"] {
                let chunk = json!({
                    "id": id,
                    "object": "chat.completion.chunk",
                    "choices": [{
                        "index": 0,
                        "delta": { "content": delta },
                        "finish_reason": null
                    }]
                });
                yield Ok(SseChunk::from_json(&chunk));
            }
            let stop = json!({
                "id": id,
                "object": "chat.completion.chunk",
                "choices": [{ "index": 0, "delta": {}, "finish_reason": "stop" }]
            });
            yield Ok(SseChunk::from_json(&stop));
            yield Ok(SseChunk::done());
        };
        Box::pin(stream)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let skills_dir = std::env::var("AI_UI_SKILLS_DIR")
        .unwrap_or_else(|_| "../../skills".into());
    let skills = SkillRegistry::load_dir(&skills_dir)?;
    tracing::info!(count = skills.len(), dir = %skills_dir, "skills loaded");

    let manifest_path = std::env::var("AI_UI_COMPONENT_MANIFEST").unwrap_or_else(|_| {
        "../../packages/ai-ui-react-shadcn/dist/components.json".into()
    });

    let state = AiUiState::builder()
        .skills(skills)
        .component_manifest_file(PathBuf::from(&manifest_path))
        .provider(MockProvider)
        .build()?;
    tracing::info!(path = %manifest_path, "component manifest loaded");

    let app = Router::new()
        .merge(router::chat_routes(state.clone()))
        .merge(router::push_routes(state.clone()))
        .merge(router::manifest_routes(state))
        .layer(CorsLayer::permissive());

    let addr = "127.0.0.1:3099";
    tracing::info!("listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
