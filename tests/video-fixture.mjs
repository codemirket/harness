// Test-only video fixture selection and revision-bound request construction.
// Production discovery and request APIs intentionally remain unchanged.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { lstat, open, readdir, realpath, writeFile } from "node:fs/promises";
import { join, relative } from "node:path";

export const SUPPORTED_VIDEO_ASSET = /\.(?:html|css|js|mjs|json|svg|vtt|srt|png|jpg|jpeg|webp|gif|ttf|otf|woff|woff2|wav|mp3|m4a|aac|opus|mp4|webm|ogg)$/i;
export const MAX_SOURCE_FILES = 128;
export const MAX_SOURCE_BYTES = 16 * 1024 * 1024;
export const digest = (bytes) => `sha256:${createHash("sha256").update(bytes).digest("hex")}`;

export function parseCaptionSelection(value) {
  assert.ok(typeof value === "string" && value.length > 0, "--captions requires PATH, none, or metadata-only");
  if (value === "none") return { mode: "none", path: null, tested: false };
  if (value === "metadata-only") return { mode: "metadata-only", path: "captions.json", tested: false };
  assert.ok(!value.startsWith("/") && !value.includes("\\") && value.split("/").every((part) => part && part !== "." && part !== ".."), "caption PATH must be project-relative");
  return { mode: "selected", path: value, tested: true };
}

async function walk(root) {
  const files = [];
  for (const item of await readdir(root, { withFileTypes: true })) {
    if (item.name === ".agents") continue;
    const file = join(root, item.name);
    assert.equal(item.isSymbolicLink(), false, `source symlink refused: ${file}`);
    if (item.isDirectory()) files.push(...await walk(file));
    else if (item.isFile() && item.name !== "preview.html") files.push(file);
  }
  return files.sort();
}

export async function collectRevisionBoundSources(sourceRoot) {
  const all = [];
  let totalBytes = 0;
  for (const source of await walk(sourceRoot)) {
    const path = relative(sourceRoot, source);
    if (!SUPPORTED_VIDEO_ASSET.test(path)) continue;
    assert.ok(all.length < MAX_SOURCE_FILES, `source file limit exceeded (${MAX_SOURCE_FILES})`);
    const before = await lstat(source);
    assert.ok(before.isFile() && !before.isSymbolicLink(), `source file type changed: ${source}`);
    assert.ok(before.size <= MAX_SOURCE_BYTES, `source file exceeds ${MAX_SOURCE_BYTES} bytes: ${path}`);
    assert.ok(totalBytes <= MAX_SOURCE_BYTES - before.size, `source byte limit exceeded (${MAX_SOURCE_BYTES} bytes)`);
    assert.equal(await realpath(source), source, `source symlink or ancestry changed: ${source}`);
    const handle = await open(source, "r");
    let data;
    try {
      const opened = await handle.stat();
      assert.ok(opened.isFile() && !opened.isSymbolicLink() && opened.dev === before.dev && opened.ino === before.ino && opened.size === before.size, `source changed before read: ${path}`);
      data = Buffer.alloc(before.size);
      let offset = 0;
      while (offset < data.length) {
        const result = await handle.read(data, offset, data.length - offset, offset);
        assert.ok(result.bytesRead > 0, `source truncated during read: ${path}`);
        offset += result.bytesRead;
      }
      const after = await lstat(source);
      assert.ok(after.isFile() && !after.isSymbolicLink() && after.dev === before.dev && after.ino === before.ino
        && after.size === before.size && after.mtimeMs === before.mtimeMs && after.ctimeMs === before.ctimeMs,
      `source changed during read: ${path}`);
    } finally {
      await handle.close();
    }
    totalBytes += data.length;
    all.push({ path, revision: digest(data), bytes: data.length, data });
  }
  assert.ok(all.length <= MAX_SOURCE_FILES, `source file limit exceeded (${MAX_SOURCE_FILES})`);
  assert.ok(all.some((entry) => entry.path === "index.html"), "source project needs root index.html");
  return all;
}

export function selectRevisionBoundSources(allSources, captions) {
  assert.ok(captions && ["none", "metadata-only", "selected"].includes(captions.mode));
  const selected = captions.mode === "none"
    ? allSources.filter((entry) => entry.path !== "captions.json" && entry.path !== captions.path)
    : allSources;
  if (captions.mode !== "none") assert.ok(selected.some((entry) => entry.path === captions.path), `selected caption source missing: ${captions.path}`);
  return { allSources, selectedSources: selected, captions };
}

export function buildVideoRequest({ projectRoot, release, selectedSources, captions }) {
  const entry = selectedSources.find((source) => source.path === "index.html");
  assert.ok(entry, "selected sources need index.html");
  return {
    expected_release_id: release,
    project_root: projectRoot,
    source_path: join(projectRoot, "index.html"),
    expected_revision: entry.revision,
    assets: selectedSources.filter((source) => source.path !== "index.html").map(({ path, revision }) => ({ path, expected_revision: revision })),
    destination_path: join(projectRoot, "result.mp4"),
    profile: { width: 640, height: 360, fps: { numerator: 12, denominator: 1 }, frame_count: 36, format: "mp4", captions_path: captions.mode === "selected" ? captions.path : null },
  };
}

export function sourceManifest(sources) {
  return sources.map(({ path, revision, bytes }) => ({ path, revision, bytes }));
}

export function retainVideoTrialFailure(record, error) {
  return { ...record, error: String(error), stack: error?.stack };
}

export async function writeVideoTrialEvidence(file, metadata, records) {
  await writeFile(file, JSON.stringify({ ...metadata, records }, null, 2), { mode: 0o600 });
}
