//! shadcn-dashboard example server.
//!
//! Identical wiring to `stock-openui-server` except the component manifest
//! defaults to the **shadcn catalogue** shipped at
//! `packages/ai-ui-react-shadcn/dist/components.json`.
//!
//! The React UI lives in `ui/` and is the consumer's responsibility — it
//! imports the shadcn primitives from its own `@/components/ui/*` install
//! and calls `composeLibrary({ Button, Card, ... })` to bind them to the
//! catalogue schemas. See `ui/README.md`.

use std::path::PathBuf;

use ai_ui_axum as router;
use ai_ui_core::AiUiState;
use ai_ui_skills::SkillRegistry;
use axum::Router;
use tower_http::{cors::CorsLayer, services::ServeDir};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let skills_dir = std::env::var("AI_UI_SKILLS_DIR")
        .unwrap_or_else(|_| "../../skills".into());
    let skills = SkillRegistry::load_dir(&skills_dir)?;
    tracing::info!(count = skills.len(), dir = %skills_dir, "skills loaded");

    let manifest_path = std::env::var("AI_UI_COMPONENT_MANIFEST")
        .unwrap_or_else(|_| {
            "../../packages/ai-ui-react-shadcn/dist/components.json".into()
        });

    let state = AiUiState::builder()
        .skills(skills)
        .component_manifest_file(PathBuf::from(&manifest_path));
    tracing::info!(path = %manifest_path, "shadcn component manifest loaded");
    let state = select_provider(state)?.build()?;

    let ui_dir = PathBuf::from("ui/dist");
    let serve_ui = ServeDir::new(ui_dir).append_index_html_on_directories(true);

    let app = Router::new()
        .merge(router::chat_routes(state.clone()))
        .merge(router::push_routes(state.clone()))
        .merge(router::manifest_routes(state))
        .layer(CorsLayer::permissive())
        .fallback_service(serve_ui);

    let addr = "0.0.0.0:3002";
    tracing::info!("listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

#[cfg(feature = "openai-proxy")]
fn select_provider(
    builder: ai_ui_core::AiUiStateBuilder,
) -> Result<ai_ui_core::AiUiStateBuilder, Box<dyn std::error::Error>> {
    let provider = ai_ui_core::openai::OpenAiProvider::from_env()
        .map_err(|_| "OPENAI_API_KEY is not set")?;
    tracing::info!("provider: openai-proxy");
    Ok(builder.provider(provider))
}

#[cfg(all(feature = "claude-cli", not(feature = "openai-proxy")))]
fn select_provider(
    builder: ai_ui_core::AiUiStateBuilder,
) -> Result<ai_ui_core::AiUiStateBuilder, Box<dyn std::error::Error>> {
    let provider = ai_ui_core::claude_cli::ClaudeCliProvider::from_env()
        .ok_or("no `claude` binary found — set CLAUDE_BINARY or install Claude Code CLI")?;
    tracing::info!("provider: claude-cli");
    Ok(builder.provider(provider))
}

#[cfg(not(any(feature = "openai-proxy", feature = "claude-cli")))]
fn select_provider(
    _builder: ai_ui_core::AiUiStateBuilder,
) -> Result<ai_ui_core::AiUiStateBuilder, Box<dyn std::error::Error>> {
    Err("Build with --features openai-proxy or --features claude-cli to pick a reference provider, \
         or wire your own Provider impl.".into())
}
