import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import test from "node:test";

const text = "const çay = 1; çay;", release = "0.0.1:navigation-test";
const revision = "sha256:" + createHash("sha256").update(text).digest("hex");
const frame = { protocol_version: 1, release_id: release, job_id: "op-navigation-test", request: {
  sources: [{ language: "ts", source_utf8: text, expected_revision: revision }], source: 0,
  typescript_model: { imports: [] },
  position_byte: Buffer.byteLength(text.slice(0, text.lastIndexOf("çay"))), operation: "references", limit: 100, max_response_bytes: 30000 } };
function run(input, args = ["--release-id", release]) {
  const result = spawnSync(process.execPath, [new URL("../dist/runtime/code-navigation-main.js", import.meta.url).pathname, ...args],
    { input, encoding: "utf8", timeout: 10000, maxBuffer: 100000 });
  assert.equal(result.error, undefined); assert.equal(result.stderr, "");
  const lines = result.stdout.trimEnd().split("\n"); assert.equal(lines.length, 1);
  return { code: result.status, value: JSON.parse(lines[0]) };
}
test("finite byte-only navigation transport preserves identity and original spans", () => {
  const result = run(JSON.stringify(frame) + "\n");
  assert.equal(result.code, 0); assert.equal(result.value.job_id, frame.job_id); assert.equal(result.value.release_id, release);
  assert.equal(result.value.result.status, "locations"); assert.equal(result.value.result.locations.length, 2);
});
test("navigation transport rejects wrong identity framing unknown keys and byte overflow", () => {
  for (const [input, error] of [[JSON.stringify({ ...frame, release_id: "wrong" }) + "\n", "invalid_request"],
    [JSON.stringify({ ...frame, project_root: "/project" }) + "\n", "invalid_request"],
    [JSON.stringify(frame), "invalid_frame"], [JSON.stringify(frame) + "\n{}\n", "invalid_frame"],
    ["x".repeat(2 * 1024 * 1024 + 1), "request_limit"]]) {
    const result = run(input); assert.equal(result.code, 1); assert.equal(result.value.error, error);
  }
  assert.equal(run(JSON.stringify(frame) + "\n", ["--config", "/tsconfig.json"]).value.error, "invalid_startup");
});
