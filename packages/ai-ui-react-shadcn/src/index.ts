// The shadcn variant.
//
// Hard rule R4 (SCOPE.md): files in this package never import a shadcn
// primitive directly. They go through the consumer's `@/components/ui/*`
// alias so the consumer's installed shadcn version is the source of truth.
// `composeLibrary` is the seam.

export { AiUi } from "@nube/ai-ui-react";
export type { AiUiProps } from "@nube/ai-ui-react";
export { useAiUiPush } from "@nube/ai-ui-react";
export type { PushEvent } from "@nube/ai-ui-react";

export { catalogue, composeLibrary } from "./catalogue/composeLibrary";
export type { CatalogueEntry, InferProps, Renderers } from "./catalogue/composeLibrary";

export { manifest } from "./catalogue/manifest";
export type { ComponentEntry, ComponentManifest } from "./catalogue/manifest";
