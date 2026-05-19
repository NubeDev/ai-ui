import { chromium } from "playwright";

const URL = process.env.URL ?? "http://localhost:3001/";

const browser = await chromium.launch({ headless: true });
const ctx = await browser.newContext();
const page = await ctx.newPage();

const logs = [];
page.on("console", (m) => logs.push(`[console:${m.type()}] ${m.text()}`));
await page.addInitScript(() => {
  const _w = console.warn.bind(console);
  console.warn = (...args) => { _w("WARN_FULL:", ...args.map((a) => typeof a === "string" ? a : JSON.stringify(a))); };
});
page.on("pageerror", (e) => logs.push(`[pageerror] ${e.message}`));
page.on("requestfailed", (r) =>
  logs.push(`[reqfail] ${r.url()} ${r.failure()?.errorText}`),
);
page.on("request", (r) => {
  if (r.url().includes("/api/")) logs.push(`[req] ${r.method()} ${r.url()}`);
});

await page.goto(URL, { waitUntil: "domcontentloaded" });
await page.waitForTimeout(800);

// Click the "Data table" starter.
const starter = page.locator('button:has-text("Data table")').first();
await starter.waitFor({ timeout: 5000 });
await starter.click();
logs.push("[probe] clicked Data table");

// Poll the DOM every 1s for up to 60s.
const start = Date.now();
let lastSnap = "";
while (Date.now() - start < 60000) {
  await page.waitForTimeout(1000);
  const text = await page.evaluate(() => document.body.innerText);
  if (text !== lastSnap) {
    logs.push(`[dom @${((Date.now() - start) / 1000).toFixed(0)}s] ${text.length} chars`);
    lastSnap = text;
  }
  if (text.includes("Python")) {
    logs.push("[probe] saw Python — rendered!");
    break;
  }
}

await page.screenshot({ path: "/tmp/demo-probe.png", fullPage: true });

console.log("=== LOGS ===");
for (const l of logs) console.log(l);
console.log("=== FINAL BODY ===");
console.log(lastSnap.slice(0, 3000));

await browser.close();
