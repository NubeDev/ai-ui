# ai-ui

A reusable pair of libraries — one Rust, one React — that lets any product
drop in an **AI-generated UI surface**: a host agent emits
[OpenUI Lang](https://github.com/thesysdev/openui), the frontend streams
structured React components back into the page as the model thinks.

**ai-ui is a tool, not a runtime.** It does not bring its own AI. The
consumer's existing AI runner plugs in through a one-method `Provider`
trait. The Rust side handles system-prompt assembly (from user-extensible
**skills** + a **component catalogue**), the SSE/wire format, and a
**backend push channel** so the consumer's backend can drive the UI without
the user typing. The React side wraps OpenUI with an opinionated component
library — opt-in **shadcn/ui** variant for consumers that already use shadcn.

See [`SCOPE.md`](./SCOPE.md) for the full scope and [`USECASE.md`](./USECASE.md)
for the two driving consumers (IoT dashboard builder + codeless scope preview).

## Repo layout

```
ai-ui/
├── crates/                          Rust workspace
│   ├── ai-ui-types/                 wire types, no I/O
│   ├── ai-ui-skills/                .md + YAML-frontmatter skill loader
│   ├── ai-ui-prompt/                system-prompt assembler
│   ├── ai-ui-core/                  Provider trait + state + push channel (no HTTP)
│   └── ai-ui-axum/                  opt-in axum routes (chat/push/manifest)
├── packages/                        npm workspace
│   ├── ai-ui-react/                 stock OpenUI library wrapper
│   └── ai-ui-react-shadcn/          shadcn/ui variant (catalogue + composeLibrary)
├── skills/                          example skills (drop in your own *.md)
└── examples/
    ├── library-only/                pure-Rust use (no axum, no HTTP)
    ├── stock-openui-server/         axum + stock OpenUI library
    └── shadcn-dashboard/            axum + shadcn catalogue
```

## Library vs server

`ai-ui-core` is HTTP-free. A consumer that already has a transport — Tauri
IPC, an existing RPC trait, a job runner — uses `ai-ui-core` directly. The
axum routes in `ai-ui-axum` are an opt-in convenience for consumers who
want to expose `POST /api/chat`, `POST /api/push`, etc. over HTTP.

```rust
// pure-library, no HTTP, no port binding
use ai_ui_core::{Provider, ProviderContext};
use ai_ui_prompt::{PromptBuilder, load_manifest};
use ai_ui_skills::SkillRegistry;

let skills   = SkillRegistry::load_dir("skills/")?;
let manifest = load_manifest("ui/src/generated/components.json")?;
let prompt   = PromptBuilder::new().components(manifest).skills(&skills).build();

let stream = my_provider.stream_chat(
    ProviderContext { system_prompt: prompt },
    messages,
);
// pipe `stream` into whatever transport you already have
```

```rust
// library + axum routes — what the example binaries do
use ai_ui_axum as router;
use ai_ui_core::AiUiState;

let state = AiUiState::builder()
    .skills(skills)
    .component_manifest_file("ui/src/generated/components.json")
    .provider(MyProvider)
    .build()?;

let app = axum::Router::new()
    .merge(router::chat_routes(state.clone()))
    .merge(router::push_routes(state.clone()))
    .merge(router::manifest_routes(state));
```

## Quick start (reference provider — OpenAI-compatible)

```bash
# pure-library shape — prints the assembled prompt and streams one turn
OPENAI_API_KEY=sk-... cargo run -p library-only --features openai-proxy

# axum + UI
cd examples/stock-openui-server
OPENAI_API_KEY=sk-... cargo run --features openai-proxy
```
