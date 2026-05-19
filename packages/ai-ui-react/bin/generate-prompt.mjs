#!/usr/bin/env node
// Render OpenUI's own canonical system prompt for a chosen library and
// write it to disk. This is the SINGLE source of truth for component
// syntax — the Rust side splices the resulting text verbatim into the
// system prompt instead of reinventing it.
//
// Usage:
//   generate-ai-ui-prompt --library openui --out path/to/components-prompt.txt
//
// Flags:
//   --library    "openui" (default) | "openui-chat". The chat library has
//                Card as root; the full library has Stack as root and is
//                what a free-form UI generator wants.
//   --out        Output file path. Default: ./generated/components-prompt.txt
//   --json-out   Optional: write a JSON sidecar with { name, prompt, root,
//                components: [name,...] } for introspection via /api/components.

import { writeFileSync, mkdirSync } from "node:fs";
import { dirname, resolve } from "node:path";
import {
  openuiLibrary,
  openuiChatLibrary,
  openuiPromptOptions,
  openuiChatPromptOptions,
} from "@openuidev/react-ui/genui-lib";

function arg(flag, def) {
  const i = process.argv.indexOf(flag);
  if (i < 0) return def;
  const next = process.argv[i + 1];
  if (next === undefined || next.startsWith("--")) {
    console.error(`error: ${flag} requires a value`);
    process.exit(1);
  }
  return next;
}

function propNames(def) {
  // The library's Zod schema for props lives on `def.props`. ZodObject
  // exposes its shape under `.shape`; nothing else carries reliable
  // structure. Return a comma-separated string of prop names, or empty
  // when the component takes no props.
  const shape = def?.props?.shape;
  return shape ? Object.keys(shape).join(", ") : "";
}

const libraryName = arg("--library", "openui");
const outArg = arg("--out", "./generated/components-prompt.txt");
const out = resolve(outArg);
const jsonOutArg = arg("--json-out", null);
const jsonOut = jsonOutArg ? resolve(jsonOutArg) : null;

const pick = {
  openui: { lib: openuiLibrary, opts: openuiPromptOptions },
  "openui-chat": { lib: openuiChatLibrary, opts: openuiChatPromptOptions },
}[libraryName];

if (!pick) {
  console.error(`unknown --library ${libraryName}; expected openui|openui-chat`);
  process.exit(1);
}

let promptText;
try {
  promptText = pick.lib.prompt(pick.opts);
} catch (err) {
  console.error(`error: lib.prompt() failed: ${err.message}`);
  process.exit(1);
}

try {
  mkdirSync(dirname(out), { recursive: true });
  writeFileSync(out, promptText, "utf8");
} catch (err) {
  console.error(`error: writing ${out}: ${err.message}`);
  process.exit(1);
}
console.log(`wrote ${promptText.length} chars to ${out}`);

if (jsonOut) {
  // ComponentManifest-shaped sidecar for `GET /api/components` introspection.
  // No `example` fields — examples that teach incorrect syntax are exactly
  // how this codebase got into trouble; the canonical prompt above is the
  // only place that teaches the model how to call components.
  const manifest = {
    name: libraryName,
    preamble: `Library: ${libraryName}. Root component: ${pick.lib.root}. The full, parser-correct component signatures and rules are spliced separately into the system prompt by ai-ui-prompt.`,
    components: Object.entries(pick.lib.components).map(([name, def]) => ({
      name,
      description: def.description ?? "",
      props: propNames(def),
    })),
  };
  try {
    mkdirSync(dirname(jsonOut), { recursive: true });
    writeFileSync(jsonOut, JSON.stringify(manifest, null, 2), "utf8");
  } catch (err) {
    console.error(`error: writing ${jsonOut}: ${err.message}`);
    process.exit(1);
  }
  console.log(`wrote component manifest sidecar to ${jsonOut}`);
}
