//! Integration test: load the real `components.json` shipped by
//! `packages/ai-ui-react-shadcn/dist/components.json` and assert the prompt
//! assembler produces a prompt that mentions every component.
//!
//! This is the cross-language contract: if the React side regenerates
//! `components.json` and breaks the shape, this test catches it.

use ai_ui_prompt::{load_manifest, PromptBuilder};
use ai_ui_skills::SkillRegistry;

fn shadcn_manifest_path() -> std::path::PathBuf {
    // tests/ -> crates/ai-ui-prompt/ -> crates/ -> repo root
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    std::path::Path::new(manifest_dir)
        .join("..")
        .join("..")
        .join("packages")
        .join("ai-ui-react-shadcn")
        .join("dist")
        .join("components.json")
}

#[test]
fn loads_shadcn_manifest_and_renders_every_component() {
    let path = shadcn_manifest_path();
    let manifest = load_manifest(&path).expect("shadcn components.json should load");
    assert!(
        manifest.components.len() >= 10,
        "expected at least 10 components in the catalogue, got {}",
        manifest.components.len()
    );

    // Every Stage-5 core component must be present.
    let expected = [
        "Page",
        "Card",
        "KpiTile",
        "DataTable",
        "LineChart",
        "BarChart",
        "Form",
        "Input",
        "Select",
        "Button",
    ];
    for name in expected {
        assert!(
            manifest.components.iter().any(|c| c.name == name),
            "missing core component `{}` in shadcn catalogue",
            name
        );
    }

    let prompt = PromptBuilder::new().components(manifest).build();
    for name in expected {
        assert!(
            prompt.contains(&format!("## {name}")),
            "prompt missing section for `{name}`"
        );
    }
    // The preamble should be inlined too.
    assert!(prompt.contains("shadcn/ui-backed"), "preamble missing");
}

#[test]
fn shadcn_manifest_works_with_repo_skills() {
    let manifest =
        load_manifest(shadcn_manifest_path()).expect("shadcn components.json should load");
    let skills_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("skills");
    let skills = SkillRegistry::load_dir(&skills_dir).expect("skills dir should load");

    let prompt = PromptBuilder::new()
        .components(manifest)
        .skills(&skills)
        .build();

    assert!(prompt.contains("# COMPONENTS"));
    assert!(prompt.contains("# SKILLS"));
    // Specific skill bodies should appear.
    assert!(prompt.contains("## Skill: iot-dashboard"));
    assert!(prompt.contains("## Skill: scope-preview"));
}
