/** Passive syntax structure over supplied original bytes; no project/model IO. */
import { createHash } from "node:crypto";
import ts from "typescript";
export interface CodeSourceSpan { start_byte: number; end_byte: number }
export interface CodeSourceNode {
  pointer: string;
  kind: string;
  name?: string;
  location: CodeSourceSpan | null;
  location_reason?: string;
  name_location?: CodeSourceSpan;
  value_location?: CodeSourceSpan;
  child_count: number;
}
export interface CodeSourceIssue { reason: string; location: CodeSourceSpan | null }
export interface CodeSourceInspection {
  status: "page" | "changed" | "missing_pointer" | "invalid_start" | "refused" | "budget_blocked";
  backend: { name: string; version: string };
  revision: string | null;
  source_bytes: number;
  /** True only when the finite structural model contains no recovery/unparsed nodes. */
  structure_complete: boolean;
  coverage: string[];
  issues: CodeSourceIssue[];
  issues_truncated?: boolean;
  reason?: string;
  selected?: CodeSourceNode;
  children?: CodeSourceNode[];
  child_count?: number;
  start_child?: number;
  /** Pagination completion, distinct from structure_complete. */
  complete?: boolean;
  continuation?: { pointer: string; expected_revision: string; start_child: number; limit: number } | null;
  blocked_child?: { pointer: string; kind: string; location: CodeSourceSpan | null };
}


export const CODE_SOURCE_LIMITS = Object.freeze({ source_bytes: 1_048_576, depth: 64,
  nodes: 32_768, children: 100, response_bytes: 32_768, metadata_bytes: 4096 });
export interface CodeSourceInspectRequest {
  language: "js" | "ts" | "jsx" | "tsx";
  pointer?: string;
  expected_revision?: string | null;
  start_child?: number;
  limit?: number;
  max_response_bytes?: number;
}
interface ModelNode {
  kind: string; start: number; end: number; reason?: string; name?: string; children: ModelNode[];
}
const encoder = new TextEncoder();
const scriptKinds = Object.freeze({ js: ts.ScriptKind.JS, ts: ts.ScriptKind.TS,
  jsx: ts.ScriptKind.JSX, tsx: ts.ScriptKind.TSX });
// Numeric enum aliases such as FirstStatement must not hide the actual syntax kind.
const kindNames = new Map<number, string>();
for (const [name, kind] of Object.entries(ts.SyntaxKind)) {
  if (typeof kind === "number" && !/^(?:First|Last|Count)/.test(name)) kindNames.set(kind, name);
}
const coverage = ["TypeScript 5.9.3 createSourceFile/forEachChild syntax occurrences with a fixed explicit script kind",
  "node.pos..end includes leading trivia; comments and most punctuation are not separate children; EOF is included",
  "names are parser-decoded identifiers; recovery/missing/unproved spans are explicit; no validation of types or JS-only grammar",
  "single supplied source only; no symbols, definitions, references, imports/configuration resolution, plugins or execution"];
class Refusal extends Error { constructor(readonly reason: string) { super(reason); } }
function fail(reason: string): never { throw new Refusal(reason); }
function bytes(value: unknown): number { return encoder.encode(JSON.stringify(value)).length; }
function bounded(result: CodeSourceInspection, budget: number): CodeSourceInspection {
  while (bytes(result) > budget && result.issues.length) { result.issues.pop(); result.issues_truncated = true; }
  if (bytes(result) > budget) fail("response_budget");
  return result;
}
function offsets(text: string): Int32Array {
  const table = new Int32Array(text.length + 1).fill(-1);
  let index = 0, offset = 0;
  table[0] = 0;
  for (const character of text) {
    const code = character.codePointAt(0)!;
    offset += code <= 0x7f ? 1 : code <= 0x7ff ? 2 : code <= 0xffff ? 3 : 4;
    index += character.length; table[index] = offset;
  }
  return table;
}
function span(table: Int32Array, start: number, end: number): CodeSourceSpan | null {
  if (!Number.isInteger(start) || !Number.isInteger(end) || start < 0 || end < start || end >= table.length) return null;
  const a = table[start]!, b = table[end]!;
  return a < 0 || b < 0 ? null : { start_byte: a, end_byte: b };
}
function expose(node: ModelNode, pointer: string, table: Int32Array): CodeSourceNode {
  const location = node.reason ? null : span(table, node.start, node.end);
  const result: CodeSourceNode = { pointer, kind: node.kind, location, child_count: node.children.length };
  if (node.name !== undefined) result.name = node.name;
  if (!location) result.location_reason = node.reason ?? "unproved_original_offset";
  return result;
}
function model(text: string, language: CodeSourceInspectRequest["language"], table: Int32Array,
  result: CodeSourceInspection): ModelNode {
  // This constant logical name is not a path lookup. No compiler host, Program,
  // project configuration, module resolver, language service or emitter is created.
  const source = ts.createSourceFile(`source.${language}`, text, ts.ScriptTarget.ESNext, false, scriptKinds[language]);
  // parseDiagnostics is present on the qualified 5.9.3 parser result but omitted
  // from its public .d.ts. Fail closed if that exact passive result shape changes.
  const diagnostics = (source as ts.SourceFile & { parseDiagnostics?: readonly ts.DiagnosticWithLocation[] }).parseDiagnostics;
  if (!Array.isArray(diagnostics)) fail("parser_contract");
  result.structure_complete = diagnostics.length === 0;
  for (const diagnostic of diagnostics.slice(0, 16)) {
    result.issues.push({ reason: `typescript_parse_${diagnostic.code}`,
      location: span(table, diagnostic.start, diagnostic.start + diagnostic.length) });
  }
  if (diagnostics.length > 16) result.issues_truncated = true;
  let count = 1;
  const make = (node: ts.Node): ModelNode => {
    const kind = kindNames.get(node.kind) ?? "unknown";
    const result: ModelNode = { kind, start: node.pos, end: node.end, children: [] };
    if (node.flags & ts.NodeFlags.ThisNodeHasError) result.reason = "typescript_recovery";
    else if (node.kind === ts.SyntaxKind.MissingDeclaration ||
      node.pos === node.end && node.kind === ts.SyntaxKind.Identifier) result.reason = "missing_syntax";
    else if (!span(table, node.pos, node.end)) result.reason = "unproved_original_offset";
    else if (kind === "unknown") result.reason = "unknown_syntax";
    if (node.kind === ts.SyntaxKind.Identifier) {
      const name = String((node as ts.Identifier).escapedText);
      // TypeScript internally escapes identifiers starting with two underscores.
      result.name = name.startsWith("___") ? name.slice(1) : name;
    }
    return result;
  };
  const root = make(source), pending = [{ source, node: root, depth: 0 } as { source: ts.Node; node: ModelNode; depth: number }];
  while (pending.length) {
    const item = pending.pop()!;
    if (item.node.reason) result.structure_complete = false;
    ts.forEachChild(item.source, (child) => {
      if (++count > CODE_SOURCE_LIMITS.nodes) fail("node_limit");
      if (item.depth + 1 > CODE_SOURCE_LIMITS.depth) fail("depth_limit");
      const node = make(child);
      item.node.children.push(node); pending.push({ source: child, node, depth: item.depth + 1 });
    });
  }
  return root;
}

/** Returns one bounded immediate-child page; caller owns saved-file scope/rechecks. */
export function inspectCodeSource(source: Uint8Array, request: CodeSourceInspectRequest): CodeSourceInspection {
  const result: CodeSourceInspection = { status: "refused", backend: { name: "typescript", version: "5.9.3" },
    revision: null, source_bytes: source instanceof Uint8Array ? source.byteLength : 0,
    structure_complete: false, coverage: [...coverage], issues: [] };
  const pointer = request?.pointer ?? "", start = request?.start_child ?? 0, limit = request?.limit ?? 100;
  const budget = request?.max_response_bytes ?? CODE_SOURCE_LIMITS.response_bytes;
  try {
    if (!request || typeof request.language !== "string" || !Object.hasOwn(scriptKinds, request.language) || !(source instanceof Uint8Array) ||
        typeof pointer !== "string" || pointer.length > CODE_SOURCE_LIMITS.metadata_bytes || !/^(?:\/(?:0|[1-9][0-9]*))*$/.test(pointer) ||
        !Number.isInteger(start) || start < 0 || start > CODE_SOURCE_LIMITS.nodes ||
        !Number.isInteger(limit) || limit < 1 || limit > CODE_SOURCE_LIMITS.children ||
        !Number.isInteger(budget) || budget < 1024 || budget > CODE_SOURCE_LIMITS.response_bytes ||
        (request.expected_revision != null && (typeof request.expected_revision !== "string" || !/^sha256:[0-9a-f]{64}$/.test(request.expected_revision))) ||
        ((pointer !== "" || start !== 0) && request.expected_revision == null) ||
        Object.keys(request).some((key) => !["language", "pointer", "expected_revision", "start_child", "limit", "max_response_bytes"].includes(key))) fail("invalid_request");
    if (encoder.encode(pointer).length + bytes(result) + 256 > budget) fail("request_metadata_exceeds_budget");
    if (source.byteLength > CODE_SOURCE_LIMITS.source_bytes) fail("source_too_large");
    const snapshot = new Uint8Array(source);
    result.revision = "sha256:" + createHash("sha256").update(snapshot).digest("hex");
    if (request.expected_revision != null && request.expected_revision !== result.revision) { result.status = "changed"; return bounded(result, budget); }
    let text: string;
    try { text = new TextDecoder("utf-8", { fatal: true, ignoreBOM: true }).decode(snapshot); }
    catch { fail("invalid_utf8"); }
    if (text.includes("\0")) fail("nul_source");
    const table = offsets(text), root = model(text, request.language, table, result);
    let selected = root;
    for (const ordinal of pointer.split("/").slice(1)) {
      const index = Number(ordinal);
      if (!Number.isSafeInteger(index) || !selected.children[index]) { result.status = "missing_pointer"; return bounded(result, budget); }
      selected = selected.children[index]!;
    }
    result.selected = expose(selected, pointer, table); result.child_count = selected.children.length; result.start_child = start;
    if (start > selected.children.length) { result.status = "invalid_start"; return bounded(result, budget); }
    result.status = "page"; result.children = []; result.complete = false; result.continuation = null;
    if (bytes(result.selected) > CODE_SOURCE_LIMITS.metadata_bytes || bytes(result) > budget) {
      delete result.selected; delete result.children; delete result.complete; delete result.continuation;
      result.status = "budget_blocked"; result.reason = "selected_metadata_exceeds_budget";
      result.blocked_child = { pointer, kind: selected.kind, location: selected.reason ? null : span(table, selected.start, selected.end) };
      return bounded(result, budget);
    }
    let index = start;
    for (; index < selected.children.length && index - start < limit; index++) {
      const child = expose(selected.children[index]!, `${pointer}/${index}`, table);
      result.children.push(child); result.complete = index + 1 === selected.children.length;
      result.continuation = result.complete ? null : { pointer, expected_revision: result.revision, start_child: index + 1, limit };
      if (bytes(child) > CODE_SOURCE_LIMITS.metadata_bytes || bytes(result) > budget) {
        result.children.pop();
        if (result.children.length === 0) {
          result.status = "budget_blocked"; result.reason = "child_metadata_exceeds_budget";
          result.blocked_child = { pointer: child.pointer, kind: child.kind, location: child.location };
          result.complete = false; result.continuation = null;
          if (bytes(result) > budget) { delete result.selected; result.issues = []; }
          return bounded(result, budget);
        }
        break;
      }
    }
    result.complete = index === selected.children.length;
    result.continuation = result.complete ? null : { pointer, expected_revision: result.revision, start_child: start + result.children.length, limit };
    return bounded(result, budget);
  } catch (error: unknown) {
    return { status: "refused", backend: result.backend, revision: result.revision, source_bytes: result.source_bytes,
      structure_complete: false, coverage: result.coverage, issues: [], reason: error instanceof Refusal ? error.reason : "parser_failure" };
  }
}
