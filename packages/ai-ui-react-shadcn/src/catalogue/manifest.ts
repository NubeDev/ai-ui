import { catalogue } from "./components";

/**
 * Wire-format type matching the Rust-side `ComponentManifest` in `ai-ui-types`.
 *
 * Generated and consumed verbatim by the Rust prompt assembler. Hand-edit at
 * your own risk — re-run `pnpm run generate:components` to regenerate.
 */
export interface ComponentEntry {
  name: string;
  description: string;
  props: string;
  example?: string;
}

export interface ComponentManifest {
  name: string;
  preamble: string;
  components: ComponentEntry[];
}

/**
 * The shadcn-variant manifest. The shape matches the Rust crate's
 * `ai_ui_types::ComponentManifest` so the JSON file we emit can be loaded
 * directly by `ai_ui_prompt::load_manifest`.
 */
export const manifest: ComponentManifest = {
  name: "ai-ui-react-shadcn",
  preamble:
    "These are shadcn/ui-backed components. Every component below renders via " +
    "the consumer's local @/components/ui/* primitives — never invent a " +
    "component name, never invent a prop name. Always provide an `empty` " +
    "string on DataTable; never leave a list without an explicit empty state.",
  components: catalogue.map((c) => ({
    name: c.name,
    description: c.description,
    props: c.propsDoc,
    ...(c.example ? { example: c.example } : {}),
  })),
};
