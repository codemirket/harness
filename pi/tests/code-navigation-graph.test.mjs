import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import childProcess from "node:child_process";
import ts from "typescript";
import test from "node:test";
import { navigateCode, CODE_NAVIGATION_LIMITS as limits } from "../dist/runtime/code-navigation.js";

const revision = text => "sha256:" + createHash("sha256").update(text).digest("hex");
const saved = (source_utf8, language = "ts") => ({ source_utf8, language, expected_revision: revision(source_utf8) });
const sources = [
  saved('const emoji = "😀"; export function selam(kişi: string) { return kişi.length; }\r\nexport const twice = (x: number) => x * 2;\r\nexport const local = selam("İzmir");\r\n'),
  saved('import { selam, twice } from "./a";\r\nimport { prepared } from "prepared";\r\nexport const total = selam("😀İ") + twice(prepared(2));\r\nexport const globalUse: FixtureGlobal = { value: 1 };\r\n'),
  saved('import { selam } from "./a";\r\nexport function Widget() { return <div>{selam("İ")}</div>; }\r\nexport const view = <Widget />;\r\n', "tsx"),
  saved('export function square(x) { return x * x; }\r\n', "js"),
  saved('import { square } from "./logic.js";\r\nexport function View() { return <span>{square(3)}</span>; }\r\nexport const view = <View />;\r\n', "jsx"),
  saved('declare namespace JSX { interface IntrinsicElements { div: any; span: any; } }\r\n', "ts_declaration"),
  saved('export declare function prepared(x: number): number;\r\n', "ts_declaration"),
  saved('interface FixtureGlobal { value: number }\r\n', "ts_declaration"),
  saved('export const unrelated = 1;\r\n'), saved('export const another = 2;\r\n', "js"),
];
const imports = [{ source: 1, specifier: "./a", target: 0 }, { source: 2, specifier: "./a", target: 0 },
  { source: 4, specifier: "./logic.js", target: 3 }, { source: 1, specifier: "prepared", target: 6 }];
const position = (text, needle, occurrence = 0) => {
  let index = -1; for (let i = 0; i <= occurrence; i++) index = text.indexOf(needle, index + 1);
  assert.ok(index >= 0); return Buffer.byteLength(text.slice(0, index));
};
const request = (source = 1, needle = "selam", occurrence = 1, operation = "definitions") => ({
  sources: structuredClone(sources), typescript_model: { imports: structuredClone(imports) }, source,
  position_byte: position(sources[source].source_utf8, needle, occurrence), operation, limit: 100,
  max_response_bytes: limits.response_bytes,
});
const checked = (value, needle) => {
  assert.equal(value.status, "locations", JSON.stringify(value));
  assert.equal(value.model_complete, false); assert.match(value.model_revision, /^sha256:[0-9a-f]{64}$/);
  for (const item of value.locations) {
    assert.equal(item.revision, sources[item.source].expected_revision);
    assert.equal(Buffer.from(sources[item.source].source_utf8).subarray(item.location.start_byte, item.location.end_byte).toString(), needle);
  }
  return value.locations;
};
test("selected graph resolves the finite TS/JS/JSX/TSX cross-file and prepared-declaration corpus", () => {
  for (const [source, needle, occurrence, target] of [[0, "selam", 1, 0], [1, "selam", 1, 0], [2, "selam", 1, 0],
    [2, "Widget", 1, 2], [4, "square", 1, 3], [4, "View", 1, 4], [1, "prepared", 2, 6], [1, "FixtureGlobal", 0, 7]]) {
    assert.ok(checked(navigateCode(request(source, needle, occurrence)), needle).some(item => item.source === target));
  }
  for (const [source, needle, files, count] of [[0, "selam", [0, 1, 2], 6], [3, "square", [3, 4], 3]]) {
    const items = checked(navigateCode(request(source, needle, 0, "references")), needle);
    assert.deepEqual([...new Set(items.map(item => item.source))], files); assert.equal(items.length, count);
  }
});
test("the whole model rejects missing duplicate unused and unselected import bindings", () => {
  for (const [change, reason] of [
    [r => { r.typescript_model.imports.pop(); }, "missing_import_binding"],
    [r => { r.typescript_model.imports.push({ ...r.typescript_model.imports[0] }); }, "ambiguous_import_binding"],
    [r => { r.typescript_model.imports.push({ source: 0, specifier: "unused", target: 1 }); }, "unused_import_binding"],
    [r => { r.typescript_model.imports[0].target = 100; }, "invalid_import_binding"],
    [r => { r.typescript_model.imports[0].specifier = "wrong"; }, "missing_import_binding"],
  ]) {
    const value = request(0, "selam", 0); change(value); const result = navigateCode(value);
    assert.equal(result.status, "refused"); assert.equal(result.reason, reason); assert.equal(result.locations, undefined);
  }
  const wrongExport = request(0, "selam", 0); wrongExport.typescript_model.imports[0].target = 6;
  assert.equal(navigateCode(wrongExport).reason, "unresolved_import", "an unrelated query must not hide a broken imported export");
});
test("default namespace renamed imports and named re-exports use only explicit transitive edges", () => {
  const texts = [saved("export default function café() {} export const value = 1;"),
    saved("export {value as renamed} from 'first';"),
    saved("import café from 'first'; import * as api from 'second'; café(); api.renamed;")];
  const input = { sources: texts, typescript_model: { imports: [
    { source: 1, specifier: "first", target: 0 }, { source: 2, specifier: "first", target: 0 }, { source: 2, specifier: "second", target: 1 },
  ] }, source: 2, position_byte: position(texts[2].source_utf8, "café", 1), operation: "definitions", limit: 100, max_response_bytes: limits.response_bytes };
  const value = navigateCode(input); assert.equal(value.status, "locations", JSON.stringify(value)); assert.equal(value.locations[0].source, 0);
  input.position_byte = position(texts[2].source_utf8, "renamed");
  const renamed = navigateCode(input); assert.equal(renamed.status, "locations", JSON.stringify(renamed)); assert.equal(renamed.locations[0].source, 0);
});
test("graph and every declaration revision are bound while default-library members stay incomplete", () => {
  const input = request(), before = navigateCode(input); checked(before, "selam");
  input.sources[7].source_utf8 += "// changed unrelated declaration";
  assert.equal(navigateCode(input).status, "changed");
  const reordered = request(); reordered.typescript_model.imports.reverse();
  assert.notEqual(navigateCode(reordered).model_revision, before.model_revision);
  const length = navigateCode(request(0, "length", 0)); assert.equal(length.status, "refused"); assert.equal(length.reason, "unresolved_symbol");
});
test("unsupported discovery and import forms refuse before exposing any locations", () => {
  for (const text of ["/// <reference types='other'/>\nconst value = 1; value;", "const value = import('other'); value;",
    "import value = require('other'); value;", "type T = import('other').T; const value = 1; value;",
    "const value = require('other'); value;", "declare module 'other' { export const value: number; } const value = 1; value;"]) {
    const input = request(0, "selam"); input.sources = [saved(text)]; input.typescript_model.imports = [];
    input.source = 0; input.position_byte = position(text, "value");
    const value = navigateCode(input); assert.equal(value.status, "refused"); assert.equal(value.locations, undefined);
    assert.ok(["unsupported_reference_directive", "unsupported_import_form"].includes(value.reason), JSON.stringify(value));
  }
});
test("declaration and graph bounds remain finite and cannot enable a legacy or filesystem host", () => {
  const valid = request();
  for (const change of [r => { delete r.typescript_model; }, r => { r.typescript_model.path = "/project/model.json"; },
    r => { r.typescript_model.imports[0].specifier = "x".repeat(257); }, r => { r.typescript_model.imports[0].specifier = "\ud800"; },
    r => { r.typescript_model.imports = Array.from({ length: 129 }, () => imports[0]); },
    r => { r.sources = [saved("const x=1;"), ...Array.from({ length: 9 }, () => saved("interface X {}", "ts_declaration"))]; },
    r => { r.sources[6].source_utf8 = " ".repeat(limits.source_bytes + 1); r.sources[6].expected_revision = revision(r.sources[6].source_utf8); }]) {
    const input = structuredClone(valid); change(input); assert.equal(navigateCode(input).status, "refused");
  }
  assert.equal(limits.total_bytes, 1_048_576); assert.equal(limits.nodes, 32_768); assert.equal(limits.response_bytes, 32_768);
});
test("prepared imports call no filesystem process or TypeScript system API", () => {
  const calls = [], restore = [];
  for (const [name, object] of [["fs", fs], ["promises", fs.promises], ["process", childProcess], ["ts.sys", ts.sys]]) {
    for (const key of Object.keys(object)) {
      const descriptor = Object.getOwnPropertyDescriptor(object, key);
      if (!descriptor?.writable || typeof object[key] !== "function") continue;
      restore.push(() => Object.defineProperty(object, key, descriptor));
      object[key] = () => { calls.push(`${name}.${key}`); throw Error("forbidden IO"); };
    }
  }
  try { checked(navigateCode(request()), "selam"); checked(navigateCode(request(3, "square", 0, "references")), "square"); assert.deepEqual(calls, []); }
  finally { for (const undo of restore.reverse()) undo(); }
});
