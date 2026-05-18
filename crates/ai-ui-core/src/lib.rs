//! Provider trait, state, and reference provider implementations.
//!
//! This crate is HTTP-free. The axum routes for chat/push/manifest live in
//! the sister crate `ai-ui-axum`, which depends on this one. A consumer that
//! drives the provider from Tauri IPC, from a CLI, or from any other
//! transport uses `ai-ui-core` directly and never pulls axum.
//!
//! The "BYO AI" default: implement [`Provider`] against your own AI runner.
//! Two reference impls are available behind feature flags for examples and
//! quick demos:
//! - `openai-proxy` — passthrough to any OpenAI-compatible HTTP API.
//! - `claude-cli` — spawns the local `claude` binary via `claude-wrapper`.

pub mod provider;
pub mod state;

#[cfg(feature = "openai-proxy")]
pub mod openai;

#[cfg(feature = "claude-cli")]
pub mod claude_cli;

pub use provider::{ChatStream, Provider, ProviderContext, ProviderError, SseChunk};
pub use state::{AiUiState, AiUiStateBuilder, BuildError};

/// Re-export skill registry so consumers don't need a second `use` line.
pub use ai_ui_skills::{Skill, SkillRegistry};
