//! `AiUiState` — what every route handler receives.

use std::sync::Arc;

use ai_ui_prompt::PromptBuilder;
use ai_ui_skills::SkillRegistry;
use ai_ui_types::{ComponentManifest, PushEvent};
use tokio::sync::broadcast;

use crate::provider::Provider;

#[derive(Debug, thiserror::Error)]
pub enum BuildError {
    #[error("no provider configured")]
    NoProvider,
    #[error(transparent)]
    Prompt(#[from] ai_ui_prompt::PromptError),
}

/// Shared state for the ai-ui axum routes.
///
/// Cheap to clone — internally `Arc`'d. Each axum handler takes
/// `State<AiUiState>` as its first argument.
#[derive(Clone)]
pub struct AiUiState {
    inner: Arc<Inner>,
}

struct Inner {
    prompt: PromptBuilder,
    skills: SkillRegistry,
    manifest: ComponentManifest,
    provider: Box<dyn Provider>,
    /// Broadcast bus for `/api/push` → `/api/events`.
    push_tx: broadcast::Sender<PushEvent>,
}

impl AiUiState {
    pub fn builder() -> AiUiStateBuilder {
        AiUiStateBuilder::default()
    }

    pub fn prompt(&self) -> &PromptBuilder {
        &self.inner.prompt
    }

    pub fn skills(&self) -> &SkillRegistry {
        &self.inner.skills
    }

    pub fn manifest(&self) -> &ComponentManifest {
        &self.inner.manifest
    }

    pub fn provider(&self) -> &dyn Provider {
        &*self.inner.provider
    }

    /// Send a push event to every connected `/api/events` subscriber. Returns
    /// the number of receivers that got the message; returns 0 silently if
    /// there are no listeners (a `POST /api/push` to a server nobody is
    /// watching is not an error).
    pub fn broadcast(&self, event: PushEvent) -> usize {
        self.inner.push_tx.send(event).unwrap_or(0)
    }

    /// Subscribe to push events. Used by the SSE route handler.
    pub fn subscribe(&self) -> broadcast::Receiver<PushEvent> {
        self.inner.push_tx.subscribe()
    }
}

#[derive(Default)]
pub struct AiUiStateBuilder {
    base_prompt: Option<String>,
    component_manifest: Option<ComponentManifest>,
    component_manifest_path: Option<std::path::PathBuf>,
    skills: SkillRegistry,
    provider: Option<Box<dyn Provider>>,
}

impl AiUiStateBuilder {
    /// Override the baked-in OpenUI Lang preamble.
    pub fn system_prompt_base(mut self, base: impl Into<String>) -> Self {
        self.base_prompt = Some(base.into());
        self
    }

    /// Set the component manifest directly.
    pub fn component_manifest(mut self, manifest: ComponentManifest) -> Self {
        self.component_manifest = Some(manifest);
        self
    }

    /// Load the component manifest from a JSON file path. If both this and
    /// `component_manifest` are called, the JSON file wins (it's the more
    /// specific intent).
    pub fn component_manifest_file<P: Into<std::path::PathBuf>>(mut self, path: P) -> Self {
        self.component_manifest_path = Some(path.into());
        self
    }

    pub fn skills(mut self, skills: SkillRegistry) -> Self {
        self.skills = skills;
        self
    }

    pub fn provider<P: Provider>(mut self, provider: P) -> Self {
        self.provider = Some(Box::new(provider));
        self
    }

    pub fn build(self) -> Result<AiUiState, BuildError> {
        let provider = self.provider.ok_or(BuildError::NoProvider)?;

        let manifest = if let Some(path) = self.component_manifest_path {
            ai_ui_prompt::load_manifest(&path)?
        } else {
            self.component_manifest.unwrap_or_default()
        };

        let mut prompt = PromptBuilder::new();
        if let Some(base) = self.base_prompt {
            prompt = prompt.base(base);
        }
        prompt = prompt.components(manifest.clone()).skills(&self.skills);

        let (push_tx, _) = broadcast::channel(64);

        Ok(AiUiState {
            inner: Arc::new(Inner {
                prompt,
                skills: self.skills,
                manifest,
                provider,
                push_tx,
            }),
        })
    }
}
