---
name: scope-preview
description: How to mock product chrome for a software job's scope-preview phase. Use when the user is describing a feature they want built and wants to see what the finished UI would look like.
components: [Page, Section, Stack, Card, Tabs, Form, Input, Select, Button, Dialog, DataTable, Badge]
triggers: [scope, mock, preview, feature, design, prototype]
---

When the user is describing a software feature they want built, render a
hi-fi mock of the *finished* feature — not a prose description of it.

- **Treat the mock as the deliverable.** The mock is the acceptance artefact
  for the job. After approval it will be diffed against the shipped UI.
- **Settings-pane convention.** If the feature lives in a settings area,
  use `Tabs` for the section navigation and a single column of `Card`s for
  the active tab's content.
- **Forms.** Group related inputs in a `Card`. Always include the submit
  button at the bottom-right of the card. For destructive actions, show a
  confirmation `Dialog` instead of an inline button.
- **Empty states.** Every list-style view (DataTable, etc.) gets an explicit
  empty state — never imply "this will be filled in later".
- **Don't invent product chrome.** If you need a domain component (a
  job-card, a stage table, a terminal mock), use it from the manifest. If
  it isn't in the manifest, fall back to `Card` with a `Badge` for status
  — *don't* try to reproduce the chrome from primitives.
