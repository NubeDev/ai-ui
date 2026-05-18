//! Assemble the system prompt that the host model receives.
//!
//! The output is a deterministic concatenation of:
//!
//! 1. **Base** — the OpenUI Lang syntax preamble. Either the baked-in
//!    [`OPENUI_BASE`] string, or a consumer-supplied override.
//! 2. **Components** — the registered component manifest, rendered as a
//!    human-readable list.
//! 3. **Skills** — every loaded skill's body, in alphabetical order by name.
//!
//! The ordering is stable so prompt-caching (Anthropic prompt cache, OpenAI
//! cached-prefix) stays warm across requests.

use ai_ui_skills::{Skill, SkillRegistry};
use ai_ui_types::{ComponentEntry, ComponentManifest};

/// Baked-in OpenUI Lang syntax preamble. Used when the consumer does not
/// override [`PromptBuilder::base`].
pub const OPENUI_BASE: &str = include_str!("openui-base.txt");

/// Builder for the assembled system prompt.
///
/// Cheap to clone; cheaper to assemble again on every request (it's just
/// string concatenation), so the server can rebuild it per-request when the
/// consumer wants per-user skill selection.
#[derive(Debug, Clone)]
pub struct PromptBuilder {
    base: String,
    manifest: ComponentManifest,
    skills: Vec<Skill>,
}

impl Default for PromptBuilder {
    fn default() -> Self {
        Self {
            base: OPENUI_BASE.to_string(),
            manifest: ComponentManifest::default(),
            skills: Vec::new(),
        }
    }
}

impl PromptBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Override the baked-in OpenUI base preamble. The default is [`OPENUI_BASE`].
    pub fn base(mut self, base: impl Into<String>) -> Self {
        self.base = base.into();
        self
    }

    /// Set the component manifest (typically loaded from the React side's
    /// generated `components.json`).
    pub fn components(mut self, manifest: ComponentManifest) -> Self {
        self.manifest = manifest;
        self
    }

    /// Include all skills in the registry.
    pub fn skills(mut self, registry: &SkillRegistry) -> Self {
        self.skills = registry.skills().to_vec();
        self
    }

    /// Include only the named skills (silently drops unknown names).
    pub fn skills_subset(mut self, registry: &SkillRegistry, names: &[String]) -> Self {
        self.skills = registry
            .select(names)
            .into_iter()
            .cloned()
            .collect::<Vec<_>>();
        self
    }

    /// Assemble the final prompt string.
    pub fn build(&self) -> String {
        let mut out = String::with_capacity(
            self.base.len() + self.manifest.preamble.len() + 4096,
        );
        out.push_str(self.base.trim());
        out.push_str("\n\n");

        out.push_str("# COMPONENTS\n\n");
        if !self.manifest.preamble.trim().is_empty() {
            out.push_str(self.manifest.preamble.trim());
            out.push_str("\n\n");
        }
        if self.manifest.components.is_empty() {
            out.push_str("(No components registered. The host has not generated a component manifest yet.)\n");
        } else {
            for c in &self.manifest.components {
                render_component(&mut out, c);
            }
        }

        if !self.skills.is_empty() {
            out.push_str("\n# SKILLS\n\n");
            for skill in &self.skills {
                render_skill(&mut out, skill);
            }
        }
        out
    }
}

fn render_component(out: &mut String, c: &ComponentEntry) {
    out.push_str("## ");
    out.push_str(&c.name);
    out.push('\n');
    if !c.description.trim().is_empty() {
        out.push_str(c.description.trim());
        out.push('\n');
    }
    if !c.props.trim().is_empty() {
        out.push_str("\nProps:\n");
        out.push_str(c.props.trim());
        out.push('\n');
    }
    if let Some(example) = &c.example {
        if !example.trim().is_empty() {
            out.push_str("\nExample:\n```openui\n");
            out.push_str(example.trim());
            out.push_str("\n```\n");
        }
    }
    out.push('\n');
}

fn render_skill(out: &mut String, skill: &Skill) {
    out.push_str("## Skill: ");
    out.push_str(&skill.manifest.name);
    out.push('\n');
    out.push_str(skill.manifest.description.trim());
    out.push('\n');
    if !skill.manifest.components.is_empty() {
        out.push_str("\nPreferred components: ");
        out.push_str(&skill.manifest.components.join(", "));
        out.push('\n');
    }
    out.push('\n');
    out.push_str(skill.manifest.body.trim());
    out.push_str("\n\n");
}

#[derive(Debug, thiserror::Error)]
pub enum PromptError {
    #[error("io error reading {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("invalid component manifest at {path}: {source}")]
    Manifest {
        path: String,
        #[source]
        source: serde_json::Error,
    },
}

/// Load a component manifest from a JSON file on disk.
///
/// Typical layout (the React side regenerates this on `npm run generate:components`):
/// ```json
/// { "name": "shadcn-dashboard",
///   "preamble": "...",
///   "components": [ {"name": "KpiTile", "description": "...", "props": "..."} ] }
/// ```
pub fn load_manifest<P: AsRef<std::path::Path>>(path: P) -> Result<ComponentManifest, PromptError> {
    let path = path.as_ref();
    let raw = std::fs::read_to_string(path).map_err(|source| PromptError::Io {
        path: path.display().to_string(),
        source,
    })?;
    serde_json::from_str(&raw).map_err(|source| PromptError::Manifest {
        path: path.display().to_string(),
        source,
    })
}

/// Resolve a prompt-file path from `env` with a fallback. Returns the file
/// contents, or `Err` if the env-pointed file does not exist (the fallback
/// path missing is OK — we just return the empty string).
pub fn load_prompt_file_with_env(env_var: &str, fallback: &str) -> std::io::Result<Option<String>> {
    let path = std::env::var(env_var).ok().unwrap_or_else(|| fallback.to_string());
    let p = std::path::Path::new(&path);
    if !p.exists() {
        return Ok(None);
    }
    std::fs::read_to_string(p).map(Some)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_build_includes_base_and_empty_components_marker() {
        let p = PromptBuilder::new().build();
        assert!(p.contains("OpenUI Lang"));
        assert!(p.contains("# COMPONENTS"));
        assert!(p.contains("(No components registered."));
    }

    #[test]
    fn renders_components_and_skills() {
        let manifest = ComponentManifest {
            name: "test".into(),
            preamble: "Shadcn primitives.".into(),
            components: vec![ComponentEntry {
                name: "KpiTile".into(),
                description: "A big number.".into(),
                props: "label: string\nvalue: number".into(),
                example: Some(r#"root = KpiTile(label="Revenue", value=42)"#.into()),
            }],
        };
        let mut reg = SkillRegistry::new();
        reg.insert(ai_ui_skills::Skill {
            manifest: ai_ui_types::SkillManifest {
                name: "demo".into(),
                description: "demo skill".into(),
                components: vec!["KpiTile".into()],
                triggers: vec![],
                body: "use a KpiTile".into(),
            },
            source: std::path::PathBuf::from("inline"),
        })
        .unwrap();

        let p = PromptBuilder::new()
            .components(manifest)
            .skills(&reg)
            .build();
        assert!(p.contains("## KpiTile"));
        assert!(p.contains("A big number."));
        assert!(p.contains("Shadcn primitives."));
        assert!(p.contains("## Skill: demo"));
        assert!(p.contains("use a KpiTile"));
    }
}
