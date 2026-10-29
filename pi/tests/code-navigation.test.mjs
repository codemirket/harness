import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import childProcess from "node:child_process";
import net from "node:net";
import http from "node:http";
import https from "node:https";
import dns from "node:dns";
import ts from "typescript";
import test from "node:test";
import { navigateCode, CODE_NAVIGATION_LIMITS as limits } from "../dist/runtime/code-navigation.js";

const revision = (text) => "sha256:" + createHash("sha256").update(Buffer.from(text)).digest("hex");
const saved = (text, language = "ts") => ({ language, source_utf8: text, expected_revision: revision(text) });
const byte = (text, target, last = true) => Buffer.byteLength(text.slice(0, last ? text.lastIndexOf(target) : text.indexOf(target)));
const span = (text, target, last = true) => ({ start_byte: byte(text, target, last), end_byte: byte(text, target, last) + Buffer.byteLength(target) });
const request = (sources, source, position_byte, options = {}) => ({ sources, source, position_byte,
  typescript_model: { imports: [] }, operation: "definitions", limit: 100, max_response_bytes: limits.response_bytes, ...options });
const navigate = (texts, source, target, options = {}) => navigateCode(request(texts.map((text) => typeof text === "string" ? saved(text) : text), source,
  byte(typeof texts[source] === "string" ? texts[source] : texts[source].source_utf8, target), options));
const location = (result) => { assert.equal(result.status, "locations", JSON.stringify(result)); return result.locations; };

test("scope-aware definitions and references distinguish identical nested names", () => {
  const text = "const value = 1; function f(value: number) { return value; } value;";
  const outer = navigate([text], 0, "value");
  assert.deepEqual(location(outer), [{ source: 0, revision: revision(text), location: span(text, "value", false) }]);
  const innerPosition = Buffer.byteLength(text.slice(0, text.indexOf("return value") + 7));
  const inner = navigateCode(request([saved(text)], 0, innerPosition));
  assert.deepEqual(location(inner)[0].location, { start_byte: text.indexOf("f(value") + 2, end_byte: text.indexOf("f(value") + 7 });
  const refs = location(navigate([text], 0, "value", { operation: "references" }));
  assert.equal(refs.length, 2); assert.deepEqual(refs.map((item) => item.location), [span(text, "value", false), span(text, "value")]);
  assert.deepEqual(refs.map((item) => item.is_write), [true, false]);
});
test("selected scripts share globals while same-spelled external modules remain isolated", () => {
  const sources = ["const globalValue = 1;", "globalValue;", "export const globalValue = 3; globalValue;"];
  assert.deepEqual(location(navigate(sources, 1, "globalValue"))[0], { source: 0, revision: revision(sources[0]), location: span(sources[0], "globalValue") });
  const refs = location(navigate(sources, 1, "globalValue", { operation: "references" }));
  assert.deepEqual(refs.map((item) => item.source), [0, 1]);
  assert.deepEqual(location(navigate(sources, 2, "globalValue"))[0].source, 2);
});
test("JS TS JSX and TSX preserve exact BOM CRLF Unicode and astral-prefix byte spans", () => {
  for (const [language, body, target] of [["js", "function çay(x) { return x; } çay(1);", "çay"],
    ["ts", "const çay: number = 1; çay;", "çay"],
    ["jsx", "const Panel = () => <div/>; const view = <Panel/>;", "Panel"],
    ["tsx", "const Panel = (p: {x: number}) => <div>{p.x}</div>; const view = <Panel x={1}/>;", "Panel"]]) {
    const text = "\ufeff// 😀\r\n" + body + "\r\n";
    const result = navigate([saved(text, language)], 0, target);
    assert.deepEqual(result.backend, { name: "typescript-language-service", version: "5.9.3" });
    assert.equal(result.model_complete, false);
    assert.deepEqual(result.sources, [{ source: 0, revision: revision(text), source_bytes: Buffer.byteLength(text) }]);
    assert.deepEqual(location(result)[0].location, span(text, target, false));
    assert.equal(Buffer.from(text).subarray(location(result)[0].location.start_byte, location(result)[0].location.end_byte).toString(), target);
  }
});
test("renamed shadowed and private symbols retain scope identity", () => {
  const text = "class A { #value = 1; method() { return this.#value; } } class B { #value = 2; method() { return this.#value; } }";
  const refs = location(navigate([text], 0, "#value", { operation: "references" }));
  assert.equal(refs.length, 2); assert.ok(refs.every((item) => item.location.start_byte > text.indexOf("class B")));
  const escaped = "const caf\\u00e9 = 1; café;";
  assert.deepEqual(location(navigate([escaped], 0, "café"))[0].location, span(escaped, "caf\\u00e9"));
});
test("distinct object and interface properties do not collapse into same-spelled references", () => {
  const text = "interface A { x: number }; interface B { x: number }; const a: A = {x:1}; const b: B = {x:2}; a.x; b.x;";
  const result = navigateCode(request([saved(text)], 0, text.indexOf("a.x") + 2, { operation: "references" }));
  assert.deepEqual(location(result).map((item) => item.location.start_byte), [text.indexOf("x:"), text.indexOf("{x:1}") + 1, text.indexOf("a.x") + 2]);
  const any = "const a = {x:1}; const b: any = {}; a.x; b.x;";
  const anyResult = navigateCode(request([saved(any)], 0, any.indexOf("a.x") + 2, { operation: "references" }));
  assert.deepEqual(location(anyResult).map((item) => item.location.start_byte), [any.indexOf("x:"), any.indexOf("a.x") + 2]);
});
test("eight selected snapshots are the complete explicit model without declaration or library discovery", () => {
  const sources = ["const shared = 1;", ...Array.from({ length: 7 }, (_, i) => `const value${i} = shared;`)];
  assert.deepEqual(location(navigate(sources, 7, "shared", { operation: "references" })).map((item) => item.source), [0, 1, 2, 3, 4, 5, 6, 7]);
  const hostile = '/// <reference path="/never-read/types.d.ts" />\n/// <reference types="hostile" />\n/// <reference lib="dom" />\nconst value = 1; value;';
  assert.equal(navigate([hostile], 0, "value").reason, "unsupported_reference_directive");
});
test("every import requires an explicit binding even when a matching selected source is present", () => {
  const sources = ["export const imported = 1;", 'import { imported } from "./source-0"; imported;'];
  for (const operation of ["definitions", "references"]) {
    const result = navigate(sources, 1, "imported", { operation });
    assert.equal(result.status, "refused"); assert.equal(result.reason, "missing_import_binding"); assert.equal(result.locations, undefined);
  }
  const renamed = 'import { imported as alias } from "hostile-package"; alias;';
  assert.equal(navigate([renamed], 0, "alias").reason, "missing_import_binding");
});
test("unresolved names cannot fall back to spelling-based reference matches", () => {
  const text = "missing; function f() { missing; }";
  const result = navigate([text], 0, "missing", { operation: "references" });
  assert.equal(result.status, "refused"); assert.equal(result.reason, "unresolved_symbol"); assert.equal(result.locations, undefined);
});
test("every selected revision binds the model and changes discard all locations", () => {
  const first = saved("const value = 1; value;"), second = saved("const unrelated = 1;");
  second.source_utf8 = "const unrelated = 2;";
  const changed = navigateCode(request([first, second], 0, byte(first.source_utf8, "value")));
  assert.equal(changed.status, "changed"); assert.equal(changed.locations, undefined);
  const next = "const renamed = 1; renamed;";
  const result = navigate([next], 0, "renamed");
  assert.deepEqual(location(result)[0].location, span(next, "renamed", false));
});
test("byte interiors EOF punctuation source errors and malformed UTF-16 refuse", () => {
  const text = "// 😀\r\nconst çay = 1; çay;", source = saved(text);
  for (const position of [Buffer.byteLength(text), byte(text, "çay") + 1, byte(text, "😀") + 1]) {
    assert.equal(navigateCode(request([source], 0, position)).reason, "invalid_position");
  }
  assert.equal(navigateCode(request([source], 0, byte(text, "="))).reason, "identifier_required");
  assert.equal(navigate(["const value = ; value;"], 0, "value").reason, "syntax_errors");
  assert.equal(navigateCode(request([saved("const value = '\ud800'; value;")], 0, 6)).reason, "invalid_utf8");
  assert.equal(navigateCode(request([saved("const value = '\0'; value;")], 0, 6)).reason, "nul_source");
});
test("finite source AST location and response bounds report honest query truncation", () => {
  const text = "const value = 1;" + "value;".repeat(120);
  const result = navigate([text], 0, "value", { operation: "references", limit: 3 });
  assert.equal(location(result).length, 3); assert.equal(result.complete, false);
  assert.equal(navigateCode(request(Array.from({ length: 9 }, () => saved("let x;")), 0, 4)).reason, "source_kind_limit");
  assert.equal(navigateCode(request([saved(" ".repeat(limits.source_bytes + 1))], 0, 0)).reason, "source_limit");
  assert.equal(navigateCode(request(Array.from({ length: 5 }, () => saved(" ".repeat(limits.source_bytes))), 0, 0)).reason, "source_limit");
  assert.equal(navigateCode(request([saved(";".repeat(limits.nodes))], 0, 0)).reason, "node_limit");
  assert.equal(navigate(["const value = " + "(".repeat(80) + "1" + ")".repeat(80) + "; value;"], 0, "value").reason, "depth_limit");
  assert.ok(Buffer.byteLength(JSON.stringify(result)) < limits.response_bytes);
});
test("unknown settings paths kinds and revisions cannot enter the passive host", () => {
  const valid = request([saved("const value = 1; value;")], 0, 17);
  for (const mutate of [(r) => { r.config = "/tsconfig.json"; }, (r) => { r.sources[0].path = "/project/source.ts"; },
    (r) => { r.sources[0].language = "typescript"; }, (r) => { r.sources[0].expected_revision = "latest"; },
    (r) => { r.operation = "rename"; }]) {
    const copy = structuredClone(valid); mutate(copy); assert.equal(navigateCode(copy).status, "refused");
  }
});
test("qualified service calls no filesystem network process or TypeScript system API after trusted import", () => {
  const calls = [], restore = [];
  const replace = (object, key, name) => {
    const descriptor = Object.getOwnPropertyDescriptor(object, key);
    if (!descriptor || !descriptor.writable || typeof object[key] !== "function") return;
    restore.push(() => Object.defineProperty(object, key, descriptor));
    object[key] = () => { calls.push(name); throw new Error(`passive sentinel ${name}`); };
  };
  for (const [name, object] of [["fs", fs], ["fs.promises", fs.promises], ["child_process", childProcess], ["dns", dns], ["ts.sys", ts.sys]]) {
    for (const key of Object.keys(object)) replace(object, key, `${name}.${key}`);
  }
  replace(net.Socket.prototype, "connect", "socket.connect"); replace(process, "cwd", "process.cwd"); replace(process, "chdir", "process.chdir");
  for (const [name, object] of [["net", net], ["http", http], ["https", https]]) {
    for (const key of ["connect", "createConnection", "request", "get", "createServer"]) replace(object, key, `${name}.${key}`);
  }
  try {
    for (const language of ["js", "ts", "jsx", "tsx"]) {
      const text = language.endsWith("x") ? "const Panel = () => <div/>; const view = <Panel/>;" : "const value = 1; value;";
      assert.equal(navigate([saved(text, language)], 0, language.endsWith("x") ? "Panel" : "value", { operation: "references" }).status, "locations");
    }
    assert.equal(navigate(['import {x} from "hostile-package"; x;'], 0, "x").reason, "missing_import_binding");
    assert.deepEqual(calls, []);
  } finally { for (const undo of restore.reverse()) undo(); }
});
