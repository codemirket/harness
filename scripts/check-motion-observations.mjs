#!/usr/bin/env node
// Validate numeric geometry observations from an existing preview/decoded
// capture. This does not render, decode, view pixels, or decide visual quality.
// It is deliberately independent of the evidence packet validator.
import { readFile, lstat } from "node:fs/promises";
import { createHash } from "node:crypto";
import path from "node:path";
import { pathToFileURL } from "node:url";

const SHA = /^sha256:[a-f0-9]{64}$/;
const finite = value => typeof value === "number" && Number.isFinite(value);
const object = value => value !== null && typeof value === "object" && !Array.isArray(value);
const text = (value, field) => {
  if (typeof value !== "string" || value.length === 0 || value.length > 512) throw new ObservationError("invalid_" + field);
  return value;
};
const integer = (value, field, min = 0, max = Number.MAX_SAFE_INTEGER) => {
  if (!Number.isSafeInteger(value) || value < min || value > max) throw new ObservationError("invalid_" + field);
  return value;
};
const digest = bytes => "sha256:" + createHash("sha256").update(bytes).digest("hex");
const fileDigest = async file => digest(await readFile(file));

export class ObservationError extends Error {
  constructor(code, detail = "") {
    super(detail ? `${code}: ${detail}` : code);
    this.code = code;
  }
}

function geometry(value, field) {
  if (!object(value) || !["x", "y", "width", "height", "visible"].every(key => key in value)
    || !["x", "y", "width", "height"].every(key => finite(value[key]))
    || value.width < 0 || value.height < 0 || typeof value.visible !== "boolean") {
    throw new ObservationError("invalid_geometry", field);
  }
  let clip;
  if (value.clip !== undefined) {
    if (!object(value.clip) || !["client_width", "client_height", "scroll_width", "scroll_height", "clipped"].every(key => key in value.clip)
      || !["client_width", "client_height", "scroll_width", "scroll_height"].every(key => finite(value.clip[key]))
      || ["client_width", "client_height", "scroll_width", "scroll_height"].some(key => value.clip[key] < 0)
      || typeof value.clip.clipped !== "boolean") throw new ObservationError("invalid_clip_observation", field);
    clip = { client_width: value.clip.client_width, client_height: value.clip.client_height,
      scroll_width: value.clip.scroll_width, scroll_height: value.clip.scroll_height, clipped: value.clip.clipped };
  }
  return { x: value.x, y: value.y, width: value.width, height: value.height, visible: value.visible, ...(clip ? { clip } : {}) };
}

function bbox(frameMap, entity, frame) {
  const row = frameMap.get(frame);
  if (!row) throw new ObservationError("missing_frame", String(frame));
  if (!row.entities[entity]) throw new ObservationError("missing_entity", `${entity}@${frame}`);
  return row.entities[entity];
}

function framesIn(check, frameCount) {
  if (Array.isArray(check.frames)) {
    if (!check.frames.length) throw new ObservationError("empty_check_frames", check.id);
    return check.frames.map(frame => integer(frame, "check_frame", 0, frameCount - 1));
  }
  const from = integer(check.from, "check_from", 0, frameCount - 1);
  const to = integer(check.to, "check_to", from, frameCount - 1);
  return Array.from({ length: to - from + 1 }, (_, index) => from + index);
}

function visible(value, label) {
  if (!value.visible) throw new ObservationError("entity_not_visible", label);
}

function environment(value, field) {
  if (!object(value) || !object(value.viewport)
    || !finite(value.viewport.width) || !finite(value.viewport.height)
    || value.viewport.width < 2 || value.viewport.height < 2
    || !finite(value.dpr) || value.dpr <= 0
    || !["light", "dark"].includes(value.theme)
    || typeof value.locale !== "string" || value.locale.length < 2 || value.locale.length > 64) {
    throw new ObservationError("invalid_capture_environment", field);
  }
  return { viewport: { width: value.viewport.width, height: value.viewport.height }, dpr: value.dpr, theme: value.theme, locale: value.locale };
}

function sameEnvironment(a, b) {
  return a.dpr === b.dpr && a.theme === b.theme && a.locale === b.locale
    && a.viewport.width === b.viewport.width && a.viewport.height === b.viewport.height;
}

function delta(a, b) {
  return Math.max(Math.abs(a.x - b.x), Math.abs(a.y - b.y), Math.abs(a.width - b.width), Math.abs(a.height - b.height));
}

function overlaps(a, b, margin) {
  return a.x - margin < b.x + b.width && a.x + a.width + margin > b.x
    && a.y - margin < b.y + b.height && a.y + a.height + margin > b.y;
}

function inside(a, b, margin) {
  return a.x >= b.x + margin && a.y >= b.y + margin
    && a.x + a.width <= b.x + b.width - margin
    && a.y + a.height <= b.y + b.height - margin;
}

function checkOne(check, frameMap, frameCount) {
  if (!object(check)) throw new ObservationError("invalid_check");
  const id = text(check.id, "check_id");
  const kind = text(check.kind, "check_kind");
  const tolerance = check.tolerance === undefined ? 0 : check.tolerance;
  if (!finite(tolerance) || tolerance < 0) throw new ObservationError("invalid_tolerance", id);
  const frames = framesIn(check, frameCount);
  if (kind === "moves") {
    const entity = text(check.entity, "check_entity");
    const axis = check.axis === "y" ? "y" : check.axis === "x" ? "x" : check.axis === "distance" ? "distance" : null;
    if (!axis) throw new ObservationError("invalid_move_axis", id);
    const minDelta = check.min_delta;
    if (!finite(minDelta) || minDelta < 0) throw new ObservationError("invalid_min_delta", id);
    const first = bbox(frameMap, entity, frames[0]); const last = bbox(frameMap, entity, frames.at(-1));
    visible(first, `${entity}@${frames[0]}`); visible(last, `${entity}@${frames.at(-1)}`);
    const observed = axis === "distance"
      ? Math.hypot(last.x - first.x, last.y - first.y)
      : Math.abs(last[axis] - first[axis]);
    return { id, kind, passed: observed >= minDelta, frames, observed, expected: { min_delta: minDelta, axis } };
  }
  if (kind === "holds" || kind === "settles") {
    const entity = text(check.entity, "check_entity");
    const values = frames.map(frame => { const value = bbox(frameMap, entity, frame); visible(value, `${entity}@${frame}`); return value; });
    const observed = values.reduce((max, value) => Math.max(max, delta(values[0], value)), 0);
    return { id, kind, passed: observed <= tolerance, frames, observed, expected: { tolerance } };
  }
  if (kind === "inside") {
    const entity = text(check.entity, "check_entity"); const container = text(check.container, "check_container");
    const margin = check.margin === undefined ? 0 : check.margin;
    if (!finite(margin) || margin < 0) throw new ObservationError("invalid_margin", id);
    const failures = [];
    for (const frame of frames) {
      const child = bbox(frameMap, entity, frame); const parent = bbox(frameMap, container, frame);
      visible(child, `${entity}@${frame}`); visible(parent, `${container}@${frame}`);
      if (!inside(child, parent, margin)) failures.push(frame);
    }
    return { id, kind, passed: failures.length === 0, frames, observed: { failed_frames: failures }, expected: { margin } };
  }
  if (kind === "position") {
    const entity = text(check.entity, "check_entity");
    if (!object(check.expected) || (!finite(check.expected.x) && !finite(check.expected.y))) {
      throw new ObservationError("invalid_expected_position", id);
    }
    const expected = { ...(finite(check.expected.x) ? { x: check.expected.x } : {}), ...(finite(check.expected.y) ? { y: check.expected.y } : {}) };
    const failures = [];
    for (const frame of frames) {
      const value = bbox(frameMap, entity, frame); visible(value, `${entity}@${frame}`);
      if ((expected.x !== undefined && Math.abs(value.x - expected.x) > tolerance)
        || (expected.y !== undefined && Math.abs(value.y - expected.y) > tolerance)) failures.push(frame);
    }
    return { id, kind, passed: failures.length === 0, frames, observed: { failed_frames: failures }, expected: { ...expected, tolerance } };
  }
  if (kind === "no_clip") {
    const entity = text(check.entity, "check_entity");
    const failures = [];
    for (const frame of frames) {
      const value = bbox(frameMap, entity, frame); visible(value, `${entity}@${frame}`);
      if (!value.clip || value.clip.clipped || value.clip.scroll_width > value.clip.client_width
        || value.clip.scroll_height > value.clip.client_height) failures.push(frame);
    }
    return { id, kind, passed: failures.length === 0, frames, observed: { clipped_frames: failures }, expected: { clipped: false } };
  }
  if (kind === "no_overlap") {
    const firstEntity = text(check.a, "check_a"); const secondEntity = text(check.b, "check_b");
    const margin = check.margin === undefined ? 0 : check.margin;
    if (!finite(margin) || margin < 0) throw new ObservationError("invalid_margin", id);
    const failures = [];
    for (const frame of frames) {
      const first = bbox(frameMap, firstEntity, frame); const second = bbox(frameMap, secondEntity, frame);
      visible(first, `${firstEntity}@${frame}`); visible(second, `${secondEntity}@${frame}`);
      if (overlaps(first, second, margin)) failures.push(frame);
    }
    return { id, kind, passed: failures.length === 0, frames, observed: { overlap_frames: failures }, expected: { margin } };
  }
  throw new ObservationError("unsupported_check", kind);
}

async function verifyFileIdentity(value, file, field) {
  if (!SHA.test(value)) throw new ObservationError("invalid_" + field);
  if (!file) throw new ObservationError("missing_file_" + field);
  const metadata = await lstat(file);
  if (!metadata.isFile() || metadata.isSymbolicLink()) throw new ObservationError("invalid_file_" + field);
  const actual = await fileDigest(file);
  if (actual !== value) throw new ObservationError("stale_" + field);
}

export async function validateObservations(packet, files = {}) {
  if (!object(packet) || packet.schema_version !== 1) throw new ObservationError("invalid_schema");
  const capture = packet.capture;
  if (!object(capture)) throw new ObservationError("missing_capture");
  text(capture.id, "capture_id");
  if (!object(capture.fps)) throw new ObservationError("invalid_fps");
  const numerator = integer(capture.fps.numerator, "fps_numerator", 1, 60000);
  const denominator = integer(capture.fps.denominator, "fps_denominator", 1, 1001);
  if (numerator < denominator) throw new ObservationError("invalid_fps");
  const frameCount = integer(capture.frame_count, "frame_count", 1, 18000);
  if (frameCount * denominator > 600 * numerator) throw new ObservationError("motion_too_long");
  text(capture.measurement_basis, "measurement_basis");
  if (!["browser_frame_geometry", "decoded_frame_geometry", "decoded_frame_pixels"].includes(capture.measurement_basis)) {
    throw new ObservationError("unsupported_measurement_basis");
  }
  await verifyFileIdentity(capture.source_revision, files.source, "source_revision");
  const exportBinding = text(capture.export_binding, "export_binding");
  if (exportBinding === "bound") {
    await verifyFileIdentity(capture.export_revision, files.export, "export_revision");
  } else if (exportBinding === "unbound_browser_capture") {
    if (capture.measurement_basis !== "browser_frame_geometry") throw new ObservationError("decoded_capture_requires_export");
    if (capture.export_revision !== undefined && capture.export_revision !== null) throw new ObservationError("unexpected_export_revision");
    if (files.export) throw new ObservationError("unexpected_export_file");
  } else {
    throw new ObservationError("invalid_export_binding");
  }
  const captureEnvironment = environment(capture.environment, "capture_environment");
  if (!object(capture.font) || typeof capture.font.loaded !== "boolean" || !capture.font.loaded) {
    throw new ObservationError("font_not_ready");
  }
  text(capture.font.path, "font_path");
  if (!capture.font.sha256) throw new ObservationError("missing_font_revision");
  await verifyFileIdentity(capture.font.sha256, files.font, "font_revision");
  const rawFrames = capture.frames;
  if (!Array.isArray(rawFrames) || rawFrames.length < 1 || rawFrames.length > frameCount) {
    throw new ObservationError("invalid_frames");
  }
  const frameMap = new Map();
  for (const row of rawFrames) {
    if (!object(row)) throw new ObservationError("invalid_frame");
    const frame = integer(row.frame, "frame", 0, frameCount - 1);
    if (!finite(row.timestamp_ms) || row.timestamp_ms < 0
      || Math.abs(row.timestamp_ms - (frame * denominator * 1000 / numerator)) > 0.001) {
      throw new ObservationError("frame_time_mismatch", String(frame));
    }
    const sampleEnvironment = environment(row.environment, `frame_environment@${frame}`);
    if (!sameEnvironment(sampleEnvironment, captureEnvironment)) throw new ObservationError("capture_environment_mismatch", String(frame));
    if (frameMap.has(frame) || !object(row.entities)) throw new ObservationError("duplicate_or_invalid_frame", String(frame));
    const entities = {};
    for (const [id, value] of Object.entries(row.entities)) entities[text(id, "entity_id")] = geometry(value, `${id}@${frame}`);
    frameMap.set(frame, { frame, timestamp_ms: row.timestamp_ms, environment: sampleEnvironment, entities });
  }
  const checks = packet.checks;
  if (!Array.isArray(checks) || checks.length < 1 || checks.length > 128) throw new ObservationError("invalid_checks");
  const seen = new Set(); const results = [];
  for (const check of checks) {
    const result = checkOne(check, frameMap, frameCount);
    if (seen.has(result.id)) throw new ObservationError("duplicate_check", result.id);
    seen.add(result.id); results.push(result);
  }
  const failures = results.filter(result => !result.passed);
  return {
    status: failures.length ? "failed" : "passed",
    capture_id: capture.id,
    checks: results,
    failures: failures.map(result => result.id),
    coverage: "numeric_observation_contract_only; no_render_no_decode_no_visual_inspection",
    identity: { source_revision: capture.source_revision, export_binding: exportBinding, export_revision: capture.export_revision ?? null, font_revision: capture.font.sha256 ?? null },
    environment: captureEnvironment
  };
}

function cli(args) {
  if (args.length < 2 || args.length > 8 || args[0] !== "--observations") throw new ObservationError("usage");
  const options = { observations: args[1] };
  for (let i = 2; i < args.length; i += 2) {
    const key = args[i]?.replace(/^--/, ""); const value = args[i + 1];
    if (!["source", "export", "font"].includes(key) || !value || options[key]) throw new ObservationError("usage");
    options[key] = value;
  }
  if (!options.source || !options.font) throw new ObservationError("usage_requires_source_and_font");
  return options;
}

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  try {
    const options = cli(process.argv.slice(2));
    const packet = JSON.parse(await readFile(options.observations, "utf8"));
    for (const key of ["source", "export", "font"]) if (options[key]) {
      const info = await lstat(options[key]);
      if (!info.isFile() || info.isSymbolicLink()) throw new ObservationError("invalid_" + key);
    }
    const result = await validateObservations(packet, options);
    process.stdout.write(JSON.stringify(result) + "\n");
    if (result.status !== "passed") process.exitCode = 1;
  } catch (error) {
    process.stdout.write(JSON.stringify({ status: "refused", code: error?.code ?? "invalid_observations", coverage: "numeric_observation_contract_only; no_render_no_decode_no_visual_inspection" }) + "\n");
    process.exitCode = 1;
  }
}
