import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { preflight } from "../adapters/dist/video-export.js";
import { buildVideoRequest, collectRevisionBoundSources, MAX_SOURCE_BYTES, MAX_SOURCE_FILES, parseCaptionSelection, retainVideoTrialFailure, selectRevisionBoundSources, writeVideoTrialEvidence } from "./video-fixture.mjs";

async function fixture() {
  const root = await fs.realpath(await fs.mkdtemp(path.join(os.tmpdir(), "mirket-video-fixture-")));
  await fs.writeFile(path.join(root, "index.html"), '<!doctype html><html><head><style>@font-face{font-family:Local;src:url("./font.ttf")}</style></head><body><main data-composition-id="fixture" data-no-timeline data-width="640" data-height="360" data-duration="3"></main><script src="./app.js"></script></body></html>');
  await fs.writeFile(path.join(root, "font.ttf"), "font fixture");
  await fs.writeFile(path.join(root, "app.js"), "console.log('fixture')");
  await fs.writeFile(path.join(root, "captions.json"), JSON.stringify([{ start_frame: 0, end_frame: 3, text: "Valid" }]));
  return root;
}

function relevant(request) {
  return { expected_revision: request.expected_revision, assets: request.assets, profile: request.profile };
}

test("revision-bound fixture requests are deterministic and caption selection is explicit", async () => {
  const root = await fixture();
  try {
    const all = await collectRevisionBoundSources(root);
    const selected = selectRevisionBoundSources(all, parseCaptionSelection("captions.json"));
    const first = buildVideoRequest({ projectRoot: "/tmp/project-a", release: "test:video", selectedSources: selected.selectedSources, captions: selected.captions });
    const second = buildVideoRequest({ projectRoot: "/tmp/project-b", release: "test:video", selectedSources: selected.selectedSources, captions: selected.captions });
    assert.deepEqual(Object.keys(first).sort(), ["assets", "destination_path", "expected_release_id", "expected_revision", "profile", "project_root", "source_path"]);
    assert.deepEqual(relevant(first), relevant(second));
    assert.equal(first.profile.captions_path, "captions.json");
    assert.ok(first.assets.some((asset) => asset.path === "captions.json"));

    const metadata = selectRevisionBoundSources(all, parseCaptionSelection("metadata-only"));
    const metadataRequest = buildVideoRequest({ projectRoot: root, release: "test:video", selectedSources: metadata.selectedSources, captions: metadata.captions });
    assert.equal(metadata.captions.tested, false);
    assert.equal(metadataRequest.profile.captions_path, null);
    assert.ok(metadataRequest.assets.some((asset) => asset.path === "captions.json"));

    const none = selectRevisionBoundSources(all, parseCaptionSelection("none"));
    const noneRequest = buildVideoRequest({ projectRoot: root, release: "test:video", selectedSources: none.selectedSources, captions: none.captions });
    assert.equal(noneRequest.profile.captions_path, null);
    assert.equal(noneRequest.assets.some((asset) => asset.path === "captions.json"), false);
  } finally {
    await fs.rm(root, { recursive: true, force: true });
  }
});

test("selected malformed captions fail compiled adapter preflight; metadata-only stays untested", async () => {
  const root = await fixture();
  try {
    await fs.writeFile(path.join(root, "captions.json"), JSON.stringify([{ start_frame: 0, end_frame: 0, text: "invalid" }]));
    const all = await collectRevisionBoundSources(root);
    const selected = selectRevisionBoundSources(all, parseCaptionSelection("captions.json"));
    const intent = buildVideoRequest({ projectRoot: root, release: "test:video", selectedSources: selected.selectedSources, captions: selected.captions });
    const request = {
      protocol_version: 1, release_id: "test:video", job_id: "fixture-captions", job_root: root,
      input_root: root, entry_file: "index.html", sources: selected.selectedSources.map(({ path, revision, bytes }) => ({ path, revision, bytes })),
      output_path: path.join(root, "result.mp4"), profile: intent.profile, timeout_ms: 30_000, max_output_bytes: 20 * 1024 * 1024,
    };
    await assert.rejects(preflight(request), /invalid_caption_cues/);
    const metadata = selectRevisionBoundSources(all, parseCaptionSelection("metadata-only"));
    assert.equal(metadata.captions.tested, false);
  } finally {
    await fs.rm(root, { recursive: true, force: true });
  }
});

test("source collection refuses oversized bodies before allocating them", async () => {
  const root = await fs.realpath(await fs.mkdtemp(path.join(os.tmpdir(), "mirket-video-fixture-limit-")));
  try {
    await fs.writeFile(path.join(root, "index.html"), "<html></html>");
    await fs.writeFile(path.join(root, "oversized.js"), "");
    await fs.truncate(path.join(root, "oversized.js"), MAX_SOURCE_BYTES + 1);
    await assert.rejects(collectRevisionBoundSources(root), /exceeds/);
  } finally {
    await fs.rm(root, { recursive: true, force: true });
  }
});

test("source collection refuses more than the adapter file-count contract", async () => {
  const root = await fs.realpath(await fs.mkdtemp(path.join(os.tmpdir(), "mirket-video-fixture-count-")));
  try {
    await fs.writeFile(path.join(root, "index.html"), "<html></html>");
    for (let index = 0; index < MAX_SOURCE_FILES; index++) {
      await fs.writeFile(path.join(root, `asset-${String(index).padStart(3, "0")}.js`), "");
    }
    await assert.rejects(collectRevisionBoundSources(root), /file limit exceeded/);
  } finally {
    await fs.rm(root, { recursive: true, force: true });
  }
});

test("trial failures retain the partial record and persist it before cleanup", async () => {
  const root = await fs.realpath(await fs.mkdtemp(path.join(os.tmpdir(), "mirket-video-trial-record-")));
  try {
    const records = [];
    const partial = { name: "video-current", source_root: "/frozen/source", prepare: { status: "prepared" } };
    records.push(retainVideoTrialFailure(partial, new Error("renderer refused")));
    const file = path.join(root, "runtime-oracle.json");
    await writeVideoTrialEvidence(file, { schema: 1, limits: ["no external runtime"] }, records);
    const saved = JSON.parse(await fs.readFile(file, "utf8"));
    assert.equal(saved.records.length, 1);
    assert.equal(saved.records[0].prepare.status, "prepared");
    assert.match(saved.records[0].error, /renderer refused/);
  } finally {
    await fs.rm(root, { recursive: true, force: true });
  }
});
