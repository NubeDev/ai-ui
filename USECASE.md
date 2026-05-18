# ai-ui — Use cases

Two concrete consumers drive the design in [`SCOPE.md`](./SCOPE.md). Both ride
the same Rust crate and the same shadcn-variant React package; the only
difference between them is the **component catalogue** the consumer's
`library.ts` exposes to the model, and the **system prompt preamble** that
sets the agent's role.

## Use case 1 — IoT dashboard builder

### Who the user is

A non-developer end user of an IoT product (think: a building manager, a
plant supervisor, a homeowner with a dozen sensors). They have devices
emitting telemetry — temperatures, valve states, kWh, occupancy, alarms.
Today they would either pay an integrator to draw screens for them, or they
would learn a low-code dashboard tool (Grafana, ThingsBoard, Node-RED
dashboards) — both options gate on tools the user does not want to learn.

### What they want to do

Type one sentence and get a dashboard page. Then refine it by talking.

> "Make me a dashboard for the chiller plant — show the 4 chillers' kW, a
> 24-hour line chart of total load, and a table of any alarms in the last
> 8 hours."

The page assembles live in front of them: KPI tiles, a chart, a table.
They say "the chart should be bar, not line, and group by hour"; the page
re-renders. They pin it to their navigation and move on. Saved pages are
plain stored OpenUI Lang plus a snapshot of the data bindings the agent
chose.

### What the agent needs to be allowed to generate

The shadcn variant's catalogue (see [`SCOPE.md`](./SCOPE.md) -> "Surface --
React") covers ~80% of this consumer's needs out of the box. The IoT
consumer extends `library.ts` with **two domain-aware components**:

- `<DeviceTile device="chiller-01" metric="kW" />` — looks up the binding,
  renders a `KpiTile` with live telemetry.
- `<TimeseriesChart point="..." range="24h" agg="mean" />` — same idea,
  rendered via the catalogue's `LineChart` / `BarChart`.

Both components have Zod schemas constrained to real device IDs and point
names. Constraint values come from the IoT product's device registry at
prompt-generation time — the system prompt the model receives lists only
devices the *current user* is allowed to see, so the agent cannot
hallucinate a chiller that doesn't exist or one belonging to a different
site.

### What makes `ai-ui` the right substrate for this

- **shadcn variant.** The IoT product's existing console is already shadcn;
  AI-generated pages match the rest of the UI without any styling work.
- **No multi-tenant in the lib.** Auth, device scoping, and per-user
  device-list narrowing are the IoT product's job; `ai-ui` just renders
  what the agent emits.
- **Live streaming render.** A dashboard with 12 tiles + 3 charts feels
  built-as-you-watch, not loaded-after-spinner. Matches the product's
  perceived-snappiness bar.
- **Provider flexibility.** The IoT product self-hosts; it can point the
  Rust crate at a self-hosted llama.cpp / vLLM endpoint via the
  OpenAI-compatible mode and never touch a third-party API.

### What the IoT product owns, separate from `ai-ui`

- The page persistence model (which user owns which pages, which device
  bindings are pinned).
- The device-registry call that produces the per-user constraint values
  embedded in the system prompt.
- Auth, multi-tenancy, RBAC.
- The non-AI-generated parts of the console (settings, device admin,
  user management).

## Use case 2 — Codeless scope preview

### Who the user is

The codeless single-user developer (today: dogfooding self; later: anyone
running a codeless instance on their own box). They are about to kick off
an AI coding job and want to **see what they're getting** before the loop
chews through hours of model time on a misread brief.

### What they want to do

In the codeless UI, when they enter a job's `scope` stage, the chat panel
opens to the codeless agent and the right-hand preview tab opens an empty
scope page. As the user describes the job, the agent emits OpenUI Lang
into the preview tab: a hi-fi mock of the *finished* feature, not a prose
description of it.

> "I want a Secrets tab in the workspace settings that lets the user store
> per-provider API keys, encrypted at rest, listed by alias, with copy and
> delete buttons."

The scope page materializes live: settings layout, the new tab, a list of
fake aliases, the copy/delete buttons in the right place, an "Add secret"
dialog. The user clicks "no, make this a modal, not an inline form";
the agent rewrites the OpenUI Lang; the preview re-renders. When the user
clicks "approve scope", the OpenUI Lang is frozen as the **acceptance
artefact** for the job. At job completion, the shipped feature is
diffed against this artefact in a `REVIEW` stage.

This replaces the codeless scope phase's current output (pages of prose in
`SCOPE.md`-style docs) with a clickable artefact the user can review in
seconds.

### What the agent needs to be allowed to generate

The shadcn variant's catalogue covers ~95% of codeless's needs because
codeless's own UI is already shadcn — every primitive the agent might want
to mock is already in the catalogue. Codeless extends `library.ts` with
**codeless-specific composite components** so the agent can emit
recognisable product chrome:

- `<JobCard status="..." stage={...} />` — renders codeless's actual
  job-card visual.
- `<StageOverview stages={[...]} />` — the stage table the user already
  knows.
- `<TerminalMock lines={[...]} />` — fake terminal output for the bottom
  panel.
- `<CodeDiffMock files={[...]} />` — fake diff view.

These map to the components codeless already ships in its UI tree, so
"scope mock" and "shipped feature" use the *same components* — the only
difference is whether the data is real or scripted.

### What makes `ai-ui` the right substrate for this

- **Same component substrate as codeless's real UI.** The scope mock looks
  pixel-identical to the future shipped feature because both render through
  the same shadcn primitives. There is no "design vs. implementation" gap.
- **Streaming-first.** Matches codeless's existing SSE / RpcClient
  architecture natively; the scope page streams into the preview tab the
  same way a job's task events stream into the job page.
- **No new shell coupling.** `<AiUi>` is a React component; codeless's
  "one UI, four shells" rule (R3 in the workspace `CLAUDE.md`) is
  preserved — works identically in browser, Tauri desktop, and mobile
  webview.
- **Local provider.** Codeless already discovers the `claude` CLI on the
  host; `ai-ui-server`'s `claude-cli` feature uses the same binary, so the
  user pays no extra setup cost and brings no new API key.
- **The acceptance artefact is plain text.** OpenUI Lang stored in SQLite
  alongside the job; auditable, diffable, re-renderable. It does not need
  its own asset pipeline.

### What codeless owns, separate from `ai-ui`

- Everything described in the codeless `CLAUDE.md` and `SCOPE.md` — the
  job runtime, the worktree manager, the runner abstraction, SQLite as the
  source of truth, the four-shell client story.
- The codeless-specific composite components in `library.ts`.
- The "scope approval" review gate state machine (a codeless `Review`,
  the same as any other stage review).
- The diff between "approved scope OpenUI Lang" and "shipped feature
  rendered through the same components" at job-completion time.

## What the two consumers prove together

| Property | IoT dashboard builder | Codeless scope preview |
|---|---|---|
| End user | Non-developer | Developer |
| Surface | Dashboards (display-heavy) | Product chrome mocks (interaction-heavy) |
| Persistence model | Pages pinned per user | Scope frozen per job |
| Provider | Self-hosted OpenAI-compatible | Local `claude` CLI |
| shadcn? | Yes | Yes |
| Custom components beyond catalogue | 2 (DeviceTile, TimeseriesChart) | ~4 (JobCard, StageOverview, ...) |
| Constraint values in prompt | Per-user device registry | Existing repo / job context |

If `ai-ui` works cleanly for both, the substrate is general enough that a
third consumer (e.g. an internal-tools page builder, a CRM-record page
generator) is a matter of writing their `library.ts`, not modifying the
library itself. That is the bar for declaring `ai-ui` 1.0.
