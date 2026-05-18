# examples/shadcn-dashboard

Second example — same Rust wiring as `stock-openui-server`, but the system
prompt teaches the model the **shadcn catalogue** (`packages/ai-ui-react-shadcn`)
instead of the stock OpenUI library.

## Run

```bash
# from this directory
OPENAI_API_KEY=sk-... cargo run --features openai-proxy
# server listens on :3002
```

`GET /api/components` returns the 10-component shadcn catalogue
([dist/components.json](../../packages/ai-ui-react-shadcn/dist/components.json)).

## React side (consumer's responsibility)

The shadcn package can't import `@/components/ui/*` directly (hard rule R4 in
`SCOPE.md`) — the consumer's shadcn install is the source of truth. To wire
renderers, the consumer's app does roughly:

```tsx
import { AiUi, composeLibrary } from "@nube/ai-ui-react-shadcn";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
// ...etc, one import per catalogue entry

const library = composeLibrary({
  Button: ({ props }) => (
    <Button variant={mapVariant(props.variant)}>{props.label}</Button>
  ),
  Card: ({ props, children }) => (
    <Card>
      {props.title ? (
        <CardHeader>
          <CardTitle>{props.title}</CardTitle>
          {props.description ? (
            <CardDescription>{props.description}</CardDescription>
          ) : null}
        </CardHeader>
      ) : null}
      <CardContent>{children}</CardContent>
    </Card>
  ),
  // ...one renderer per catalogue entry; see ../../packages/ai-ui-react-shadcn/src/catalogue/components.ts
});

export default function App() {
  return <AiUi componentLibrary={library} agentName="dashboard" />;
}
```

The UI's `ui/` subdirectory in this example is intentionally a skeleton —
fully shipping a shadcn install (Tailwind v4, init scripts, theme tokens) is
the *consumer's* call and out of scope for this repo. Codeless (the first
real consumer) will adopt this wiring in its existing React app and inherit
the shadcn theme it already ships.

## Backend push (no chat round-trip)

```bash
curl -X POST http://localhost:3002/api/push \
  -H 'Content-Type: application/json' \
  -d '{
    "event": {
      "type": "replace",
      "slot": "main",
      "openui": "root = Card(title=\"Hello\", children=[t])\nt = TextContent(\"pushed from the backend\")"
    }
  }'
```

Any frontend subscribed to `GET /api/events` via `useAiUiPush` from
`@nube/ai-ui-react` will receive this event and re-render the `main` slot.
