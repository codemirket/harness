import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { mkdtemp, readFile, rm, stat, symlink, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import test from "node:test";

const entry = fileURLToPath(new URL("../dist/runtime/design-main.js", import.meta.url));
const release = "m4.design-test";

async function fixture(t, input, operation, extra = {}) {
  const root = await mkdtemp(join(tmpdir(), "mirket-design-adapter-"));
  t.after(() => rm(root, { recursive: true, force: true }));
  const inputPath = join(root, operation === "svg.optimize" ? "input.svg" : "input.json");
  const outputPath = join(root, operation === "svg.optimize" ? "stage.svg" : "stage.out");
  await writeFile(inputPath, input);
  return {
    root,
    inputPath,
    outputPath,
    request: {
      type: "run", protocol_version: 1, release_id: release, job_id: "design-job-1",
      input_path: inputPath, output_path: outputPath, operation,
      deadline_ms: 30_000, ...extra,
    },
  };
}

function invoke(request, { raw, args = ["--release-id", release] } = {}) {
  return new Promise((resolve, reject) => {
    const child = spawn(process.execPath, [entry, ...args], { stdio: ["pipe", "pipe", "pipe"] });
    let stdout = "";
    let stderr = "";
    const timeout = setTimeout(() => { child.kill("SIGKILL"); reject(new Error("Design adapter did not exit")); }, 10_000);
    child.on("error", reject);
    child.stdout.setEncoding("utf8").on("data", (chunk) => { stdout += chunk; });
    child.stderr.setEncoding("utf8").on("data", (chunk) => { stderr += chunk; });
    child.stdin.on("error", () => {});
    child.on("close", (code, signal) => {
      clearTimeout(timeout);
      try {
        resolve({ code, signal, stderr, messages: stdout.trim().split("\n").filter(Boolean).map(JSON.parse) });
      } catch (error) {
        reject(error);
      }
    });
    child.stdin.end(raw ?? JSON.stringify(request) + "\n");
  });
}

const rasterModule = process.env.MIRKET_RASTER_MODULE;

test("private raster module transforms pixels and refuses mismatched image claims", { skip: !rasterModule }, async (t) => {
  const { default: sharp } = await import(pathToFileURL(rasterModule).href);
  const source = await sharp({
    create: { width: 2, height: 2, channels: 4, background: { r: 255, g: 0, b: 0, alpha: 0.5 } },
  }).png().toBuffer();
  const root = await mkdtemp(join(tmpdir(), "mirket-raster-adapter-"));
  t.after(() => rm(root, { recursive: true, force: true }));
  const inputPath = join(root, "input.png");
  const outputPath = join(root, "stage.webp");
  await writeFile(inputPath, source);
  const request = {
    type: "run", protocol_version: 1, release_id: release, job_id: "raster-job-1",
    input_path: inputPath, output_path: outputPath, operation: "raster.transform",
    source_format: "png", output_format: "webp", width: 4, height: 4, fit: "fill", deadline_ms: 30_000,
  };
  const args = ["--release-id", release, "--raster-module", rasterModule];
  const result = await invoke(request, { args });
  assert.equal(result.code, 0, JSON.stringify(result.messages));
  assert.deepEqual(result.messages[0].operations, ["tokens.compile", "svg.optimize", "raster.transform"]);
  assert.equal(result.messages.at(-1).width, 4);
  assert.equal(result.messages.at(-1).height, 4);
  const pixels = await sharp(await readFile(outputPath)).raw().toBuffer({ resolveWithObject: true });
  assert.equal(pixels.info.width, 4);
  assert.equal(pixels.info.height, 4);
  assert.equal(pixels.info.channels, 4);
  assert.ok(pixels.data[0] > 240);
  assert.ok(pixels.data[3] > 110 && pixels.data[3] < 150);

  for (const [sourceFormat, outputFormat] of [
    ["png", "jpg"], ["png", "avif"], ["avif", "png"], ["webp", "png"],
  ]) {
    const sourceBytes = sourceFormat === "png" ? source :
      await sharp(source)[sourceFormat === "avif" ? "avif" : "webp"]().toBuffer();
    const alternateInput = join(root, `alternate-${sourceFormat}-${outputFormat}.${sourceFormat}`);
    const alternateOutput = join(root, `alternate-${sourceFormat}-${outputFormat}.${outputFormat}`);
    await writeFile(alternateInput, sourceBytes);
    const converted = await invoke({ ...request, input_path: alternateInput, output_path: alternateOutput,
      source_format: sourceFormat, output_format: outputFormat }, { args });
    assert.equal(converted.code, 0, `${sourceFormat}->${outputFormat}: ${JSON.stringify(converted.messages)}`);
    const meta = await sharp(await readFile(alternateOutput)).metadata();
    assert.equal(meta.width, 4);
    assert.equal(meta.height, 4);
    assert.equal(meta.format === "heif" ? "avif" : meta.format === "jpeg" ? "jpg" : meta.format, outputFormat);
  }

  const badOutput = join(root, "bad.webp");
  const wrongFormat = await invoke({ ...request, output_path: badOutput, source_format: "jpg" }, { args });
  assert.equal(wrongFormat.messages.at(-1).code, "raster_invalid");
  await assert.rejects(stat(badOutput), { code: "ENOENT" });
  const corruptInput = join(root, "corrupt.png");
  await writeFile(corruptInput, "not a PNG");
  const corruptOutput = join(root, "corrupt.webp");
  const corrupt = await invoke({ ...request, input_path: corruptInput, output_path: corruptOutput }, { args });
  assert.equal(corrupt.messages.at(-1).code, "raster_invalid");
  await assert.rejects(stat(corruptOutput), { code: "ENOENT" });
});

const tokens = JSON.stringify({ color: { brand: { value: "#1463ff" }, accent: { value: "{color.brand}" } } });
const svg = '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10"><rect width="10" height="10" fill="#1463ff"/></svg>';

test("compiles tokens into an exclusive stage with identities and hashes", async (t) => {
  const f = await fixture(t, tokens, "tokens.compile", { format: "css" });
  const result = await invoke(f.request);
  assert.equal(result.code, 0);
  assert.deepEqual(result.messages[0].operations, ["tokens.compile", "svg.optimize"]);
  const terminal = result.messages.at(-1);
  assert.equal(terminal.type, "result");
  assert.equal(terminal.operation, "tokens.compile");
  assert.equal(terminal.format, "css");
  assert.equal(terminal.token_count, 2);
  assert.match(terminal.input_sha256, /^[0-9a-f]{64}$/u);
  assert.match(terminal.output_sha256, /^[0-9a-f]{64}$/u);
  assert.match(await readFile(f.outputPath, "utf8"), /--color-brand: #1463ff;/u);
  assert.deepEqual(await readFile(f.inputPath, "utf8"), tokens);
});

test("optimizes a bounded SVG and refuses unsafe frames", async (t) => {
  const f = await fixture(t, svg, "svg.optimize");
  const result = await invoke(f.request);
  assert.equal(result.code, 0);
  assert.equal(result.messages.at(-1).operation, "svg.optimize");
  assert.match(await readFile(f.outputPath, "utf8"), /<svg/u);

  for (const unsafe of [
    '<!DOCTYPE svg><svg xmlns="http://www.w3.org/2000/svg"/>',
    '<svg xmlns="http://www.w3.org/2000/svg"><script>alert(1)</script></svg>',
    '<svg xmlns="http://www.w3.org/2000/svg"><rect onclick="alert(1)"/></svg>',
    '<svg xmlns="http://www.w3.org/2000/svg"><image href="https://example.test/a.png"/></svg>',
  ]) {
    const bad = await fixture(t, unsafe, "svg.optimize");
    const refusal = await invoke(bad.request);
    assert.equal(refusal.code, 1);
    assert.equal(refusal.messages.at(-1).code, "svg_rejected");
    assert.equal(refusal.messages.at(-1).output_may_exist, false);
    await assert.rejects(stat(bad.outputPath), { code: "ENOENT" });
  }
});

test("rejects stale releases, duplicate stages, and malformed protocol frames", async (t) => {
  const f = await fixture(t, tokens, "tokens.compile", { format: "json" });
  const stale = await invoke({ ...f.request, release_id: "old-release" });
  assert.equal(stale.messages.at(-1).code, "release_mismatch");
  await writeFile(f.outputPath, "keep");
  const existing = await invoke(f.request);
  assert.equal(existing.messages.at(-1).code, "output_exists");
  assert.equal(await readFile(f.outputPath, "utf8"), "keep");
  const malformed = await invoke(null, { raw: JSON.stringify(f.request) });
  assert.equal(malformed.messages.at(-1).code, "invalid_frame");
});

test("requires operation-specific fields and bounds the deadline", async (t) => {
  const f = await fixture(t, tokens, "tokens.compile", { format: "css" });
  const missingFormat = { ...f.request };
  delete missingFormat.format;
  const invalid = await invoke(missingFormat);
  assert.equal(invalid.messages.at(-1).code, "invalid_request");
  const tooShort = await invoke({ ...f.request, deadline_ms: 99 });
  assert.equal(tooShort.messages.at(-1).code, "invalid_request");
});

test("refuses source symlinks before writing", async (t) => {
  const f = await fixture(t, tokens, "tokens.compile", { format: "json" });
  const link = join(f.root, "link.json");
  await symlink(f.inputPath, link);
  const result = await invoke({ ...f.request, input_path: link });
  assert.equal(result.messages.at(-1).code, "invalid_source");
  await assert.rejects(stat(f.outputPath), { code: "ENOENT" });
});
