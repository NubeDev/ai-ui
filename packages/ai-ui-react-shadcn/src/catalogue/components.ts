/**
 * The 10 core shadcn-variant components (SCOPE.md Stage 5).
 *
 * Each entry has:
 * - A Zod schema (`propsSchema`) the consumer's renderer can use for typing.
 * - A hand-authored `propsDoc` block that gets spliced into the system prompt.
 *   We hand-author because the prompt is the AI contract; rendering it from
 *   the Zod schema would lose the editorial tone.
 *
 * Rendering is NOT in this package (hard rule R4). The consumer pairs each
 * entry's `name` with a render function imported from their own
 * `@/components/ui/*` install, via `composeLibrary({ Button: MyButton, ... })`.
 */

import { z } from "zod";

import type { CatalogueEntry } from "./types";

// ---------------------------------------------------------------------------
// Layout: Page, Card
// ---------------------------------------------------------------------------

export const Page: CatalogueEntry = {
  name: "Page",
  category: "layout",
  description:
    "Top-level page wrapper. Use exactly once at `root`. Wraps a single column of children with consistent gutters and a max-width.",
  propsSchema: z.object({
    title: z.string().optional().describe("Page title rendered in the header."),
    subtitle: z.string().optional(),
    children: z.array(z.any()).describe("Stack of section-level children."),
  }),
  propsDoc: [
    "title?: string         // page title in the header",
    "subtitle?: string",
    "children: [...]        // section-level children, rendered in a column",
  ].join("\n"),
  example: [
    'root = Page(title="Chiller plant", children=[overview, alarms])',
    'overview = Section(title="Overview", children=[grid])',
    'alarms = Section(title="Alarms", children=[alarmsTable])',
  ].join("\n"),
};

export const Card: CatalogueEntry = {
  name: "Card",
  category: "layout",
  description:
    "Bordered container for grouped content. Use for any logical group — KPI cluster, form, table.",
  propsSchema: z.object({
    title: z.string().optional(),
    description: z.string().optional(),
    children: z.array(z.any()),
    footer: z.any().optional().describe("Optional footer (usually a Button)."),
  }),
  propsDoc: [
    "title?: string",
    "description?: string",
    "children: [...]",
    "footer?: identifier   // typically a Button reference",
  ].join("\n"),
  example: [
    'root = Card(title="Secrets", description="API keys", children=[list], footer=addBtn)',
    'list = DataTable(...)',
    'addBtn = Button(label="Add secret", variant="primary")',
  ].join("\n"),
};

// ---------------------------------------------------------------------------
// Display: KpiTile
// ---------------------------------------------------------------------------

export const KpiTile: CatalogueEntry = {
  name: "KpiTile",
  category: "display",
  description:
    "Single-value display: big number on top, label below, optional delta. The right tool for any live telemetry reading or rolled-up KPI.",
  propsSchema: z.object({
    label: z.string().describe("What this number represents (e.g. 'Revenue')."),
    value: z.union([z.string(), z.number()]),
    unit: z.string().optional(),
    delta: z
      .object({
        value: z.union([z.string(), z.number()]),
        direction: z.enum(["up", "down", "flat"]),
      })
      .optional()
      .describe("Optional change vs. previous period."),
  }),
  propsDoc: [
    "label: string",
    "value: string | number",
    "unit?: string",
    "delta?: { value: string | number, direction: \"up\" | \"down\" | \"flat\" }",
  ].join("\n"),
  example:
    'root = KpiTile(label="Total kW", value=842, unit="kW", delta={value:"+3%", direction:"up"})',
};

// ---------------------------------------------------------------------------
// Data: DataTable, LineChart, BarChart
// ---------------------------------------------------------------------------

export const DataTable: CatalogueEntry = {
  name: "DataTable",
  category: "data",
  description:
    "Sortable table of rows. Columns are declared upfront; rows are objects keyed by column id.",
  propsSchema: z.object({
    columns: z
      .array(
        z.object({
          id: z.string(),
          header: z.string(),
          align: z.enum(["left", "right", "center"]).optional(),
        }),
      )
      .describe("Column definitions, in render order."),
    rows: z
      .array(z.record(z.union([z.string(), z.number(), z.boolean()])))
      .describe("Row objects; each key matches a column id."),
    empty: z
      .string()
      .optional()
      .describe("Empty-state text. Always set this; never leave a blank table."),
  }),
  propsDoc: [
    "columns: [{ id: string, header: string, align?: \"left\"|\"right\"|\"center\" }]",
    "rows:    [ { <columnId>: string | number | boolean } ]",
    "empty?:  string   // empty-state text — ALWAYS provide one",
  ].join("\n"),
  example: [
    "root = DataTable(",
    '  columns=[{id:"name", header:"Name"}, {id:"site", header:"Site"}, {id:"kW", header:"kW", align:"right"}],',
    '  rows=[{name:"CH-01", site:"A", kW:212}, {name:"CH-02", site:"A", kW:198}],',
    '  empty="No chillers online."',
    ")",
  ].join("\n"),
};

export const LineChart: CatalogueEntry = {
  name: "LineChart",
  category: "data",
  description:
    "Time-series line chart. Default tool for any 'value over time'.",
  propsSchema: z.object({
    series: z
      .array(
        z.object({
          name: z.string(),
          data: z.array(
            z.object({ x: z.union([z.string(), z.number()]), y: z.number() }),
          ),
        }),
      )
      .describe("One or more named series."),
    yLabel: z.string().optional(),
    xLabel: z.string().optional(),
    range: z
      .string()
      .optional()
      .describe("Human-readable range tag, e.g. '24h'. Display-only."),
  }),
  propsDoc: [
    "series: [{ name: string, data: [{x: string|number, y: number}] }]",
    "yLabel?: string",
    "xLabel?: string",
    "range?:  string   // e.g. \"24h\"; display only",
  ].join("\n"),
  example:
    'root = LineChart(series=[{name:"Total kW", data:[{x:"00:00", y:780}, {x:"06:00", y:920}, {x:"12:00", y:1180}]}], range="24h", yLabel="kW")',
};

export const BarChart: CatalogueEntry = {
  name: "BarChart",
  category: "data",
  description:
    "Bar chart for grouped or aggregated values. Use when the user wants a comparison, not a trend.",
  propsSchema: z.object({
    series: z.array(
      z.object({
        name: z.string(),
        data: z.array(
          z.object({ x: z.union([z.string(), z.number()]), y: z.number() }),
        ),
      }),
    ),
    yLabel: z.string().optional(),
    xLabel: z.string().optional(),
    stacked: z.boolean().optional(),
  }),
  propsDoc: [
    "series: [{ name: string, data: [{x: string|number, y: number}] }]",
    "yLabel?: string",
    "xLabel?: string",
    "stacked?: boolean",
  ].join("\n"),
  example:
    'root = BarChart(series=[{name:"kWh by chiller", data:[{x:"CH-01", y:4200}, {x:"CH-02", y:3800}]}], yLabel="kWh")',
};

// ---------------------------------------------------------------------------
// Forms: Form, Input, Select, Button
// ---------------------------------------------------------------------------

export const Form: CatalogueEntry = {
  name: "Form",
  category: "forms",
  description:
    "Group of form fields with a single submit. Wrap any cluster of Inputs/Selects/Checkboxes in a Form.",
  propsSchema: z.object({
    name: z.string().describe("Form id, used to namespace field values."),
    children: z.array(z.any()),
    submitLabel: z.string().optional(),
    submitAction: z
      .string()
      .optional()
      .describe(
        "Action name the consumer's onAction handler will receive on submit.",
      ),
  }),
  propsDoc: [
    "name: string                 // form id; namespaces field values",
    "children: [Input | Select | Checkbox | Button ...]",
    "submitLabel?: string         // defaults to \"Submit\"",
    "submitAction?: string        // action id passed to onAction on submit",
  ].join("\n"),
  example: [
    'root = Form(name="contact", submitLabel="Send", submitAction="contact.submit", children=[name, email, msg])',
    'name = Input(name="name", label="Name")',
    'email = Input(name="email", label="Email", type="email")',
    'msg = Input(name="message", label="Message", multiline=true)',
  ].join("\n"),
};

export const Input: CatalogueEntry = {
  name: "Input",
  category: "forms",
  description: "Single text input field. Use multiline=true for a textarea.",
  propsSchema: z.object({
    name: z.string().describe("Field id; must be unique within the Form."),
    label: z.string().optional(),
    type: z
      .enum(["text", "email", "password", "url", "number"])
      .optional()
      .describe("HTML input type. Default 'text'."),
    placeholder: z.string().optional(),
    required: z.boolean().optional(),
    multiline: z.boolean().optional(),
    defaultValue: z.union([z.string(), z.number()]).optional(),
  }),
  propsDoc: [
    "name: string                                    // unique within the Form",
    "label?: string",
    "type?: \"text\" | \"email\" | \"password\" | \"url\" | \"number\"",
    "placeholder?: string",
    "required?: boolean",
    "multiline?: boolean                              // renders a textarea",
    "defaultValue?: string | number",
  ].join("\n"),
  example: 'root = Input(name="alias", label="Alias", required=true)',
};

export const Select: CatalogueEntry = {
  name: "Select",
  category: "forms",
  description: "Dropdown select. Options are { value, label } pairs.",
  propsSchema: z.object({
    name: z.string(),
    label: z.string().optional(),
    options: z.array(
      z.object({ value: z.string(), label: z.string() }),
    ),
    defaultValue: z.string().optional(),
    placeholder: z.string().optional(),
  }),
  propsDoc: [
    "name: string",
    "label?: string",
    "options: [{ value: string, label: string }]",
    "defaultValue?: string",
    "placeholder?: string",
  ].join("\n"),
  example:
    'root = Select(name="severity", label="Severity", options=[{value:"info",label:"Info"},{value:"warn",label:"Warning"},{value:"crit",label:"Critical"}])',
};

export const Button: CatalogueEntry = {
  name: "Button",
  category: "forms",
  description:
    "Action button. Pair with `action` to trigger a consumer-defined action when clicked.",
  propsSchema: z.object({
    label: z.string(),
    variant: z
      .enum(["primary", "secondary", "destructive", "ghost", "link"])
      .optional(),
    action: z
      .string()
      .optional()
      .describe(
        "Action id passed to onAction on click. Omit for inert buttons inside a Form (the Form's submit handles it).",
      ),
    disabled: z.boolean().optional(),
  }),
  propsDoc: [
    "label: string",
    "variant?: \"primary\" | \"secondary\" | \"destructive\" | \"ghost\" | \"link\"",
    "action?: string         // action id passed to onAction; omit for in-Form buttons",
    "disabled?: boolean",
  ].join("\n"),
  example: 'root = Button(label="Delete", variant="destructive", action="secret.delete")',
};

// ---------------------------------------------------------------------------
// Manifest export
// ---------------------------------------------------------------------------

export const catalogue: CatalogueEntry[] = [
  Page,
  Card,
  KpiTile,
  DataTable,
  LineChart,
  BarChart,
  Form,
  Input,
  Select,
  Button,
];
