import type { z } from "zod";

/**
 * Internal-only definition of one component in the shadcn catalogue.
 *
 * - `name` — identifier the model emits in OpenUI Lang.
 * - `description` — one-line description used by the system prompt.
 * - `propsSchema` — Zod schema; the consumer's renderer receives the parsed
 *   prop bag typed from this schema.
 * - `propsDoc` — pre-rendered human-readable prop list spliced into the
 *   system prompt. Hand-authored so the prompt is exactly as we want it.
 * - `example` — optional OpenUI Lang snippet showing canonical usage.
 * - `category` — grouping used by docs / manifest output.
 */
export interface CatalogueEntry<S extends z.ZodTypeAny = z.ZodTypeAny> {
  name: string;
  description: string;
  propsSchema: S;
  propsDoc: string;
  example?: string;
  category:
    | "layout"
    | "display"
    | "data"
    | "forms"
    | "feedback"
    | "domain-agnostic";
}

/**
 * Inferred renderer prop type for one entry — the consumer's render function
 * receives `{ props: InferProps<E>; children: ReactNode }` etc., depending on
 * how `composeLibrary` wires it to OpenUI.
 */
export type InferProps<E extends CatalogueEntry> = z.infer<E["propsSchema"]>;
