import assert from "node:assert/strict";
import { test } from "node:test";
import { mkdtemp, writeFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { createHash } from "node:crypto";
import { validateObservations } from "../scripts/check-motion-observations.mjs";

const sha = bytes => "sha256:" + createHash("sha256").update(bytes).digest("hex");

async function fixture(defect = null) {
  const root = await mkdtemp(path.join(tmpdir(), "motion-observations-"));
  const source = path.join(root, "index.html");
  const output = path.join(root, "result.mp4");
  const font = path.join(root, "NotoSans.ttf");
  const sourceBytes = Buffer.from("<main data-composition-id=scene data-no-timeline></main>");
  const outputBytes = Buffer.from("decoded export fixture");
  const fontBytes = Buffer.from("font fixture");
  await writeFile(source, sourceBytes); await writeFile(output, outputBytes); await writeFile(font, fontBytes);
  const frames = Array.from({ length: 5 }, (_, frame) => ({
    frame,
    timestamp_ms: frame * 1000 / 12,
    environment: { viewport: { width: 640, height: 360 }, dpr: 1, theme: "light", locale: "en-US" },
    entities: {
      stage: { x: 0, y: 0, width: 200, height: 100, visible: true },
      ball: { x: frame < 2 ? 10 + frame * 20 : 50, y: 10, width: 10, height: 10, visible: true },
      label: { x: 120, y: 10, width: 40, height: 20, visible: true,
        clip: { client_width: 40, client_height: 20, scroll_width: 40, scroll_height: 20, clipped: false } }
    }
  }));
  const packet = {
    schema_version: 1,
    capture: {
      id: "clean-control",
      source_revision: sha(sourceBytes),
      export_revision: sha(outputBytes),
      export_binding: "bound",
      environment: { viewport: { width: 640, height: 360 }, dpr: 1, theme: "light", locale: "en-US" },
      fps: { numerator: 12, denominator: 1 },
      frame_count: 5,
      measurement_basis: "decoded_frame_geometry",
      font: { path: "NotoSans.ttf", sha256: sha(fontBytes), loaded: true },
      frames
    },
    checks: [
      { id: "move", kind: "moves", entity: "ball", from: 0, to: 2, axis: "x", min_delta: 30 },
      { id: "hold", kind: "holds", entity: "ball", from: 2, to: 4, tolerance: 0 },
      { id: "settle", kind: "settles", entity: "ball", from: 3, to: 4, tolerance: 0 },
      { id: "inside", kind: "inside", entity: "ball", container: "stage", frames: [0, 1, 2, 3, 4] },
      { id: "label_position", kind: "position", entity: "label", from: 0, to: 4, expected: { x: 120, y: 10 }, tolerance: 0 },
      { id: "label_no_clip", kind: "no_clip", entity: "label", from: 0, to: 4 },
      { id: "separation", kind: "no_overlap", a: "ball", b: "label", frames: [0, 1, 2, 3, 4] }
    ]
  };
  if (defect === "midpoint") {
    packet.capture.frames[0].entities.ball.x = 50;
    packet.capture.frames[1].entities.ball.x = 70;
    packet.capture.frames[2].entities.ball.x = 50;
    packet.checks = [{ id: "endpoint_hold", kind: "holds", entity: "ball", from: 0, to: 4, tolerance: 0 }];
  }
  if (defect === "overlap") {
    packet.capture.frames[2].entities.label.x = 52;
    packet.checks = [{ id: "separation", kind: "no_overlap", a: "ball", b: "label", from: 0, to: 4 }];
  }
  if (defect === "font") packet.capture.font.loaded = false;
  if (defect === "identity") packet.capture.source_revision = sha(Buffer.from("stale source"));
  return { root, source, output, font, packet };
}

async function run(defect) {
  const value = await fixture(defect);
  try {
    return await validateObservations(value.packet, { source: value.source, export: value.output, font: value.font });
  } finally {
    await rm(value.root, { recursive: true, force: true });
  }
}

test("clean control passes movement, hold, settle, containment and no-overlap checks", async () => {
  const result = await run();
  assert.equal(result.status, "passed");
  assert.deepEqual(result.failures, []);
  assert.equal(result.coverage, "numeric_observation_contract_only; no_render_no_decode_no_visual_inspection");
  assert.deepEqual(result.checks.map(check => check.id), ["move", "hold", "settle", "inside", "label_position", "label_no_clip", "separation"]);
});

test("midpoint-only defect fails even when endpoints match", async () => {
  const result = await run("midpoint");
  assert.equal(result.status, "failed");
  assert.deepEqual(result.failures, ["endpoint_hold"]);
  assert.equal(result.checks[0].observed, 20);
});

test("containment and overlap defects fail at the affected frame", async () => {
  const result = await run("overlap");
  assert.equal(result.status, "failed");
  assert.deepEqual(result.failures, ["separation"]);
  assert.deepEqual(result.checks[0].observed.overlap_frames, [2]);
});

test("font readiness is required and does not come from a hash alone", async () => {
  await assert.rejects(run("font"), /font_not_ready/);
});

test("source identity mismatch is refused before accepting observations", async () => {
  await assert.rejects(run("identity"), /stale_source_revision/);
});

test("bound identity cannot pass without the actual export and reported unclipped cannot hide overflow", async () => {
  const value = await fixture();
  try {
    await assert.rejects(validateObservations(value.packet, { source: value.source, font: value.font }), /missing_file_export_revision/);
    value.packet.capture.frames[2].entities.label.clip.scroll_height = 25;
    const result = await validateObservations(value.packet, { source: value.source, export: value.output, font: value.font });
    assert.deepEqual(result.failures, ["label_no_clip"]);
    value.packet.capture.export_binding = "unbound_browser_capture";
    delete value.packet.capture.export_revision;
    await assert.rejects(validateObservations(value.packet, { source: value.source, font: value.font }), /decoded_capture_requires_export/);
  } finally { await rm(value.root, { recursive: true, force: true }); }
});

test("unsupported semantic checks are refused rather than silently passed", async () => {
  const value = await fixture();
  try {
    value.packet.checks = [{ id: "unknown", kind: "beautiful", entity: "ball", from: 0, to: 1 }];
    await assert.rejects(validateObservations(value.packet, { source: value.source, export: value.output, font: value.font }), /unsupported_check/);
  } finally {
    await rm(value.root, { recursive: true, force: true });
  }
});

test("frame timestamps are bound to the declared rational clock", async () => {
  const value = await fixture();
  try {
    value.packet.capture.frames[1].timestamp_ms = 750;
    await assert.rejects(validateObservations(value.packet, { source: value.source, export: value.output, font: value.font }), /frame_time_mismatch/);
  } finally {
    await rm(value.root, { recursive: true, force: true });
  }
});
