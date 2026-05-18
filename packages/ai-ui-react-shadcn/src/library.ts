/**
 * shadcn-backed component library. Stage 5 of SCOPE.md.
 *
 * Each entry in `library` is a `defineComponent` from `@openuidev/react-lang`
 * with:
 *  - a Zod schema whose `.describe()` calls flow into the auto-generated
 *    system prompt
 *  - a render function that delegates to `@/components/ui/*` (the consumer's
 *    shadcn install)
 *
 * The skeleton ships with `Page` and `Card` so consumers can see the wiring;
 * the remaining catalogue (KpiTile, DataTable, charts, forms, etc.) lands in
 * subsequent stages.
 */

// Placeholder — the real library is built up in Stage 5. Exporting an empty
// object keeps the package importable today.
export const library = {} as const;
export const promptOptions = {} as const;
