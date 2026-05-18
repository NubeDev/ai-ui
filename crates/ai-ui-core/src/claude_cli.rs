//! Reference [`Provider`] implementation: local `claude` CLI via
//! `claude-wrapper`. Behind `feature = "claude-cli"`.

use std::path::PathBuf;
use std::sync::Arc;

use ai_ui_types::ChatMessage;
use claude_wrapper::{Claude, OutputFormat, QueryCommand};
use serde_json::json;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

use crate::provider::{ChatStream, Provider, ProviderContext, ProviderError, SseChunk};

#[derive(Clone)]
pub struct ClaudeCliProvider {
    inner: Arc<ClaudeCliInner>,
}

struct ClaudeCliInner {
    binary: PathBuf,
}

impl ClaudeCliProvider {
    pub fn new<P: Into<PathBuf>>(binary: P) -> Self {
        Self {
            inner: Arc::new(ClaudeCliInner {
                binary: binary.into(),
            }),
        }
    }

    /// Discover the `claude` binary via env / PATH / common install locations.
    /// Returns `None` if it isn't found.
    pub fn from_env() -> Option<Self> {
        discover_claude_binary().map(Self::new)
    }
}

impl Provider for ClaudeCliProvider {
    fn stream_chat(&self, ctx: ProviderContext, messages: Vec<ChatMessage>) -> ChatStream {
        let binary = self.inner.binary.clone();
        let system_prompt = ctx.system_prompt;

        // Build the user prompt from the last user message (the CLI takes one
        // prompt at a time).
        let user_prompt = messages
            .iter()
            .rev()
            .find(|m| m.role == "user")
            .map(|m| match &m.content {
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            })
            .unwrap_or_default();

        let (tx, rx) = mpsc::channel::<Result<SseChunk, ProviderError>>(64);
        let chat_id = format!("chatcmpl-{}", uuid::Uuid::new_v4());

        tokio::task::spawn_blocking(move || {
            let rt = tokio::runtime::Handle::current();
            let chat_id = chat_id;
            rt.block_on(async move {
                let claude = match Claude::builder().binary(binary).build() {
                    Ok(c) => Arc::new(c),
                    Err(e) => {
                        let _ = tx
                            .send(Err(ProviderError::Unavailable(e.to_string())))
                            .await;
                        return;
                    }
                };
                let cmd = QueryCommand::new(&user_prompt)
                    .output_format(OutputFormat::StreamJson)
                    .system_prompt(&system_prompt);

                let tx2 = tx.clone();
                let chat_id2 = chat_id.clone();
                let result = claude_wrapper::streaming::stream_query(&claude, &cmd, move |ev| {
                    let etype = ev.event_type().unwrap_or("");
                    match etype {
                        "assistant" => {
                            if let Some(blocks) = ev.data["message"]["content"].as_array() {
                                for block in blocks {
                                    if block["type"].as_str() == Some("text") {
                                        let text = block["text"].as_str().unwrap_or("");
                                        if text.is_empty() {
                                            continue;
                                        }
                                        let chunk = json!({
                                            "id": chat_id2,
                                            "object": "chat.completion.chunk",
                                            "choices": [{
                                                "index": 0,
                                                "delta": { "content": text },
                                                "finish_reason": null
                                            }]
                                        });
                                        let _ = tx2.try_send(Ok(SseChunk::from_json(&chunk)));
                                    }
                                }
                            }
                        }
                        "result" => {
                            let stop = json!({
                                "id": chat_id2,
                                "object": "chat.completion.chunk",
                                "choices": [{
                                    "index": 0,
                                    "delta": {},
                                    "finish_reason": "stop"
                                }]
                            });
                            let _ = tx2.try_send(Ok(SseChunk::from_json(&stop)));
                            let _ = tx2.try_send(Ok(SseChunk::done()));
                        }
                        _ => {}
                    }
                })
                .await;

                if let Err(e) = result {
                    let _ = tx.send(Err(ProviderError::Other(e.to_string()))).await;
                }
            });
        });

        Box::pin(ReceiverStream::new(rx))
    }
}

// ---------------------------------------------------------------------------
// Claude binary discovery — ported from the openui-poc / ai-runner.
// ---------------------------------------------------------------------------

fn discover_claude_binary() -> Option<PathBuf> {
    if let Ok(v) = std::env::var("CLAUDE_BINARY") {
        let v = v.trim();
        if !v.is_empty() {
            return Some(PathBuf::from(v));
        }
    }
    if let Some(p) = find_on_path("claude") {
        return Some(p);
    }
    if let Some(home) = std::env::var_os("HOME").map(PathBuf::from) {
        for c in &[
            home.join(".local/bin/claude"),
            home.join(".bun/bin/claude"),
            home.join(".npm-global/bin/claude"),
            home.join(".config/npm/global/bin/claude"),
        ] {
            if c.is_file() {
                return Some(c.clone());
            }
        }
        for root in [
            home.join(".vscode/extensions"),
            home.join(".vscode-server/extensions"),
            home.join(".cursor/extensions"),
        ] {
            if let Some(p) = scan_vscode_extensions(&root) {
                return Some(p);
            }
        }
    }
    for sys in ["/opt/homebrew/bin/claude", "/usr/local/bin/claude"] {
        let p = PathBuf::from(sys);
        if p.is_file() {
            return Some(p);
        }
    }
    None
}

fn find_on_path(name: &str) -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        let full = dir.join(name);
        if is_executable_file(&full) {
            return Some(full);
        }
    }
    None
}

fn is_executable_file(p: &std::path::Path) -> bool {
    if !p.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::metadata(p)
            .map(|m| m.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
    }
    #[cfg(not(unix))]
    {
        true
    }
}

fn scan_vscode_extensions(root: &std::path::Path) -> Option<PathBuf> {
    let rd = std::fs::read_dir(root).ok()?;
    let mut best: Option<(String, PathBuf)> = None;
    for entry in rd.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if !name.starts_with("anthropic.claude-code-") {
            continue;
        }
        let bin = entry.path().join("resources/native-binary/claude");
        if !is_executable_file(&bin) {
            continue;
        }
        if best.as_ref().map(|(n, _)| name > *n).unwrap_or(true) {
            best = Some((name, bin));
        }
    }
    best.map(|(_, p)| p)
}
