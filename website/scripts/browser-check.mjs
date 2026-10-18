// Run with the design runtime's installed Playwright and authenticated Browserless.
import assert from "node:assert/strict";
import fs from "node:fs/promises";
import path from "node:path";
const { chromium } = await import(
  process.env.MIRKET_PLAYWRIGHT_MODULE || "playwright-core"
);
const [url, output] = process.argv.slice(2);
if (!url || !output || !process.env.BROWSERLESS_WS)
  throw Error("Provide URL, evidence directory and BROWSERLESS_WS");
await fs.mkdir(output, { recursive: true });
const browser = await chromium.connect(process.env.BROWSERLESS_WS);
const report = [];
try {
  for (const width of [1440, 768, 720, 390, 320]) {
    const context = await browser.newContext({
      viewport: { width, height: 900 },
      deviceScaleFactor: 2,
      reducedMotion: "reduce",
    });
    try {
      const page = await context.newPage();
      const errors = [];
      const failed = [];
      page.on("pageerror", (e) => errors.push(e.message));
      page.on("response", (r) => {
        if (r.status() >= 400) failed.push(r.url());
      });
      await page.goto(url, { waitUntil: "networkidle" });
      await page.screenshot({ path: path.join(output, `${width}-entry.png`) });
      assert.equal(await page.locator("h1").count(), 1);
      assert.ok(
        await page.evaluate(
          () => document.documentElement.scrollWidth <= innerWidth,
        ),
        `Overflow at ${width}`,
      );
      await page.keyboard.press("Tab");
      assert.equal(
        await page.locator(":focus").textContent(),
        "Skip to content",
      );
      for (const [route, tier] of [
        ["compare", "T1"],
        ["code", "T2"],
        ["design", "T3"],
      ]) {
        await page.locator(`[data-route=${route}]`).click();
        assert.equal(await page.locator("#route-tier").textContent(), tier);
        assert.equal(
          await page.locator("[data-route][aria-pressed=true]").count(),
          1,
        );
      }
      for (const [platform, asset] of [
        ["mac", "aarch64-apple-darwin"],
        ["linux", "x86_64-unknown-linux-gnu"],
        ["windows", "x86_64-pc-windows-msvc.exe"],
      ]) {
        await page.locator("#platform").selectOption(platform);
        assert.ok(
          (await page.locator("#download").getAttribute("href")).endsWith(
            asset,
          ),
        );
      }
      await page.locator("#platform").selectOption("mac");
      // Exercise clipboard denial instead of assuming permissions from localhost.
      await page.evaluate(() =>
        Object.defineProperty(navigator, "clipboard", {
          configurable: true,
          value: {
            writeText: async () => {
              throw Error("denied");
            },
          },
        }),
      );
      await page.locator("#copy").click();
      assert.match(
        await page.locator("#copy-status").textContent(),
        /Copy unavailable/,
      );
      await page.evaluate(() =>
        Object.defineProperty(navigator, "clipboard", {
          configurable: true,
          value: {
            writeText: async (text) => {
              window.copiedText = text;
            },
          },
        }),
      );
      await page.locator("#copy").click();
      assert.equal(
        await page.evaluate(() => window.copiedText),
        "mirket setup\nmirket doctor",
      );
      const summary = page.locator("summary").first();
      await summary.focus();
      await page.keyboard.press("Enter");
      assert.equal(
        await page.locator("details").first().getAttribute("open"),
        "",
      );
      await page.keyboard.press("Enter");
      for (const selector of [
        ".hero",
        "#approach",
        ".routing-section",
        "#toolchain",
        "#install",
        ".faq",
      ]) {
        await page.locator(selector).scrollIntoViewIfNeeded();
        await page.screenshot({
          path: path.join(
            output,
            `${width}-${selector.replace(/[.#]/g, "")}.png`,
          ),
        });
      }
      assert.deepEqual(errors, []);
      assert.deepEqual(failed, []);
      report.push({
        width,
        deviceScale: 2,
        overflow: false,
        interactions: "passed",
        consoleErrors: errors,
      });
    } finally {
      await context.close();
    }
  }
  const context = await browser.newContext({
    javaScriptEnabled: false,
    viewport: { width: 390, height: 844 },
  });
  try {
    const page = await context.newPage();
    await page.goto(url);
    assert.ok(await page.locator("noscript a").isVisible());
    assert.ok(await page.locator("#download").isVisible());
    assert.ok(
      await page.evaluate(
        () => document.documentElement.scrollWidth <= innerWidth,
      ),
    );
    report.push({ javascript: false, readable: true });
  } finally {
    await context.close();
  }
  await fs.writeFile(
    path.join(output, "browser-report.json"),
    JSON.stringify(report, null, 2) + "\n",
  );
  console.log(JSON.stringify(report));
} finally {
  await browser.close();
}
