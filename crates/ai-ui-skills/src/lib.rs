//! Load `.md` files with YAML frontmatter into a `SkillRegistry`.
//!
//! Format matches Claude Code skills so a user who has already authored one
//! of those needs zero new mental model:
//!
//! ```markdown
//! ---
//! name: iot-dashboard
//! description: How to build an IoT dashboard from device telemetry.
//! components: [Page, Grid, KpiTile, LineChart]
//! triggers: [dashboard, telemetry, sensor]
//! ---
//!
//! When the user asks for an IoT dashboard, ...
//! ```
//!
//! Front-matter fields:
//! - `name` (required, unique per registry)
//! - `description` (required)
//! - `components` (optional, list of component ids to narrow the manifest to)
//! - `triggers` (optional, keyword hints for future selective loading)

use std::path::{Path, PathBuf};

use ai_ui_types::SkillManifest;
use serde::Deserialize;

#[derive(Debug, thiserror::Error)]
pub enum SkillError {
    #[error("io error reading {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("{path}: missing or malformed YAML frontmatter")]
    MissingFrontmatter { path: PathBuf },
    #[error("{path}: invalid YAML frontmatter: {source}")]
    BadFrontmatter {
        path: PathBuf,
        #[source]
        source: serde_yaml::Error,
    },
    #[error("duplicate skill name `{name}` (already loaded from {first}, also at {second})")]
    DuplicateName {
        name: String,
        first: PathBuf,
        second: PathBuf,
    },
}

#[derive(Debug, Clone, Deserialize)]
struct Frontmatter {
    name: String,
    description: String,
    #[serde(default)]
    components: Vec<String>,
    #[serde(default)]
    triggers: Vec<String>,
}

/// A loaded skill: front-matter + body + the file it came from (for diagnostics).
#[derive(Debug, Clone)]
pub struct Skill {
    pub manifest: SkillManifest,
    pub source: PathBuf,
}

/// In-memory registry. Sorted by name so prompt assembly is deterministic.
#[derive(Debug, Clone, Default)]
pub struct SkillRegistry {
    skills: Vec<Skill>,
}

impl SkillRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Load every `*.md` under `dir` (recursive). Files without YAML frontmatter
    /// are skipped with a warning rather than failing — a user dropping a stray
    /// README.md in the skills directory should not crash the server.
    pub fn load_dir<P: AsRef<Path>>(dir: P) -> Result<Self, SkillError> {
        let dir = dir.as_ref();
        let mut registry = Self::default();
        if !dir.exists() {
            tracing::warn!(path = %dir.display(), "skills directory does not exist; no skills loaded");
            return Ok(registry);
        }
        let mut paths: Vec<PathBuf> = Vec::new();
        collect_md(dir, &mut paths).map_err(|e| SkillError::Io {
            path: dir.to_path_buf(),
            source: e,
        })?;
        paths.sort();
        for path in paths {
            match Self::load_file(&path) {
                Ok(skill) => registry.insert(skill)?,
                Err(SkillError::MissingFrontmatter { path }) => {
                    tracing::warn!(path = %path.display(), "skipping: no YAML frontmatter");
                }
                Err(e) => return Err(e),
            }
        }
        registry
            .skills
            .sort_by(|a, b| a.manifest.name.cmp(&b.manifest.name));
        Ok(registry)
    }

    /// Load a single skill file.
    pub fn load_file<P: AsRef<Path>>(path: P) -> Result<Skill, SkillError> {
        let path = path.as_ref();
        let raw = std::fs::read_to_string(path).map_err(|e| SkillError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;
        let (fm, body) = split_frontmatter(&raw).ok_or_else(|| SkillError::MissingFrontmatter {
            path: path.to_path_buf(),
        })?;
        let fm: Frontmatter = serde_yaml::from_str(fm).map_err(|e| SkillError::BadFrontmatter {
            path: path.to_path_buf(),
            source: e,
        })?;
        Ok(Skill {
            manifest: SkillManifest {
                name: fm.name,
                description: fm.description,
                components: fm.components,
                triggers: fm.triggers,
                body: body.trim().to_string(),
            },
            source: path.to_path_buf(),
        })
    }

    pub fn insert(&mut self, skill: Skill) -> Result<(), SkillError> {
        if let Some(existing) = self
            .skills
            .iter()
            .find(|s| s.manifest.name == skill.manifest.name)
        {
            return Err(SkillError::DuplicateName {
                name: skill.manifest.name.clone(),
                first: existing.source.clone(),
                second: skill.source.clone(),
            });
        }
        self.skills.push(skill);
        Ok(())
    }

    pub fn skills(&self) -> &[Skill] {
        &self.skills
    }

    pub fn is_empty(&self) -> bool {
        self.skills.is_empty()
    }

    pub fn len(&self) -> usize {
        self.skills.len()
    }

    pub fn get(&self, name: &str) -> Option<&Skill> {
        self.skills.iter().find(|s| s.manifest.name == name)
    }

    /// Returns the subset of skills whose name appears in `names`. Order
    /// preserved from the registry (alphabetical).
    pub fn select(&self, names: &[String]) -> Vec<&Skill> {
        self.skills
            .iter()
            .filter(|s| names.iter().any(|n| n == &s.manifest.name))
            .collect()
    }
}

fn collect_md(dir: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_md(&path, out)?;
        } else if path.extension().is_some_and(|e| e == "md") {
            out.push(path);
        }
    }
    Ok(())
}

/// Parse the YAML frontmatter block from a markdown file.
///
/// Returns `(frontmatter_yaml, body)` or `None` if the file does not start
/// with a `---` fence.
fn split_frontmatter(raw: &str) -> Option<(&str, &str)> {
    let raw = raw.strip_prefix('\u{FEFF}').unwrap_or(raw); // BOM
    let rest = raw.strip_prefix("---")?;
    let rest = rest
        .strip_prefix('\n')
        .or_else(|| rest.strip_prefix("\r\n"))?;
    // Find the closing fence at the start of a line.
    let mut idx = 0;
    for line in rest.split_inclusive('\n') {
        let trimmed = line.trim_end_matches(['\r', '\n']);
        if trimmed == "---" || trimmed == "..." {
            let fm = &rest[..idx];
            let body_start = idx + line.len();
            let body = &rest[body_start..];
            return Some((fm, body));
        }
        idx += line.len();
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_frontmatter() {
        let raw = "---\nname: foo\ndescription: hi\n---\nbody text\n";
        let (fm, body) = split_frontmatter(raw).expect("frontmatter present");
        assert!(fm.contains("name: foo"));
        assert_eq!(body.trim(), "body text");
    }

    #[test]
    fn no_frontmatter() {
        assert!(split_frontmatter("no fence here\n").is_none());
    }

    #[test]
    fn loads_skill_from_string() {
        let dir = tempdir();
        let path = dir.join("test.md");
        std::fs::write(
            &path,
            "---\nname: iot\ndescription: dashboards\ncomponents: [Page, Grid]\n---\n\ndo a thing\n",
        )
        .unwrap();
        let skill = SkillRegistry::load_file(&path).unwrap();
        assert_eq!(skill.manifest.name, "iot");
        assert_eq!(skill.manifest.components, vec!["Page", "Grid"]);
        assert_eq!(skill.manifest.body, "do a thing");
    }

    #[test]
    fn duplicate_name_errors() {
        let dir = tempdir();
        let a = dir.join("a.md");
        let b = dir.join("b.md");
        std::fs::write(&a, "---\nname: dup\ndescription: x\n---\nA").unwrap();
        std::fs::write(&b, "---\nname: dup\ndescription: y\n---\nB").unwrap();
        let err = SkillRegistry::load_dir(&dir).unwrap_err();
        assert!(matches!(err, SkillError::DuplicateName { .. }));
    }

    fn tempdir() -> PathBuf {
        let p = std::env::temp_dir().join(format!(
            "ai-ui-skills-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&p).unwrap();
        p
    }
}
