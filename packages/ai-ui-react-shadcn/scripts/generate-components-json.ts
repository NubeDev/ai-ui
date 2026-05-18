#!/usr/bin/env node
/**
 * Emit `dist/components.json` from the catalogue.
 *
 * Run with: `pnpm run generate:components` (defined in package.json).
 *
 * The output file matches the wire shape `ai_ui_types::ComponentManifest`
 * exactly, so a Rust consumer of `ai-ui-server` can point its
 * `component_manifest_file(...)` at it and the model will see the catalogue
 * verbatim.
 */
import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { manifest } from "../src/catalogue/manifest";

const here = dirname(fileURLToPath(import.meta.url));
const out = resolve(here, "..", "dist", "components.json");
mkdirSync(dirname(out), { recursive: true });
writeFileSync(out, JSON.stringify(manifest, null, 2) + "\n", "utf8");
// eslint-disable-next-line no-console
console.log(`wrote ${out} (${manifest.components.length} components)`);
