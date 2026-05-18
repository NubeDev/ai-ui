//! Wire types shared by `ai-ui` crates and consumers.
//!
//! This crate intentionally has no I/O, no http client, and no async runtime.
//! Any crate (including mobile-safe ones) can depend on it without pulling in
//! axum/reqwest/tokio. See `SCOPE.md` rule R1.

use serde::{Deserialize, Serialize};

/// One message in an OpenAI-compatible chat exchange.
///
/// `content` is intentionally `serde_json::Value` because the OpenAI API
/// accepts both `"hello"` and `[{"type":"text","text":"hello"}]` — we forward
/// whatever the client sends.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: serde_json::Value,
}

/// The body of `POST /api/chat`. Mirrors the OpenAI Chat Completions request,
/// minus the bits we don't need (model, temperature, etc. — those are the
/// provider's concern).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub messages: Vec<ChatMessage>,
    /// Optional: ask the server to use a specific named skill set instead of
    /// the registry's full set. The server may ignore this.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skills: Vec<String>,
}

/// One component the model is allowed to emit, as it appears in the manifest.
///
/// This is the on-the-wire shape. The React side generates this file from its
/// component library (Zod schemas with `.describe()` calls); the Rust side
/// reads it verbatim and inlines it into the system prompt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentEntry {
    /// Component identifier as the model will emit it — e.g. `KpiTile`.
    pub name: String,
    /// One-line description of what the component is for.
    pub description: String,
    /// Prop schema, rendered as human-readable text in the prompt. Format
    /// is whatever the generator emits; we treat it as opaque text.
    #[serde(default)]
    pub props: String,
    /// Optional example snippet of OpenUI Lang using the component.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub example: Option<String>,
}

/// The full component manifest the React side generates.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComponentManifest {
    /// Pretty name shown in `/api/components` responses. e.g. `shadcn-dashboard`.
    #[serde(default)]
    pub name: String,
    /// Free-form preamble; appears once at the top of the prompt's component
    /// section (e.g. "All components render via shadcn/ui primitives").
    #[serde(default)]
    pub preamble: String,
    pub components: Vec<ComponentEntry>,
}

/// Skill front-matter as it appears on the wire (`GET /api/skills`).
///
/// The full body is included as `body`; clients that just want a directory
/// listing can use `summary()`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillManifest {
    pub name: String,
    pub description: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub components: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub triggers: Vec<String>,
    pub body: String,
}

impl SkillManifest {
    /// Drop the body — useful for listings.
    pub fn summary(&self) -> SkillSummary {
        SkillSummary {
            name: self.name.clone(),
            description: self.description.clone(),
            components: self.components.clone(),
            triggers: self.triggers.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillSummary {
    pub name: String,
    pub description: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub components: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub triggers: Vec<String>,
}

/// An event the **server** pushes to the UI over `GET /api/events` (SSE).
///
/// The UI's job: apply each event to whichever render slot the consumer's
/// frontend has wired up. There is no implied diffing — `Replace` replaces
/// the whole slot, `Append` appends.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PushEvent {
    /// Replace the named slot's contents with this OpenUI Lang.
    Replace { slot: String, openui: String },
    /// Append OpenUI Lang to the named slot's contents.
    Append { slot: String, openui: String },
    /// Clear the named slot.
    Clear { slot: String },
    /// A heartbeat so the EventSource connection stays open through proxies.
    Ping,
}

/// The body of `POST /api/push` — what a consumer's backend sends to drive
/// the UI without a chat round-trip.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushRequest {
    pub event: PushEvent,
}
