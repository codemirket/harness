import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import childProcess from "node:child_process";
import test from "node:test";
import { inspectCodeSource, CODE_SOURCE_LIMITS } from "../dist/code-source-inspect.js";

const input = (text) => Buffer.from(text, "utf8");
const revision = (text) => "sha256:" + createHash("sha256").update(input(text)).digest("hex");
const inspect = (text, language = "ts", options = {}) => inspectCodeSource(input(text), { language, ...options });
function occurrences(text, language = "ts") {
  const first = inspect(text, language), result = [], pending = [first];
  assert.equal(first.status, "page");
  while (pending.length) {
    const page = pending.pop();
    result.push(page.selected);
    for (const child of page.children) {
      if (child.child_count) {
        const next = inspect(text, language, { pointer: child.pointer, expected_revision: first.revision });
        assert.equal(next.status, "page"); assert.equal(next.complete, true); pending.push(next);
      } else result.push(child);
    }
  }
  return result;
}
test("fixed JS/TS/JSX/TSX syntax owners preserve original BOM, Unicode and CRLF bytes", () => {
  for (const [language, body, target] of [
    ["ts", "export function çay(x: number) { return x; }", "FunctionDeclaration"],
    ["js", "function çay(x) { return x; }", "FunctionDeclaration"],
    ["tsx", "const çay = <Panel value={1}/>;", "JsxSelfClosingElement"],
    ["jsx", "const çay = <Panel value={1}/>;", "JsxSelfClosingElement"],
  ]) {
    const text = "\ufeff// 😀\r\n" + body + "\r\n";
    const page = inspect(text, language), all = occurrences(text, language);
    assert.deepEqual(page.backend, { name: "typescript", version: "5.9.3" });
    assert.equal(page.source_bytes, input(text).length); assert.equal(page.revision, revision(text));
    assert.equal(page.structure_complete, true); assert.ok(all.some((node) => node.kind === target));
    assert.deepEqual(page.selected.location, { start_byte: 0, end_byte: input(text).length });
    const name = all.find((node) => node.kind === "Identifier" && node.name === "çay");
    assert.ok(name); assert.equal(name.location.end_byte, input(text.slice(0, text.indexOf("çay") + 3)).length);
    assert.ok(input(text).subarray(name.location.start_byte, name.location.end_byte).toString("utf8").endsWith("çay"));
    for (const node of all) {
      assert.ok(node.location);
      const slice = input(text).subarray(node.location.start_byte, node.location.end_byte);
      assert.deepEqual(input(new TextDecoder("utf8", { fatal: true, ignoreBOM: true }).decode(slice)), slice);
    }
  }
});
test("syntax names retain decoded escapes and leading underscores without semantic resolution", () => {
  const all = occurrences("const __name = 1; const caf\\u00e9 = __name;");
  assert.ok(all.some((node) => node.name === "__name"));
  assert.ok(all.some((node) => node.name === "café"));
  assert.equal(all.find((node) => node.kind === "VariableStatement").kind, "VariableStatement");
  assert.ok(!all.some((node) => node.kind === "FirstStatement"));
});
test("valid zero-width syntax occurrences retain proven empty spans", () => {
  for (const [language, text, kind] of [["tsx", "const view = <Box/>;", "JsxAttributes"],
    ["jsx", "const view = <Box/>;", "JsxAttributes"], ["ts", "const list = [,,];", "OmittedExpression"]]) {
    const page = inspect(text, language), all = occurrences(text, language);
    assert.equal(page.structure_complete, true);
    const empty = all.find((node) => node.kind === kind);
    assert.ok(empty); assert.ok(empty.location);
    assert.equal(empty.location.start_byte, empty.location.end_byte);
  }
});
test("recovery and missing syntax never invent a successful complete tree or missing-token span", () => {
  for (const text of ["const x = ;", "function broken( {", "const a = <Panel>"]) {
    const page = inspect(text, "tsx"), all = occurrences(text, "tsx");
    assert.equal(page.status, "page"); assert.equal(page.structure_complete, false);
    assert.ok(page.issues.length > 0); assert.match(page.issues[0].reason, /^typescript_parse_[0-9]+$/);
    assert.ok(all.some((node) => node.location === null && /^(typescript_recovery|missing_syntax)$/.test(node.location_reason)));
  }
});
test("ordinal selection and pagination bind the entire original source revision", () => {
  const text = "let a;\r\nlet b;\r\nlet c;", page = inspect(text, "ts", { limit: 1 });
  assert.equal(page.children.length, 1); assert.equal(page.complete, false);
  assert.deepEqual(page.continuation, { pointer: "", expected_revision: revision(text), start_child: 1, limit: 1 });
  const next = inspect(text, "ts", { ...page.continuation });
  assert.equal(next.children[0].pointer, "/1"); assert.equal(next.children[0].kind, "VariableStatement");
  assert.equal(inspect(text, "ts", { pointer: "/0" }).reason, "invalid_request");
  assert.equal(inspect(text, "ts", { start_child: 1 }).reason, "invalid_request");
  assert.equal(inspect(text + " ", "ts", { ...page.continuation }).status, "changed");
  assert.equal(inspect(text, "ts", { pointer: "/999", expected_revision: revision(text) }).status, "missing_pointer");
  assert.equal(inspect(text, "ts", { start_child: 999, expected_revision: revision(text) }).status, "invalid_start");
});
test("source/node/depth/metadata/result limits refuse or block without a partial success", () => {
  assert.equal(inspect(" ".repeat(CODE_SOURCE_LIMITS.source_bytes + 1)).reason, "source_too_large");
  assert.equal(inspect(";".repeat(CODE_SOURCE_LIMITS.nodes)).reason, "node_limit");
  assert.equal(inspect("(".repeat(80) + "1" + ")".repeat(80)).reason, "depth_limit");
  const text = "const " + "a".repeat(5000) + " = 1;";
  const block = inspect(text, "ts", { pointer: "/0/0/0", expected_revision: revision(text) });
  assert.equal(block.status, "budget_blocked"); assert.equal(block.continuation, null);
  assert.equal(block.blocked_child.pointer, "/0/0/0/0"); assert.ok(block.blocked_child.location);
  assert.ok(Buffer.byteLength(JSON.stringify(block)) <= CODE_SOURCE_LIMITS.response_bytes);
  const page = inspect("let a;".repeat(100), "ts", { max_response_bytes: 2048 });
  assert.equal(page.status, "page"); assert.ok(page.children.length > 0); assert.equal(page.complete, false);
  assert.ok(Buffer.byteLength(JSON.stringify(page)) <= 2048);
});
test("invalid byte/request/provider forms refuse and each script kind is selected explicitly", () => {
  assert.equal(inspectCodeSource(Buffer.from([0xc3, 0x28]), { language: "ts" }).reason, "invalid_utf8");
  assert.equal(inspect("const x = '\0';").reason, "nul_source");
  for (const request of [{ language: "html" }, { language: ["ts"] }, { language: "ts", path: "/never-read" },
    { language: "ts", plugins: [] }, { language: "ts", pointer: "/01" }, { language: "ts", limit: 0 }]) {
    assert.equal(inspectCodeSource(input("let x;"), request).reason, "invalid_request");
  }
  assert.ok(occurrences("const x = <Panel/>;", "tsx").some((node) => node.kind === "JsxSelfClosingElement"));
  assert.equal(inspect("const x = <Panel/>;", "ts").structure_complete, false);
});
test("qualified parser walk causes zero filesystem, process or compiler-model calls after trusted import", () => {
  const originals = [], calls = [];
  const replace = (object, key) => {
    if (typeof object[key] !== "function") return;
    originals.push([object, key, object[key]]);
    object[key] = () => { calls.push(key); throw new Error("forbidden passive parser effect"); };
  };
  for (const key of ["readFileSync", "readFile", "readdirSync", "readdir", "statSync", "lstatSync", "existsSync",
    "realpathSync", "openSync", "writeFileSync", "writeFile", "watch", "watchFile"]) replace(fs, key);
  for (const key of ["spawn", "spawnSync", "exec", "execSync", "execFile", "execFileSync", "fork"]) replace(childProcess, key);
  try {
    for (const [language, source] of [["ts", "import value from './never-resolve'; const n: number = value;"],
      ["js", "const value = require('never-resolve');"], ["tsx", "const X = () => <Box/>;"], ["jsx", "const X = <Box/>;"]]) {
      assert.equal(inspect(source, language).status, "page");
    }
    assert.deepEqual(calls, []);
  } finally { for (const [object, key, original] of originals) object[key] = original; }
});
