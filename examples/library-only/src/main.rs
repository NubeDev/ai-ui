//! Pure-library use of `ai-ui-core` — no axum, no HTTP, no port binding.
//!
//! Prints the assembled system prompt, then (if a reference provider feature
//! is enabled and configured) sends a single hard-coded user message and
//! streams the response chunks to stdout.
//!
//! This is the shape codeless, a Tauri app, or any other in-process consumer
//! would use: skill loading + prompt assembly + provider streaming, with the
//! consumer piping bytes wherever they need to go.
//!
//! Run:
//!   cargo run -p library-only                                 # prompt only
//!   OPENAI_API_KEY=sk-... cargo run -p library-only --features openai-proxy
//!   cargo run -p library-only --features claude-cli

use std::path::Path;

use ai_ui_prompt::{load_manifest, PromptBuilder};
use ai_ui_skills::SkillRegistry;

#[cfg(any(feature = "openai-proxy", feature = "claude-cli"))]
use ai_ui_core::{Provider, ProviderContext};
#[cfg(any(feature = "openai-proxy", feature = "claude-cli"))]
use ai_ui_types::ChatMessage;
#[cfg(any(feature = "openai-proxy", feature = "claude-cli"))]
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // 1. Load skills from the workspace's skills/ directory.
    let skills = SkillRegistry::load_dir("skills")?;
    eprintln!("[skills] loaded {} skill(s)", skills.len());

    // 2. Load the shadcn component manifest from the package's dist/.
    let manifest_path = Path::new("packages/ai-ui-react-shadcn/dist/components.json");
    let manifest = load_manifest(manifest_path)?;
    eprintln!(
        "[manifest] loaded {} component(s)",
        manifest.components.len()
    );

    // 3. Assemble the system prompt — pure string operation, no I/O.
    let prompt = PromptBuilder::new()
        .components(manifest)
        .skills(&skills)
        .build();

    eprintln!(
        "--- system prompt ({} chars) ----------------------------------------",
        prompt.len()
    );
    println!("{prompt}");
    eprintln!("--- end system prompt -----------------------------------------------");

    // 4. If a reference provider feature is enabled, stream a single turn.
    stream_one_turn(prompt).await
}

#[cfg(feature = "openai-proxy")]
async fn stream_one_turn(prompt: String) -> Result<(), Box<dyn std::error::Error>> {
    let provider =
        ai_ui_core::openai::OpenAiProvider::from_env().map_err(|_| "OPENAI_API_KEY is not set")?;
    drive(provider, prompt).await
}

#[cfg(all(feature = "claude-cli", not(feature = "openai-proxy")))]
async fn stream_one_turn(prompt: String) -> Result<(), Box<dyn std::error::Error>> {
    let provider =
        ai_ui_core::claude_cli::ClaudeCliProvider::from_env().ok_or("no `claude` binary found")?;
    drive(provider, prompt).await
}

#[cfg(not(any(feature = "openai-proxy", feature = "claude-cli")))]
async fn stream_one_turn(_prompt: String) -> Result<(), Box<dyn std::error::Error>> {
    eprintln!(
        "[provider] no reference provider feature enabled — printed prompt only. \
         Re-run with `--features openai-proxy` or `--features claude-cli` to also \
         stream a model response."
    );
    Ok(())
}

#[cfg(any(feature = "openai-proxy", feature = "claude-cli"))]
async fn drive<P: Provider>(provider: P, prompt: String) -> Result<(), Box<dyn std::error::Error>> {
    let user_msg = ChatMessage {
        role: "user".into(),
        content: serde_json::Value::String(
            "Build a tiny KPI dashboard with three KpiTiles: revenue, users, conversion.".into(),
        ),
    };
    let mut stream = provider.stream_chat(
        ProviderContext {
            system_prompt: prompt,
        },
        vec![user_msg],
    );

    eprintln!("--- streaming ------------------------------------------------------");
    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(c) => {
                // Print the raw chunk JSON one per line so a human can read it.
                let s = String::from_utf8_lossy(&c.0);
                println!("{s}");
            }
            Err(e) => {
                eprintln!("[provider error] {e}");
                break;
            }
        }
    }
    eprintln!("--- end stream -----------------------------------------------------");
    Ok(())
}
