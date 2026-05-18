# ai-ui — Scope

## One-line summary

`ai-ui` is a reusable pair of libraries — one Rust, one React — that lets any
product drop in an **AI-generated UI surface**: a host agent emits
[OpenUI Lang](https://github.com/thesysdev/openui), and the frontend streams
structured React components back into the page as the model thinks.

`ai-ui` is a **tool, not a runtime** — it does not bring its own AI. The
consumer's existing AI runner (codeless's `ai-runner`, a self-hosted vLLM,
whatever) plugs in through a one-method `Provider` trait. The Rust side handles
**system-prompt assembly** (from skills + the component catalogue), the
SSE/wire format, and a **backend push channel** so the consumer's backend can
drive the UI without the user typing. The React side wraps OpenUI with an
opinionated component library, with an opt-in **shadcn/ui** variant for
consumers that already use shadcn (such as `codeless`).

## Why this exists

This came out of an
[OpenUI](https://github.com/thesysdev/openui) proof-of-concept built inside
codeless at
[`codeless/demos/openui-poc`](../codeless-workspace/codeless/demos/openui-poc/).
The PoC validated three things:

1. A Rust axum server can proxy any OpenAI-compatible API **or** spawn the
   local `claude` CLI binary and convert its stream-json events into OpenAI
   SSE chunks the OpenUI frontend already understands.
2. `@openuidev/react-ui` renders structured UI live from a streamed LLM
   response — no client-side post-processing, no JSON-schema plumbing per
   component.
3. The system prompt is auto-generated from the component library
   (`npx @openuidev/cli generate src/library.ts`), so the agent's API is the
   library's Zod schemas.

That PoC was scoped to one app. The same machinery is needed by at least two
products (see [`USECASE.md`](./USECASE.md)), so the load-bearing pieces move
into a shared repo before either consumer takes a dependency on the other.

If anything in this file contradicts upstream
[OpenUI](https://github.com/thesysdev/openui) docs, OpenUI wins — we are a
thin opinionated layer over their runtime, not a fork.

## Goals

- **Two installs, one mental model.** A consumer adds one Rust crate and one
  npm package. They get a working `/api/chat` route on the server, a
  drop-in `<AiUi>` component on the client, and a script that regenerates
  the system prompt from their component library.
- **Bring-your-own component library.** The default ships the stock OpenUI
  library so a new consumer is one `cargo run` + `npm run dev` from a working
  demo. A consumer that wants their own components defines a `library.ts`
  with `defineComponent` calls and points the prompt generator at it.
- **shadcn/ui as a first-class option.** A second package
  (`@nube/ai-ui-react-shadcn` or similar) ships a curated wrapper library
  whose `defineComponent` entries render via `shadcn/ui` primitives. A
  consumer that already runs shadcn (codeless does) picks this and their
  AI-generated UI matches the rest of their product.
- **BYO AI runner.** The default story is: the consumer implements a one-method
  `Provider` trait against whatever AI they already run. Two reference
  implementations ship behind feature flags so the examples are self-contained
  — `feature = "openai-proxy"` (any OpenAI-compatible HTTP) and
  `feature = "claude-cli"` (local `claude` binary) — but the **default-features
  list is empty**: a consumer that brings its own runner pulls in no HTTP
  client and no CLI wrapper. This is what makes `ai-ui` cheap for codeless to
  adopt: codeless's `ai-runner` is the provider, and ai-ui adds no second
  copy of model-routing code.
- **Skills are user-extensible.** A skill is a single `.md` file with YAML
  frontmatter (name, description, optional component allowlist). The consumer
  points `ai-ui` at a skills directory; at request time the relevant skills'
  bodies are concatenated into the system prompt. This is how a non-developer
  user adds "make me an IoT dashboard" knowledge without touching code.
- **Component catalogue is part of the prompt.** The system prompt always
  includes the registered component manifest (names + Zod schema descriptions
  from the React library). The model never has to guess what's available; the
  prompt is the contract.
- **Backend-driven, not chat-driven.** The consumer's backend can push OpenUI
  Lang to a connected UI **without a chat round-trip** — e.g., a codeless job
  finishes a stage and pushes a scope-mock card. The UI is a render target,
  not a chatroom. (Chat is still supported; it's just one driver among
  several.)
- **Stay thin.** No state machine, no DB, no auth, no workflow engine. That
  is the consumer's problem. `ai-ui` does only the
  "host emits OpenUI Lang -> structured UI on the page" loop.

## Non-goals

- Not a full agent framework. No tools, no MCP, no multi-step planning.
  Consumers wire those at the LLM-provider layer if they need them.
- Not a UI kit. The shadcn variant is a *wrapper around* shadcn — it doesn't
  ship shadcn itself; the consumer's app does.
- Not a hosted service. Everything runs in the consumer's process.
- No multi-tenant auth. Single trust boundary per deployment; the consumer's
  surrounding app owns auth.

## Repo layout

```
ai-ui/                              <- this repo (NubeDev/ai-ui)
├── README.md
├── SCOPE.md                        <- this file
├── USECASE.md                      <- the two concrete consumers
├── Cargo.toml                      <- Rust workspace root
├── crates/
│   ├── ai-ui-types/                <- shared wire types (ChatRequest/Message,
│   │                                  SkillManifest, ComponentManifest, the
│   │                                  push-channel event enum). No std-only,
│   │                                  no http, safe to depend on from any
│   │                                  crate (including mobile-safe).
│   ├── ai-ui-skills/               <- skill loader: parses .md files with YAML
│   │                                  frontmatter from a directory, surfaces a
│   │                                  registry the prompt assembler consumes.
│   ├── ai-ui-prompt/               <- system-prompt assembler: base + skills
│   │                                  + component manifest -> one string.
│   │                                  Also helpers for loading a pre-built
│   │                                  prompt file (include_str! convention,
│   │                                  env-var override path).
│   ├── ai-ui-core/                 <- the engine: `Provider` trait, `AiUiState`,
│   │                                  builder, push broadcast channel. **No
│   │                                  HTTP, no axum** — usable from Tauri IPC,
│   │                                  a CLI, or any other transport. Feature
│   │                                  flags add the optional reference impls:
│   │                                    default = []        # BYO AI
│   │                                    "openai-proxy" = adds reqwest impl
│   │                                    "claude-cli"   = adds claude-wrapper impl
│   └── ai-ui-axum/                 <- opt-in axum routes for consumers who want
│                                      `POST /api/chat`, `POST /api/push`,
│                                      `GET /api/events`, `GET /api/skills`,
│                                      `GET /api/components`. Depends on
│                                      `ai-ui-core` + `axum` + `tower-http`.
│                                      A consumer that doesn't want HTTP never
│                                      depends on this crate. The example
│                                      binaries use it; the `library-only`
│                                      example deliberately does not.
├── packages/
│   ├── ai-ui-react/                <- npm: thin re-export + glue around
│   │                                  @openuidev/react-ui + @openuidev/react-headless.
│   │                                  Exports <AiUi> (FullScreen wrapper),
│   │                                  the default openuiChatLibrary, and the
│   │                                  openAIAdapter pre-wired to /api/chat.
│   └── ai-ui-react-shadcn/         <- npm: the shadcn variant of the library.
│                                      Each `defineComponent` renders via the
│                                      consumer's shadcn primitives (peerDep:
│                                      shadcn components + tailwind + lucide).
│                                      Re-exports promptOptions so the
│                                      consumer's `generate:prompt` script
│                                      targets this library.
└── examples/
    ├── stock-openui/               <- the demo from openui-poc, ported to
    │                                  consume the published packages — proves
    │                                  the "two installs" claim.
    └── shadcn-dashboard/           <- minimal shadcn-variant example: a single
                                      page where the agent generates dashboard
                                      cards/tables/charts using shadcn-styled
                                      components.
```

The Rust side is a Cargo workspace; the JS side is a pnpm workspace (or npm
workspaces). Cross-language artefacts (the generated system prompt) live in
the consumer's repo, not here — `ai-ui-prompt` only provides loading
conventions.

## Surface — Rust

The Rust side is two crates plus three building blocks. **HTTP is opt-in**:
the engine is `ai-ui-core` and never pulls axum; the axum routes live in
`ai-ui-axum` and a consumer who drives the engine from Tauri IPC, a CLI, or
any other transport never depends on it.

### Pure-library path (no HTTP)

```rust
use ai_ui_core::{AiUiState, Provider, ProviderContext};
use ai_ui_prompt::{PromptBuilder, load_manifest};
use ai_ui_skills::SkillRegistry;

struct MyProvider { /* codeless ai-runner handle, etc. */ }
impl Provider for MyProvider { /* fn stream_chat(ctx, messages) -> ChatStream */ }

let skills   = SkillRegistry::load_dir("skills/")?;
let manifest = load_manifest("ui/src/generated/components.json")?;
let prompt   = PromptBuilder::new()
    .components(manifest)
    .skills(&skills)
    .build();

let provider = MyProvider { /* ... */ };
let stream   = provider.stream_chat(ProviderContext { system_prompt: prompt }, messages);
// pipe `stream` into your own transport: Tauri event, gRPC, websocket, ...
```

`examples/library-only/` is exactly this shape — a binary that loads skills,
assembles the prompt, and streams a turn to stdout. No axum compiled in.

### Library + axum routes path (the usual deployment)

```rust
use ai_ui_axum as router;
use ai_ui_core::{AiUiState, Provider};
use ai_ui_skills::SkillRegistry;

let skills = SkillRegistry::load_dir("skills/")?;
let state = AiUiState::builder()
    .component_manifest_file("ui/src/generated/components.json")
    .skills(skills)
    .provider(MyProvider { /* ... */ })
    .build()?;

let app = axum::Router::new()
    .merge(router::chat_routes(state.clone()))         // POST /api/chat
    .merge(router::push_routes(state.clone()))         // POST /api/push, GET /api/events
    .merge(router::manifest_routes(state))             // GET /api/skills, GET /api/components
    .nest_service("/", tower_http::services::ServeDir::new("ui/dist"));
```

- `Provider` is a one-method trait: `fn stream_chat(&self, ctx, messages)
  -> ChatStream`. **The consumer implements this against their own AI
  runner.** Two reference impls ship behind feature flags on `ai-ui-core`
  (`openai-proxy`, `claude-cli`) so the in-repo examples are runnable; the
  default-features list is empty.
- `AiUiState` carries the assembled system prompt (base + skills + component
  manifest), the provider handle, and a `tokio::sync::broadcast` push
  channel. It is in `ai-ui-core` and has no axum dep — Tauri and CLI
  consumers can hold a single `AiUiState` and drive everything from it.
- `ai_ui_axum::chat_routes` mounts `POST /api/chat` — accepts an
  OpenAI-compatible message list, runs it through the provider, streams
  SSE back.
- `ai_ui_axum::push_routes` mounts `POST /api/push` (consumer's backend
  writes OpenUI Lang directly; no model in the loop) and `GET /api/events`
  (the UI subscribes via EventSource and applies pushes as they arrive).
  This is the **backend-driven** path: a codeless job stage produces a
  card and pushes it without a chat round-trip. The underlying mechanism
  — `AiUiState::broadcast(event)` and `AiUiState::subscribe()` — lives in
  `ai-ui-core`, so the same push channel is usable from a non-HTTP
  consumer (e.g. emit each event over Tauri IPC instead).
- `ai_ui_axum::manifest_routes` mounts `GET /api/skills` and
  `GET /api/components` so a host agent (or a debugger UI) can introspect
  what the model has been taught. Useful when codeless's own UI wants to
  *show* the user "the AI knows about these 14 components and these
  3 skills".
- `ai-ui-skills::SkillRegistry::load_dir` parses every `*.md` in a directory
  (YAML frontmatter + body). Drop a file in, restart the server, the skill is
  live. No code changes required.

No state machine, no DB, no auth middleware. Adding any of those is the
consumer's job.

## Surface — React

`@nube/ai-ui-react` (default — stock OpenUI library):

```tsx
import { AiUi } from "@nube/ai-ui-react";

export default function App() {
  return <AiUi endpoint="/api/chat" agentName="My App" />;
}
```

That's the minimum. `<AiUi>` is `<FullScreen>` with the OpenAI adapter and
the stock OpenUI component library pre-wired; props for conversation
starters, custom library, custom adapter, and styling pass through.

`@nube/ai-ui-react-shadcn` (opt-in — shadcn-backed library):

```tsx
import { AiUi, library, promptOptions } from "@nube/ai-ui-react-shadcn";

// In ui/src/library.ts (consumer copies + extends as they wish):
export { library, promptOptions };
// In their generate:prompt script: targets this library, not the stock one.

export default function App() {
  return <AiUi endpoint="/api/chat" componentLibrary={library} />;
}
```

The shadcn variant ships `defineComponent` entries for the components a
dashboard / scope-page consumer is most likely to need:

- Layout: `Page`, `Section`, `Grid`, `Stack`, `Card`
- Display: `KpiTile`, `Stat`, `Badge`, `Avatar`, `Skeleton`
- Data: `DataTable`, `LineChart`, `BarChart`, `AreaChart`, `PieChart`, `Gauge`
- Forms: `Form`, `Input`, `Select`, `Combobox`, `Checkbox`, `Switch`, `Slider`,
  `DatePicker`, `Button`
- Feedback: `Alert`, `Banner`, `Toast`, `Dialog`, `Tabs`, `Accordion`
- Domain-agnostic: `Markdown`, `Code`, `Empty`

Each entry has a Zod schema whose `.describe()` calls flow into the
auto-generated system prompt — the prompt teaches the model the API exactly
as the consumer authored it.

The shadcn variant declares **peer dependencies**, not direct dependencies,
on `@radix-ui/*`, `lucide-react`, `tailwindcss`, etc. The consumer brings
their own shadcn install (correct version, correct theme); we render via
import paths that match the standard shadcn CLI layout
(`@/components/ui/*`).

## Skills — the user-extensible knowledge format

A skill is a **single `.md` file with YAML frontmatter**. The format
intentionally mirrors Claude Code skills so a user who has already authored
one of those needs zero new mental model.

```markdown
---
name: iot-dashboard
description: How to build an IoT dashboard from device telemetry.
  Use when the user asks for a dashboard of sensors, devices, or live readings.
components: [Page, Grid, KpiTile, LineChart, BarChart, DataTable, Alert]
triggers: [dashboard, telemetry, sensor, chiller, kWh, kpi]
---

When the user asks for an IoT dashboard, follow these conventions:

- Put KPI tiles in a Grid at the top (4 columns desktop, 2 mobile)
- Use LineChart for any value over time; BarChart for grouped aggregates
- Show alarms as an Alert banner above the grid if `alarm_count > 0`
- Bind each tile to a real device id; never hallucinate a device that
  is not in the component manifest's `deviceIds` enum

If the user has not specified a time range, default to 24h.
```

- **`name`** — slug, unique per registry. Used in logs and the `/api/skills`
  response.
- **`description`** — one sentence. The model sees the *description and the
  body*; the description doubles as the trigger hint for skill selection if
  the consumer ever turns on selective loading (default: load all skills).
- **`components`** *(optional)* — allowlist of component ids. If present, the
  prompt assembler narrows the component manifest to this subset for this
  skill's instructions. Useful for keeping the prompt small in a single-purpose
  consumer (an IoT product doesn't need `<Markdown>` rules).
- **`triggers`** *(optional)* — keyword hints for selective loading
  (Stage 7+; not required for MVP).

The body is plain markdown. It is concatenated verbatim into the system
prompt, after the base OpenUI Lang syntax and the component manifest. The
order is stable (alphabetical by `name`) so prompt-cache friendliness is
preserved.

The consumer's skills directory layout is up to them; the loader is
flat-recursive (`**/*.md`) so they can group by folder if they want.

## Hard rules

These are enforceable by `cargo check` or grep. Trip one and the build halts.

### R1 — `ai-ui-types` has no transitive http/runtime deps

It contains the JSON shapes of `ChatRequest` and `ChatMessage` and nothing
else. A mobile-safe consumer (codeless's `codeless-types`, etc.) can depend
on it without pulling axum, reqwest, or tokio.

### R2 — Provider deps are feature-gated, default-features is empty

`claude-wrapper` lives behind `feature = "claude-cli"`. `reqwest` lives
behind `feature = "openai-proxy"`. **Both are off by default.** The default
build of `ai-ui-core` ships the `Provider` trait and the state plumbing and
no model client at all — BYO AI is the headline path. A consumer that wants
the reference impls opts in explicitly:

```toml
ai-ui-core = { version = "0.1", features = ["openai-proxy"] }
```

CI runs `cargo check -p ai-ui-core --no-default-features` and asserts that
neither `reqwest`, `claude-wrapper`, `axum`, nor `tower-http` appears in the
resulting dep graph — `ai-ui-core` is **HTTP-free** by construction.

### R6 — `ai-ui-core` never depends on axum

The whole point of the split between `ai-ui-core` and `ai-ui-axum` is that
the engine is usable without HTTP. A Tauri app, a CLI, or codeless's
existing RPC layer should be able to depend on `ai-ui-core` and drive the
provider through their own transport. Adding an axum/tower/http dep to
`ai-ui-core` breaks this contract.

Grep: `grep -E '^(axum|tower|tower-http|http) =' crates/ai-ui-core/Cargo.toml`
must return zero matches.

### R3 — No state, no DB, no auth in this repo

Anything that needs persistence belongs in the consumer. If a feature
proposal requires us to remember anything across requests, the answer is
"that's a consumer concern" — open an issue, but the answer is no.

### R4 — Shadcn package never `import`s a shadcn primitive directly

Always via `@/components/ui/*` paths the consumer aliases. We never bundle
shadcn source — the consumer's installed version is the truth. A grep of
`packages/ai-ui-react-shadcn/src/` for `@radix-ui/` outside type-only
imports must return zero matches.

### R5 — One opinion per package

`ai-ui-react` is the stock OpenUI library, no shadcn. `ai-ui-react-shadcn` is
the shadcn library, no stock OpenUI. They share zero runtime code; they share
the `<AiUi>` wrapper through `ai-ui-react-core` (a tiny private package, if we
need it). A consumer picks exactly one.

## Stages of work

Sized using the same `S` / `M` / `L` tags codeless uses; one stage per
session.

| # | Stage | Size | Output |
|---|---|---|---|
| 1 | Repo skeleton + workspace tooling | S | `Cargo.toml`, `pnpm-workspace.yaml`, empty crates + packages, CI scaffolding, license, contributing |
| 2 | `ai-ui-types` + `ai-ui-skills` + `ai-ui-prompt` | M | Wire types, skill loader, prompt assembler. Unit-tested in isolation; no axum yet. |
| 3 | `ai-ui-core` (engine) + `ai-ui-axum` (routes) | M | Provider trait + state in core (no HTTP); axum routes in the sister crate; reference `openai-proxy` impl behind feature flag. PoC's `main.rs` becomes a thin example binary. |
| 4 | `ai-ui-react` glue + `examples/stock-openui` runs against the libs | S | `<AiUi>` component, default library re-export, openAI adapter pre-wired |
| 4b | `claude-cli` reference provider feature | S | Second feature-gated impl; demo runs without an API key |
| 4c | First publish — internal pre-release to a private registry | S | Versioning + changelog policy set; consumers can pin |
| 5 | shadcn variant scaffold + 10 core components | M | `Page`, `Card`, `KpiTile`, `DataTable`, `Form`, `Input`, `Select`, `Button`, `LineChart`, `BarChart` |
| 6 | shadcn variant — remainder of the catalogue | M | The rest of the list above |
| 7 | `examples/shadcn-dashboard` runs end-to-end | S | Validation example for the shadcn variant |
| 8 | codeless adopts `ai-ui` as its scope-preview substrate | M | codeless deletes its `demos/openui-poc/` copy and depends on published `ai-ui` crates + packages |
| 9 | IoT dashboard builder adopts `ai-ui` | M | Second consumer validates the shadcn variant in anger; gaps fed back as issues |

Stages 1-4 are the MVP. Stages 5-7 are the shadcn track. Stages 8-9 are the
adoption track; they may slip later in the calendar without blocking 1-7.

## Versioning

- Rust crates version together at the workspace level (`0.x.y` while
  pre-1.0).
- npm packages version together (`0.x.y`) and pin to a matching Rust
  workspace version via a string in their READMEs — the wire format
  (OpenAI SSE chunks of OpenUI Lang) is the contract that ties them.
- Breaking changes to the Rust provider trait, the SSE wire format, or the
  shadcn package's component IDs are minor-version bumps pre-1.0 and
  major-version bumps post-1.0.

## Pointers

- Concrete consumers + what each one needs: [`USECASE.md`](./USECASE.md)
- Upstream OpenUI repo: <https://github.com/thesysdev/openui>
- Source PoC inside codeless: `../codeless-workspace/codeless/demos/openui-poc/`
- shadcn/ui: <https://ui.shadcn.com>
