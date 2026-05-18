# examples/stock-openui-server

The smallest possible consumer of `ai-ui`: a Rust binary that exposes
`/api/chat`, `/api/push`, `/api/events`, `/api/skills`, `/api/components`,
and serves a Vite-built React UI.

## Pick a component manifest

By default the example reads `ui/src/generated/components.json` — the stock
OpenUI library's manifest. Point at the shadcn catalogue instead by setting
the env var:

```bash
AI_UI_COMPONENT_MANIFEST=../../packages/ai-ui-react-shadcn/dist/components.json \
  cargo run --features openai-proxy
```

`GET /api/components` will now return the 10-component shadcn catalogue and
the system prompt will teach the model that vocabulary.

## Run with a reference provider

```bash
# OpenAI-compatible
OPENAI_API_KEY=sk-... cargo run --features openai-proxy

# Local claude CLI (no API key — uses your existing claude auth)
cargo run --features claude-cli
```

## Run the UI in dev mode (hot reload)

```bash
# terminal 1
OPENAI_API_KEY=sk-... cargo run --features openai-proxy

# terminal 2
cd ui
pnpm install
pnpm dev   # proxies /api to :3001
```

## What it proves

- The Rust crates compile and assemble a real system prompt from the
  workspace's `skills/` directory and `ui/src/generated/components.json`.
- Both reference providers stream into the OpenUI Lang renderer.
- `GET /api/skills` and `GET /api/components` return the registered
  manifest — useful for introspecting "what does the AI know".
- `POST /api/push` broadcasts to anyone subscribed to `GET /api/events`.

## Try a backend push

```bash
curl -X POST http://localhost:3001/api/push \
  -H 'Content-Type: application/json' \
  -d '{"event":{"type":"replace","slot":"banner","openui":"root = TextContent(\"hello from the backend\")"}}'
```
