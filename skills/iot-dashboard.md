---
name: iot-dashboard
description: How to build an IoT dashboard from device telemetry. Use when the user asks for a dashboard of sensors, devices, plant equipment, or live readings.
components: [Page, Grid, KpiTile, LineChart, BarChart, DataTable, Alert, Section]
triggers: [dashboard, telemetry, sensor, chiller, kWh, kpi, plant, building, hvac]
---

When the user asks for an IoT dashboard, follow these conventions:

- **Layout.** Wrap the page in `Page`, then a `Section` per logical group
  (e.g. "Plant overview", "Alarms"). Put KPI tiles inside a `Grid` with 4
  columns on desktop, 2 on mobile.
- **KPI tiles.** Use `KpiTile` for any single-value live reading
  (temperature, kW, occupancy). Bind to a real device id from the
  component manifest's `deviceIds` enum — never invent an id.
- **Charts.** Use `LineChart` for any value over time. Use `BarChart` for
  grouped aggregates ("hourly average", "by device"). Always set the time
  range; if the user didn't specify, default to `24h`.
- **Alarms.** If the user mentions alarms, faults, or anomalies, render an
  `Alert` above the grid with severity = highest among matching devices.
  Show `DataTable` underneath listing the last 10 alarm events.
- **Density.** Prefer 4–8 KPI tiles per row group. More than 12 tiles total
  on one page is a smell — ask the user to filter instead.

Never emit a `Markdown` block on a dashboard; the user wants a dashboard,
not a document.
