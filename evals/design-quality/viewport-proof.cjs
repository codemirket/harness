"use strict";

// Capture the pixels of actual viewports. Scrolled fullPage screenshots can
// paint offscreen fixed content inside the document image (including skip links).
const assert = require("node:assert/strict");
const fs = require("node:fs/promises");
const path = require("node:path");
const { createHash } = require("node:crypto");

async function geometry(page) {
  return page.evaluate(() => {
    const rect = el => {
      const r = el.getBoundingClientRect();
      return { x: r.x, y: r.y, width: r.width, height: r.height, bottom: r.bottom };
    };
    const active = document.activeElement;
    const indicator = el => {
      const s = getComputedStyle(el);
      return { tag: el.tagName, id: el.id, rect: rect(el),
        focused: el.matches(":focus"), focus_visible: el.matches(":focus-visible"),
        outline: { style: s.outlineStyle, width: s.outlineWidth, color: s.outlineColor, offset: s.outlineOffset },
        box_shadow: s.boxShadow };
    };
    const focusChain = [];
    for (let el = active; el && focusChain.length < 5; el = el.parentElement) focusChain.push(indicator(el));
    return {
      width: innerWidth, height: innerHeight, scroll_x: scrollX, scroll_y: scrollY,
      document_width: document.documentElement.scrollWidth,
      document_height: document.documentElement.scrollHeight,
      active: active ? { tag: active.tagName, id: active.id,
        text: (active.getAttribute("aria-label") || active.textContent || "").trim().slice(0, 300),
        rect: rect(active) } : null,
      focus_chain: focusChain,
      fixed: [...document.querySelectorAll("body *")].filter(el =>
        ["fixed", "sticky"].includes(getComputedStyle(el).position)).slice(0, 200).map(el => {
        const r = rect(el), s = getComputedStyle(el);
        return { tag: el.tagName, id: el.id, text: (el.textContent || "").trim().slice(0, 160),
          position: s.position, rect: r, focused: el === active,
          intersects_viewport: r.width > 0 && r.height > 0 && r.bottom > 0 && r.y < innerHeight
            && r.x + r.width > 0 && r.x < innerWidth && s.display !== "none"
            && s.visibility !== "hidden" && Number(s.opacity) > 0 };
      }),
    };
  });
}

async function settle(page) {
  await page.evaluate(() => new Promise(resolve =>
    requestAnimationFrame(() => requestAnimationFrame(resolve))));
}

async function captureViewportProof(page, outputDirectory, label, options = {}) {
  assert.match(label, /^[A-Za-z0-9][A-Za-z0-9._-]{0,60}$/);
  const overlap = options.overlap ?? 128, maxFrames = options.maxFrames ?? 64;
  assert.ok(Number.isInteger(overlap) && overlap >= 0);
  assert.ok(Number.isInteger(maxFrames) && maxFrames >= 2 && maxFrames <= 256);
  const directory = path.resolve(outputDirectory);
  assert.equal((await fs.lstat(directory)).isSymbolicLink(), false, "output directory symlink refused");
  assert.ok((await fs.stat(directory)).isDirectory());
  await settle(page);
  const initial = await geometry(page);
  assert.ok(initial.height > overlap, "overlap must be smaller than the viewport");
  const last = Math.max(0, initial.document_height - initial.height);
  const positions = [0];
  for (let y = initial.height - overlap; y < last; y += initial.height - overlap) positions.push(y);
  if (last > 0) positions.push(last);
  assert.ok(positions.length + 1 <= maxFrames, "document exceeds the predeclared frame cap");
  const destinations = positions.map((_, i) => `${label}-scroll-${String(i + 1).padStart(3, "0")}.png`);
  destinations.unshift(`${label}-current.png`);
  destinations.push(`${label}-viewports.json`);
  for (const file of destinations) {
    try { await fs.lstat(path.join(directory, file)); throw new Error(`output exists: ${file}`); }
    catch (e) { if (e.code !== "ENOENT") throw e; }
  }
  const result = { schema_version: 1, kind: "actual_viewport_proof", label,
    policy: { full_page: false, overlap, max_frames: maxFrames, animations: "disabled", scale: "css" },
    initial, frames: [], document_scan_complete: false, stable_dimensions: true,
    limits: ["Actual viewport pixels at recorded scroll positions; no stitched fullPage image.",
      "Scrolling is an observation action and can trigger sticky/lazy content; dimensions and positions are retained.",
      "Still frames do not establish motion quality, real device/AT support or human acceptance."] };
  async function capture(file, requestedY, role) {
    const before = await geometry(page);
    const pixels = await page.screenshot({ fullPage: false, animations: "disabled", scale: "css" });
    const after = await geometry(page);
    await fs.writeFile(path.join(directory, file), pixels, { flag: "wx" });
    result.stable_dimensions &&= before.width === initial.width && before.height === initial.height
      && before.document_height === initial.document_height && before.document_width === initial.document_width
      && JSON.stringify(before) === JSON.stringify(after);
    result.frames.push({ role, requested_y: requestedY, before, after,
      file, sha256: createHash("sha256").update(pixels).digest("hex"), bytes: pixels.length });
  }
  try {
    await capture(destinations[0], initial.scroll_y, "current");
    for (let i = 0; i < positions.length; i++) {
      const y = positions[i];
      await page.evaluate(y => scrollTo({ left: 0, top: y, behavior: "instant" }), y);
      await settle(page);
      await capture(destinations[i + 1], y, "document_scan");
    }
    const scan = result.frames.filter(f => f.role === "document_scan");
    result.document_scan_complete = result.stable_dimensions && scan.every((f, i) =>
      Math.abs(f.before.scroll_y - positions[i]) <= 1)
      && scan[0].before.scroll_y === 0
      && scan.at(-1).before.scroll_y + initial.height >= initial.document_height - 1
      && scan.every((f, i) => i === 0 || f.before.scroll_y <= scan[i - 1].before.scroll_y + initial.height);
  } finally {
    await page.evaluate(({ x, y }) => scrollTo({ left: x, top: y, behavior: "instant" }),
      { x: initial.scroll_x, y: initial.scroll_y });
    await settle(page);
    result.restored = await geometry(page);
    result.scroll_restored = Math.abs(result.restored.scroll_y - initial.scroll_y) <= 1
      && Math.abs(result.restored.scroll_x - initial.scroll_x) <= 1;
    result.stable_dimensions &&= result.restored.width === initial.width
      && result.restored.height === initial.height
      && result.restored.document_width === initial.document_width
      && result.restored.document_height === initial.document_height;
    result.focus_preserved = JSON.stringify(result.restored.active) === JSON.stringify(initial.active);
    result.document_scan_complete &&= result.stable_dimensions && result.scroll_restored && result.focus_preserved;
    await fs.writeFile(path.join(directory, destinations.at(-1)), JSON.stringify(result, null, 2) + "\n", { flag: "wx" });
  }
  return result;
}

module.exports = { captureViewportProof };
