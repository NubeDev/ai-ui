import { createLibrary, defineComponent } from "@openuidev/react-lang";
import type { ComponentRenderer, Library } from "@openuidev/react-lang";

import { catalogue } from "./components";
import type { CatalogueEntry, InferProps } from "./types";

/**
 * Names of every component shipped in the catalogue. Kept as a literal union
 * so consumers get autocomplete when authoring their renderers map.
 */
export type ComponentName =
  | "Page"
  | "Card"
  | "KpiTile"
  | "DataTable"
  | "LineChart"
  | "BarChart"
  | "Form"
  | "Input"
  | "Select"
  | "Button";

/**
 * Map of `componentName -> renderer`. The consumer brings these — each
 * renderer is normally a thin wrapper around their `@/components/ui/*`
 * shadcn install.
 *
 * Per hard rule R4 (SCOPE.md), this package never imports a shadcn primitive
 * directly; it's the consumer's job to wire renderers. We keep the renderer
 * prop type loose (`unknown`) here; per-component strict typing flows from
 * `catalogue[].propsSchema` and is the consumer's call whether to enforce.
 */
export type Renderers = Record<ComponentName, ComponentRenderer<any>>;

/**
 * Compose an OpenUI `Library` from the catalogue schemas + the consumer's
 * render functions.
 *
 * ```tsx
 * import { Button } from "@/components/ui/button";
 * import { Card }   from "@/components/ui/card";
 *
 * const library = composeLibrary({
 *   Button: ({ props }) => <Button variant={mapVariant(props.variant)}>{props.label}</Button>,
 *   Card:   ({ props, children }) => <Card><CardHeader>{props.title}</CardHeader>{children}</Card>,
 *   // ...
 * });
 * ```
 *
 * The returned `Library` is passed directly to `<AiUi componentLibrary={library} />`
 * or to `Renderer` from `@openuidev/react-lang`.
 */
export function composeLibrary(renderers: Partial<Renderers>): Library {
  const components = catalogue.map((entry) => {
    const renderer = renderers[entry.name as keyof Renderers];
    if (!renderer) {
      throw new Error(
        `composeLibrary: missing renderer for component "${entry.name}". ` +
          `Add it to the renderers map, or omit the component from your catalogue.`,
      );
    }
    return defineComponent({
      name: entry.name,
      description: entry.description,
      props: entry.propsSchema as never,
      component: renderer as never,
    });
  });
  return createLibrary({ components });
}

export { catalogue };
export type { CatalogueEntry, InferProps };
