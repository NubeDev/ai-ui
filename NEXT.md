# NEXT.md

Two prompts to paste into a fresh Claude Code session — one per next-step.
They're self-contained: the agent does not need to have seen our prior chats.

---

## Prompt 1 — Set up CI for ai-ui

```
You're working in /home/user/code/rust/ai-ui — a Rust workspace + npm
workspace that ships a "BYO AI" UI generation toolkit. Read SCOPE.md
first — it has the hard rules R1–R6 you must protect.

Goal: add GitHub Actions CI that runs on every push and PR.

The CI must:

1. Run `cargo fmt --all -- --check` and fail on diff.
2. Run `cargo clippy --workspace --all-targets -- -D warnings` and fail on
   any warning.
3. Run `cargo test --workspace` — should be 11 passing tests today.
4. Enforce **R2** (default-features empty, no model client in core's
   default dep graph). Run:
     cargo tree -p ai-ui-core --no-default-features --prefix none
   and fail if the output contains `reqwest`, `claude-wrapper`, `axum`,
   `tower-http`, or `hyper`.
5. Enforce **R6** (ai-ui-core has no HTTP deps in its manifest). Run:
     grep -E '^(axum|tower|tower-http|http) =' crates/ai-ui-core/Cargo.toml
   and fail if anything matches.
6. Build each feature config to catch dead-code under cfg:
     cargo check -p ai-ui-core --no-default-features
     cargo check -p ai-ui-core --features openai-proxy
     cargo check -p ai-ui-core --features claude-cli
     cargo check -p ai-ui-core --features 'openai-proxy claude-cli'
7. Run the cross-language integration test that loads the React-side
   components.json from Rust (already exists at
   crates/ai-ui-prompt/tests/shadcn_manifest.rs). Just `cargo test`
   covers it.

Use Ubuntu latest, stable Rust, cache `~/.cargo` + `target`.

Skip the JS side for now (the React packages have no test runner wired
up — that's a separate task). Just lint+typecheck if it's cheap; skip
otherwise.

Show me the .github/workflows/ci.yml diff before committing. Don't
push. Don't open a PR — I'll review locally first.
```

---

## Prompt 2 — Adopt ai-ui in codeless

```
Cross-repo task. Two repos:

  source:   /home/user/code/rust/ai-ui                     (ai-ui library)
  target:   /home/user/code/rust/codeless-workspace/codeless (the consumer)

Goal: codeless replaces its in-tree openui-poc with a real dependency on
ai-ui, implementing ai-ui's `Provider` trait against codeless's existing
`ai-runner` crate. This is the validation that proves the BYO-AI design
is right.

Read these first, in this order:

  1. ai-ui/SCOPE.md              — what ai-ui is + hard rules
  2. ai-ui/USECASE.md            — codeless is consumer #2
  3. ai-ui/crates/ai-ui-core/src/provider.rs  — the trait you implement
  4. ai-ui/examples/library-only/src/main.rs  — the pure-Rust shape
  5. codeless/DOCS/SCOPE.md      — codeless's architecture + crate split
  6. codeless/demos/openui-poc/  — the in-tree precursor you're replacing
  7. codeless/ai-runner/         — codeless's existing AI abstraction

Do this:

  a) Add a new crate `codeless-ai-ui` under codeless/crates/ that depends
     on `ai-ui-core` (path = "../../../ai-ui/crates/ai-ui-core" for now;
     path-deps until we publish). It exposes a `CodelessProvider` struct
     that wraps whichever ai-runner runner the job is using and implements
     `ai_ui_core::Provider`. The system prompt arrives in the
     ProviderContext; you forward it to ai-runner correctly for both the
     CLI runners (Claude Code, Codex) and the REST runners.

  b) Mount the ai-ui axum routes inside codeless-server behind a
     namespace — e.g. `/api/ai-ui/chat`, `/api/ai-ui/events`, etc. Don't
     collide with codeless's existing routes. Skills live at
     `codeless/skills/` (create the dir if it doesn't exist); component
     manifest is sourced from
     `ai-ui/packages/ai-ui-react-shadcn/dist/components.json` until
     codeless has its own catalogue.

  c) Decide: delete demos/openui-poc OR convert it into a
     codeless-internal integration test that asserts the new wiring
     produces the same SSE output for a fixed prompt. Recommend the
     latter — it's a regression test for free.

  d) Run codeless's CI locally and confirm nothing else breaks.

Constraints (do not violate):

  - Respect codeless's R1 mobile-safe rule: codeless-types/-rpc/-client
    must not pull `codeless-ai-ui` or anything that brings in ai-runner.
  - Respect ai-ui's R6: codeless-ai-ui depends on `ai-ui-core` only.
    The `ai-ui-axum` crate is mounted by codeless-server, not by the
    provider crate.
  - Default-features on `ai-ui-core` stay empty — codeless brings the
    runner, not a reqwest/claude-cli reference impl.

This is a non-trivial cross-repo change. **Stop and propose a plan**
before writing code — list the files you'd add/modify in each repo,
the dep-graph implications, and any risks. I want to approve the plan
before you start editing.
```

---

## Notes (for me, not the future agent)

- Prompt 1 is the cheap, do-it-anytime one. Run it solo.
- Prompt 2 is bigger. Best done after a clean PR from prompt 1 has merged,
  so you can pin a tagged ai-ui revision in codeless's Cargo.toml.
- Both prompts say "don't push" / "stop and propose a plan first" — keep
  those, they buy you a review checkpoint.
