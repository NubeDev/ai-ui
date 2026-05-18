import { useEffect, useState } from "react";

/**
 * One push event sent by the Rust server over `GET /api/events`.
 *
 * Matches the `PushEvent` enum in `ai-ui-types`.
 */
export type PushEvent =
  | { type: "replace"; slot: string; openui: string }
  | { type: "append"; slot: string; openui: string }
  | { type: "clear"; slot: string }
  | { type: "ping" };

/**
 * Subscribe to the ai-ui push channel.
 *
 * Returns a map of slot id → current OpenUI Lang string. The consumer renders
 * each slot's value through whichever OpenUI renderer they're using.
 *
 * @param endpoint EventSource URL. Default `/api/events`.
 */
export function useAiUiPush(endpoint = "/api/events"): Record<string, string> {
  const [slots, setSlots] = useState<Record<string, string>>({});

  useEffect(() => {
    const es = new EventSource(endpoint);
    es.addEventListener("push", (ev: MessageEvent) => {
      let parsed: PushEvent | null = null;
      try {
        parsed = JSON.parse(ev.data) as PushEvent;
      } catch {
        return;
      }
      if (!parsed) return;
      setSlots((prev) => {
        switch (parsed!.type) {
          case "replace":
            return { ...prev, [parsed!.slot]: parsed!.openui };
          case "append":
            return {
              ...prev,
              [parsed!.slot]: (prev[parsed!.slot] ?? "") + parsed!.openui,
            };
          case "clear": {
            const next = { ...prev };
            delete next[parsed!.slot];
            return next;
          }
          case "ping":
          default:
            return prev;
        }
      });
    });
    return () => es.close();
  }, [endpoint]);

  return slots;
}
