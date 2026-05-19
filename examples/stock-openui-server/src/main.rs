//! Runnable example: a thin axum binary that wires `ai-ui-core` (state +
//! Provider trait) + `ai-ui-axum` (HTTP routes) with whichever reference
//! provider is enabled by features.
//!
//! ```bash
//! # OpenAI-compatible
//! cd examples/stock-openui-server
//! OPENAI_API_KEY=sk-... cargo run --features openai-proxy
//!
//! # Local claude CLI
//! cargo run --features claude-cli
//! ```

use std::path::PathBuf;

use ai_ui_axum as router;
use ai_ui_core::AiUiState;
use ai_ui_skills::SkillRegistry;
use axum::Router;
use tower_http::{cors::CorsLayer, services::ServeDir};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // Skills directory, relative to repo root by default.
    let skills_dir = std::env::var("AI_UI_SKILLS_DIR").unwrap_or_else(|_| "../../skills".into());
    let skills = SkillRegistry::load_dir(&skills_dir)?;
    tracing::info!(count = skills.len(), dir = %skills_dir, "skills loaded");

    let mut builder = AiUiState::builder().skills(skills);

    // Prefer the React-generated, parser-correct system prompt over the
    // Rust-side prompt assembler. The CLI in @nube/ai-ui-react writes this
    // file as part of `pnpm build` / `pnpm dev`.
    //
    // Explicit override (env var set) → hard error if the file is missing;
    // a misconfigured path should not silently downgrade to fallback.
    // Implicit default (env var unset) → warn-and-continue is fine because
    // the default path may legitimately not exist yet (fresh checkout, no
    // `pnpm build` run).
    let (lib_prompt_path, explicit) = match std::env::var("AI_UI_LIBRARY_PROMPT") {
        Ok(p) => (p, true),
        Err(_) => ("ui/src/generated/components-prompt.txt".into(), false),
    };
    if std::path::Path::new(&lib_prompt_path).exists() {
        builder = builder.library_prompt_file(PathBuf::from(&lib_prompt_path));
        tracing::info!(path = %lib_prompt_path, "library prompt loaded");
    } else if explicit {
        return Err(format!(
            "AI_UI_LIBRARY_PROMPT is set to {lib_prompt_path:?} but no such file exists"
        )
        .into());
    } else {
        tracing::warn!(path = %lib_prompt_path, "no library prompt found; falling back to component manifest rendering");
    }

    let manifest_path = std::env::var("AI_UI_COMPONENT_MANIFEST")
        .unwrap_or_else(|_| "ui/src/generated/components.json".into());
    if std::path::Path::new(&manifest_path).exists() {
        builder = builder.component_manifest_file(PathBuf::from(&manifest_path));
        tracing::info!(path = %manifest_path, "component manifest loaded");
    } else {
        tracing::warn!(path = %manifest_path, "no component manifest found");
    }

    let builder = select_provider(builder)?;
    let state = builder.build()?;

    let ui_dir = PathBuf::from("ui/dist");
    let serve_ui = ServeDir::new(ui_dir.clone()).append_index_html_on_directories(true);

    let app = Router::new()
        .merge(router::chat_routes(state.clone()))
        .merge(router::push_routes(state.clone()))
        .merge(router::manifest_routes(state))
        .layer(CorsLayer::permissive())
        .fallback_service(serve_ui);

    let addr = "0.0.0.0:3001";
    tracing::info!("listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

// --- Provider selection is feature-gated -----------------------------------

#[cfg(feature = "openai-proxy")]
fn select_provider(
    builder: ai_ui_core::AiUiStateBuilder,
) -> Result<ai_ui_core::AiUiStateBuilder, Box<dyn std::error::Error>> {
    let provider =
        ai_ui_core::openai::OpenAiProvider::from_env().map_err(|_| "OPENAI_API_KEY is not set")?;
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
