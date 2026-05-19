import { chromium } from "playwright";

const browser = await chromium.launch({ headless: true });
const ctx = await browser.newContext();
const page = await ctx.newPage();

const logs = [];
page.on("console", (m) => logs.push(`[console:${m.type()}] ${m.text()}`));
page.on("pageerror", (e) => logs.push(`[pageerror] ${e.message}`));

// Capture the SSE body so we see exactly what the model emitted.
let sseBody = "";
page.on("response", async (res) => {
  if (res.url().endsWith("/api/chat")) {
    try { sseBody = await res.text(); } catch (e) { sseBody = `(read failed: ${e.message})`; }
  }
});

await page.goto("http://localhost:3001/", { waitUntil: "domcontentloaded" });
await page.waitForTimeout(800);

await page.locator('button:has-text("IoT dashboard")').first().click();
logs.push("[probe] clicked IoT dashboard");

const start = Date.now();
let lastSnap = "";
let lastLen = 0;
while (Date.now() - start < 90000) {
  await page.waitForTimeout(1000);
  const t = await page.evaluate(() => document.body.innerText);
  if (t.length !== lastLen) { logs.push(`[dom @${((Date.now()-start)/1000).toFixed(0)}s] ${t.length}c`); lastLen = t.length; lastSnap = t; }
  if (sseBody && (Date.now() - start) > 5000 && t.length === lastLen) {
    // Stream completed and DOM stable — break.
    await page.waitForTimeout(1500);
    const t2 = await page.evaluate(() => document.body.innerText);
    if (t2.length === lastLen) { lastSnap = t2; break; }
  }
}

await page.screenshot({ path: "/tmp/iot-probe.png", fullPage: true });

console.log("=== LOGS ===");
for (const l of logs) console.log(l);
console.log("=== SSE BODY ===");
console.log(sseBody);
console.log("=== FINAL DOM ===");
console.log(lastSnap.slice(0, 4000));
await browser.close();
