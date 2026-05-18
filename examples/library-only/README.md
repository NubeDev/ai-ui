# examples/library-only

Proves the headline claim: **ai-ui does not need a server.** This binary
uses `ai-ui-core` directly — loads skills, loads the component manifest,
assembles the system prompt, and (optionally) streams a single turn through
a reference provider. No axum, no port, no HTTP.

Build deps:

```
ai-ui-core   ✓ trait + state + (optional) reference providers
ai-ui-skills ✓
ai-ui-prompt ✓
ai-ui-types  ✓
                ← no axum, no tower, no http
```

## Run

```bash
# from the repo root — prints the assembled prompt and exits
cargo run -p library-only

# also stream a turn through OpenAI-compatible
OPENAI_API_KEY=sk-... cargo run -p library-only --features openai-proxy

# or through local claude CLI
cargo run -p library-only --features claude-cli
```

## Why this is the long-term answer

The HTTP routes in `ai-ui-axum` are sugar for consumers who happen to be
running axum. A Tauri app, a CLI, a job runner like codeless's runtime —
all of them use `ai-ui-core` the way this example does and pipe the
provider's stream into whatever transport they already have (Tauri IPC,
their existing RPC trait, etc.).

The HTTP path is a deployment detail, not a requirement.
