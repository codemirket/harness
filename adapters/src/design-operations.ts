import StyleDictionary from "style-dictionary";
import { optimize } from "svgo";
import sax from "sax";
import type { SAXParser, Tag } from "sax";

/** Inputs are deliberately small: these functions are leaf operations, not a project loader. */
export const MAX_INPUT_BYTES = 1024 * 1024;
export const MAX_OUTPUT_BYTES = 16 * 1024 * 1024;
export const MAX_TOKEN_COUNT = 4096;
export const MAX_OBJECT_MEMBERS = 8192;
export const MAX_TOKEN_DEPTH = 32;
export const MAX_SVG_ELEMENTS = 4096;
export const MAX_SVG_ATTRIBUTES = 128;

const encoder = new TextEncoder();
const decoder = new TextDecoder("utf-8", { fatal: true, ignoreBOM: true });
const tokenName = /^[A-Za-z0-9_-]{1,128}$/u;
const aliasValue = /^\{([A-Za-z0-9_-]+(?:\.[A-Za-z0-9_-]+)*)\}$/u;
const forbiddenKey = /^(?:__proto__|prototype|constructor)$/u;

export class DesignOperationError extends Error {
  readonly name = "DesignOperationError";

  constructor(readonly code: DesignOperationErrorCode, message: string) {
    super(message);
  }
}

export type DesignOperationErrorCode =
  | "input_too_large"
  | "invalid_utf8"
  | "invalid_tokens"
  | "token_bounds"
  | "alias_invalid"
  | "alias_unresolved"
  | "alias_cycle"
  | "token_format"
  | "compile_failed"
  | "svg_rejected"
  | "svg_invalid"
  | "output_limit";

export interface CompileResult {
  output: Uint8Array;
  format: "css" | "json";
  input_bytes: number;
  output_bytes: number;
  token_count: number;
  max_depth: number;
}

export interface OptimizeResult {
  output: Uint8Array;
  format: "svg";
  input_bytes: number;
  output_bytes: number;
  element_count: number;
}

function fail(code: DesignOperationErrorCode, message: string): never {
  throw new DesignOperationError(code, message);
}

function decodeInput(input: Uint8Array): string {
  if (!(input instanceof Uint8Array)) fail("invalid_utf8", "Input must be a byte array.");
  if (input.byteLength > MAX_INPUT_BYTES) fail("input_too_large", "Input exceeds the 1 MiB limit.");
  try {
    return decoder.decode(input);
  } catch {
    fail("invalid_utf8", "Input must be valid UTF-8.");
  }
}

type JsonRecord = Record<string, unknown>;
type TokenRecord = JsonRecord & { value?: unknown; $value?: unknown };

interface TokenInventory {
  readonly tokens: number;
  readonly members: number;
  readonly maxDepth: number;
  readonly values: Map<string, unknown>;
}

function own(record: JsonRecord, key: string): boolean {
  return Object.prototype.hasOwnProperty.call(record, key);
}

function validObject(value: unknown): value is JsonRecord {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

function validateTokenValue(value: unknown, path: string, allowAlias = true): void {
  if (value === null || typeof value === "function" || typeof value === "undefined") {
    fail("token_format", `Token ${path} has an unsupported value.`);
  }
  if (typeof value === "number" && !Number.isFinite(value)) {
    fail("token_format", `Token ${path} has a non-finite number.`);
  }
  if (typeof value === "string") {
    if (value.length > 16_384 || value.includes("\0")) {
      fail("token_bounds", `Token ${path} contains an oversized or NUL string.`);
    }
    // References are intentionally exact. Composite strings and external paths are not accepted.
    if (value.includes("{") || value.includes("}")) {
      if (!allowAlias || !aliasValue.test(value)) fail("alias_invalid", `Token ${path} has a malformed alias.`);
    }
  }
  if (Array.isArray(value)) {
    if (value.length > 256) fail("token_bounds", `Token ${path} has too many value members.`);
    for (const item of value) validateTokenValue(item, path, false);
  }
  if (validObject(value)) {
    let members = 0;
    for (const [key, item] of Object.entries(value)) {
      members += 1;
      if (members > 256 || forbiddenKey.test(key)) fail("token_bounds", `Token ${path} has unsafe value members.`);
      validateTokenValue(item, `${path}.${key}`, false);
    }
  }
}

function inventoryTokens(root: unknown): TokenInventory {
  if (!validObject(root)) fail("invalid_tokens", "Token input must be a JSON object.");
  const values = new Map<string, unknown>();
  let tokens = 0;
  let members = 0;
  let maxDepth = 0;

  const walk = (node: JsonRecord, path: string[], depth: number): void => {
    if (depth > MAX_TOKEN_DEPTH) fail("token_bounds", "Token nesting exceeds the depth limit.");
    maxDepth = Math.max(maxDepth, depth);
    for (const [key, child] of Object.entries(node)) {
      members += 1;
      if (members > MAX_OBJECT_MEMBERS || forbiddenKey.test(key)) {
        fail("token_bounds", "Token object member limit exceeded.");
      }
      if (key !== "$type" && key !== "$description" && !tokenName.test(key)) {
        fail("invalid_tokens", `Token name ${key} is not a bounded identifier.`);
      }
      if (key === "$type" || key === "$description") {
        if (typeof child !== "string" || child.length > 256) {
          fail("invalid_tokens", `Token metadata ${key} is invalid.`);
        }
        continue;
      }
      if (!validObject(child)) fail("invalid_tokens", `Token node ${[...path, key].join(".")} is not an object.`);
      const record = child as TokenRecord;
      const hasValue = own(record, "value");
      const hasDtcgValue = own(record, "$value");
      if (hasValue && hasDtcgValue) fail("invalid_tokens", `Token ${[...path, key].join(".")} has two value fields.`);
      if (hasValue || hasDtcgValue) {
        const tokenPath = [...path, key].join(".");
        tokens += 1;
        if (tokens > MAX_TOKEN_COUNT) fail("token_bounds", "Token count limit exceeded.");
        const value = hasValue ? record.value : record.$value;
        validateTokenValue(value, tokenPath, true);
        values.set(tokenPath, value);
        for (const metadata of Object.keys(record)) {
          if (metadata !== "value" && metadata !== "$value" && metadata !== "type" && metadata !== "$type" && metadata !== "$description" && metadata !== "name" && metadata !== "comment" && metadata !== "themeable") {
            fail("invalid_tokens", `Token ${tokenPath} has unsupported metadata.`);
          }
        }
        continue;
      }
      walk(record, [...path, key], depth + 1);
    }
  };
  walk(root, [], 0);
  if (tokens === 0) fail("invalid_tokens", "Token input must contain at least one token.");
  return { tokens, members, maxDepth, values };
}

function checkAliases(inventory: TokenInventory): void {
  const resolving = new Set<string>();
  const resolved = new Set<string>();
  const resolve = (path: string): void => {
    if (resolved.has(path)) return;
    if (resolving.has(path)) fail("alias_cycle", `Token alias cycle includes ${path}.`);
    resolving.add(path);
    const value = inventory.values.get(path);
    if (typeof value === "string") {
      const match = aliasValue.exec(value);
      if (value.includes("{") || value.includes("}")) {
        if (!match) fail("alias_invalid", `Token ${path} has a malformed alias.`);
        const target = match[1];
        if (!target || !inventory.values.has(target)) fail("alias_unresolved", `Token ${path} refers to a missing alias.`);
        resolve(target);
      }
    }
    resolving.delete(path);
    resolved.add(path);
  };
  for (const path of inventory.values.keys()) resolve(path);
}

/** JSON.parse keeps the last duplicate key. Rejecting duplicates preserves source identity. */
function rejectDuplicateJsonKeys(source: string): void {
  let index = 0;
  const whitespace = (): void => {
    while (index < source.length && /\s/u.test(source[index] ?? "")) index += 1;
  };
  const string = (): string => {
    const start = index;
    if (source[index] !== '"') throw new Error("string");
    index += 1;
    while (index < source.length) {
      const char = source[index];
      if (char === "\\") {
        index += 2;
        continue;
      }
      index += 1;
      if (char === '"') {
        const raw = source.slice(start, index);
        const value = JSON.parse(raw) as unknown;
        if (typeof value !== "string") throw new Error("string");
        return value;
      }
      if (char !== undefined && char < " ") throw new Error("string");
    }
    throw new Error("string");
  };
  const value = (depth: number): void => {
    if (depth > MAX_TOKEN_DEPTH + 2) throw new Error("depth");
    whitespace();
    const char = source[index];
    if (char === "{") {
      index += 1;
      whitespace();
      const keys = new Set<string>();
      if (source[index] === "}") {
        index += 1;
        return;
      }
      while (index < source.length) {
        const key = string();
        if (keys.has(key)) fail("invalid_tokens", `Duplicate token object key: ${key.slice(0, 128)}.`);
        keys.add(key);
        whitespace();
        if (source[index] !== ":") throw new Error("colon");
        index += 1;
        value(depth + 1);
        whitespace();
        if (source[index] === "}") {
          index += 1;
          return;
        }
        if (source[index] !== ",") throw new Error("comma");
        index += 1;
        whitespace();
      }
      throw new Error("object");
    }
    if (char === "[") {
      index += 1;
      whitespace();
      if (source[index] === "]") {
        index += 1;
        return;
      }
      while (index < source.length) {
        value(depth + 1);
        whitespace();
        if (source[index] === "]") {
          index += 1;
          return;
        }
        if (source[index] !== ",") throw new Error("comma");
        index += 1;
        whitespace();
      }
      throw new Error("array");
    }
    if (char === '"') {
      string();
      return;
    }
    const start = index;
    while (index < source.length && !/[\s,\]}]/u.test(source[index] ?? "")) index += 1;
    if (start === index) throw new Error("value");
  };
  try {
    value(0);
    whitespace();
    if (index !== source.length) throw new Error("trailing");
  } catch (error) {
    if (error instanceof DesignOperationError) throw error;
    // JSON.parse below remains the authoritative syntax validator.
  }
}

function bytes(value: unknown): Uint8Array {
  if (typeof value !== "string") fail("compile_failed", "Design formatter returned non-text output.");
  const output = encoder.encode(value);
  if (output.byteLength > MAX_OUTPUT_BYTES) fail("output_limit", "Generated output exceeds the 16 MiB limit.");
  return output;
}

/** Compile a self-contained DTCG/classic token object without reading files or config. */
export async function compileTokens(input: Uint8Array, format: "css" | "json"): Promise<CompileResult> {
  if (format !== "css" && format !== "json") fail("compile_failed", "Token format is unsupported.");
  const source = decodeInput(input);
  let parsed: unknown;
  rejectDuplicateJsonKeys(source);
  try {
    parsed = JSON.parse(source) as unknown;
  } catch {
    fail("invalid_tokens", "Token input must be valid JSON.");
  }
  const inventory = inventoryTokens(parsed);
  checkAliases(inventory);

  const platform = format === "css"
    ? { transformGroup: "css", files: [{ destination: "tokens.css", format: "css/variables" }] }
    : { files: [{ destination: "tokens.json", format: "json/nested" }] };
  try {
    const dictionary = new StyleDictionary({
      tokens: parsed as Record<string, never>,
      platforms: { output: platform },
      log: { verbosity: "silent" },
    }, { verbosity: "silent" });
    const result = await dictionary.formatPlatform("output");
    const output = bytes(result[0]?.output);
    return {
      output,
      format,
      input_bytes: input.byteLength,
      output_bytes: output.byteLength,
      token_count: inventory.tokens,
      max_depth: inventory.maxDepth,
    };
  } catch (error) {
    if (error instanceof DesignOperationError) throw error;
    fail("compile_failed", error instanceof Error ? error.message.slice(0, 256) : "Token compilation failed.");
  }
}

const allowedSvgTags = new Set([
  "svg", "g", "path", "rect", "circle", "ellipse", "line", "polyline", "polygon",
  "defs", "linearGradient", "radialGradient", "stop", "clipPath", "mask", "pattern", "title", "desc",
  "style", "text", "marker",
]);
const allowedSvgAttributes = new Set([
  "xmlns", "version", "width", "height", "viewBox", "fill", "fill-opacity", "fill-rule", "stroke",
  "stroke-width", "stroke-linecap", "stroke-linejoin", "stroke-miterlimit", "stroke-opacity", "opacity",
  "color", "display", "visibility", "transform", "d", "x", "y", "x1", "x2", "y1", "y2", "cx", "cy",
  "r", "rx", "ry", "points", "offset", "stop-color", "stop-opacity", "gradientUnits", "gradientTransform",
  "spreadMethod", "clip-path", "clip-rule", "mask", "maskUnits", "patternUnits", "patternContentUnits",
  "patternTransform", "preserveAspectRatio", "id", "class", "style", "type", "href", "xlink:href",
  "marker-start", "marker-mid", "marker-end", "refX", "refY", "orient", "markerWidth", "markerHeight",
  "font-family", "font-size", "font-weight", "text-anchor", "dominant-baseline", "alignment-baseline",
  "letter-spacing", "word-spacing", "aria-label", "aria-labelledby", "aria-describedby", "role", "xml:space", "xml:lang",
]);

function svgFail(message: string): never {
  fail("svg_rejected", message);
}

function checkSvgResourceValue(value: string): void {
  if (value.length > 16_384 || value.includes("\\") || /(?:javascript:|data:|file:|https?:)/i.test(value)) {
    svgFail("SVG attribute contains a disallowed value.");
  }
  const urlPattern = /\burl\s*\(\s*(["']?)([^)"'\s]+)\1\s*\)/giu;
  let match: RegExpExecArray | null;
  while ((match = urlPattern.exec(value)) !== null) {
    if (!/^#[A-Za-z_][A-Za-z0-9_.:-]*$/u.test(match[2] ?? "")) svgFail("SVG URL must be an internal fragment.");
  }
  if (/\burl\s*\(/iu.test(value) && !urlPattern.test(value)) svgFail("SVG URL syntax is invalid.");
}

function checkSvgText(value: string): void {
  if (value.length > MAX_INPUT_BYTES || value.includes("\\") || /(?:javascript:|data:|file:|https?:|@import\b|@font-face\b|@namespace\b)/i.test(value)) {
    svgFail("SVG text contains a disallowed value.");
  }
  const urlPattern = /\burl\s*\(\s*(["']?)([^)"'\s]+)\1\s*\)/giu;
  let match: RegExpExecArray | null;
  while ((match = urlPattern.exec(value)) !== null) {
    if (!/^#[A-Za-z_][A-Za-z0-9_.:-]*$/u.test(match[2] ?? "")) svgFail("SVG URL must be an internal fragment.");
  }
  if (/\burl\s*\(/iu.test(value) && !urlPattern.test(value)) svgFail("SVG URL syntax is invalid.");
}

function validateSvg(source: string): number {
  if (source.includes("\0")) svgFail("SVG contains NUL bytes.");
  if (/<!DOCTYPE|<!ENTITY|<\s*(?:script|foreignObject|iframe|object|embed|image)\b/i.test(source)) {
    svgFail("SVG contains a disallowed declaration or element.");
  }
  if (/&(?!(?:amp|lt|gt|quot|apos);)[A-Za-z#][^;]{0,64};/u.test(source)) {
    svgFail("SVG entity references are not accepted.");
  }
  if (source.includes("\\") || /<\?xml-stylesheet\b/i.test(source)) svgFail("SVG contains an executable or escaped value.");
  const withoutNamespace = source.replace(/\bxmlns\s*=\s*(?:"[^"]*"|'[^']*')/giu, "");
  if (/(?:javascript:|data:|file:|https?:)/i.test(withoutNamespace)) svgFail("SVG contains an external resource.");
  const stack: string[] = [];
  const selfClosing = new Map<string, number>();
  let elements = 0;
  let attributes = 0;
  let rootSeen = false;
  let rootClosed = false;
  let styleDepth = 0;
  let pendingAttributes = new Set<string>();
  let pendingAttributeCount = 0;
  let parserError: Error | undefined;
  const xml: SAXParser = sax.parser(true, { trim: false, normalize: false, lowercase: false });
  xml.onopentagstart = () => {
    pendingAttributes = new Set<string>();
    pendingAttributeCount = 0;
  };
  xml.onattribute = (attribute) => {
    pendingAttributeCount += 1;
    if (pendingAttributeCount > MAX_SVG_ATTRIBUTES || pendingAttributes.has(attribute.name)) svgFail("SVG attributes exceed the bound or repeat.");
    pendingAttributes.add(attribute.name);
    if (attribute.name.startsWith("on") || !allowedSvgAttributes.has(attribute.name)) svgFail("SVG attribute is not allowed.");
    if (attribute.name === "xmlns" && attribute.value !== "http://www.w3.org/2000/svg") svgFail("SVG namespace is not allowed.");
    if (attribute.name === "xmlns") {
      // The sole admitted namespace URI is the SVG namespace itself.
    } else if (attribute.name === "href" || attribute.name === "xlink:href") {
      if (!/^#[A-Za-z_][A-Za-z0-9_.:-]*$/u.test(attribute.value)) svgFail("SVG href must be an internal fragment.");
    } else if (attribute.name === "aria-labelledby" || attribute.name === "aria-describedby") {
      // Accessibility references may name local elements only. Keep the
      // token form explicit so URL-like, whitespace/control and external
      // values cannot enter the closed SVG subset.
      const references = attribute.value.trim().split(/\s+/u);
      if (!references.length || references.some((reference) => !/^[A-Za-z_][A-Za-z0-9_.:-]*$/u.test(reference))) {
        svgFail("SVG accessibility references must name local IDs.");
      }
    } else {
      checkSvgResourceValue(attribute.value);
    }
  };
  xml.onopentag = (tag: Tag) => {
    if (rootClosed) svgFail("SVG has content after its root element.");
    if (!rootSeen && tag.name !== "svg") svgFail("SVG root element is missing.");
    if (tag.name === "svg") {
      if (rootSeen) svgFail("SVG has more than one root element.");
      rootSeen = true;
    }
    if (!allowedSvgTags.has(tag.name)) svgFail("SVG contains an unsupported element.");
    elements += 1;
    attributes += pendingAttributeCount;
    if (elements > MAX_SVG_ELEMENTS || attributes > MAX_SVG_ELEMENTS * MAX_SVG_ATTRIBUTES || stack.length >= 64) svgFail("SVG structure exceeds the bound.");
    if (tag.name === "style" && !tag.isSelfClosing) styleDepth += 1;
    if (tag.isSelfClosing) selfClosing.set(tag.name, (selfClosing.get(tag.name) ?? 0) + 1);
    else stack.push(tag.name);
  };
  xml.onclosetag = (name) => {
    const selfCount = selfClosing.get(name) ?? 0;
    if (selfCount > 0) {
      if (selfCount === 1) selfClosing.delete(name);
      else selfClosing.set(name, selfCount - 1);
      if (name === "svg") rootClosed = true;
      return;
    }
    if (stack.pop() !== name) svgFail("SVG element nesting is invalid.");
    if (name === "style") styleDepth = Math.max(0, styleDepth - 1);
    if (stack.length === 0) rootClosed = true;
  };
  xml.ontext = (text) => {
    if (text.trim() !== "" && (!rootSeen || rootClosed)) svgFail("SVG has text outside its root element.");
    checkSvgText(text);
  };
  xml.onopencdata = () => {
    if (styleDepth === 0) svgFail("CDATA is only accepted inside SVG styles.");
  };
  xml.oncdata = (text) => checkSvgText(text);
  xml.oncomment = () => svgFail("SVG comments are not accepted.");
  xml.ondoctype = () => svgFail("SVG declarations are not accepted.");
  xml.onprocessinginstruction = (instruction) => {
    if (instruction.name.toLowerCase() !== "xml" || rootSeen) svgFail("SVG processing instruction is not accepted.");
  };
  xml.onscript = () => svgFail("SVG scripts are not accepted.");
  xml.onerror = (error) => {
    parserError = error;
    xml.resume();
  };
  try {
    xml.write(source).close();
  } catch (error) {
    if (error instanceof DesignOperationError) throw error;
    parserError = error instanceof Error ? error : new Error("SVG parse failed");
  }
  if (parserError) svgFail("SVG is not well formed.");
  if (!rootSeen || !rootClosed || stack.length !== 0 || selfClosing.size !== 0) svgFail("SVG must contain one complete root element.");
  return elements;
}

/** Optimize a closed SVG subset. The Rust job owner performs rasterization. */
export function optimizeSvg(input: Uint8Array): OptimizeResult {
  const source = decodeInput(input);
  validateSvg(source);
  let optimized: string;
  try {
    // SVGO's cleanupIds does not understand ARIA IDREF attributes. When an
    // accessible name or description points at local title/desc elements,
    // preserve the IDs so optimization cannot leave dangling references.
    const hasAccessibilityReferences = /\baria-(?:labelledby|describedby)\s*=\s*(?:"[^"]+"|'[^']+')/iu.test(source);
    const plugins: NonNullable<Parameters<typeof optimize>[1]>["plugins"] = hasAccessibilityReferences
      ? [{ name: "preset-default", params: { overrides: {
        cleanupIds: false,
        removeDesc: false,
        removeEmptyContainers: false,
      } } }]
      : ["preset-default"];
    optimized = optimize(source, { multipass: false, plugins }).data;
  } catch {
    fail("svg_invalid", "SVG could not be parsed by the optimizer.");
  }
  const elementCount = validateSvg(optimized);
  const output = bytes(optimized);
  return { output, format: "svg", input_bytes: input.byteLength, output_bytes: output.byteLength, element_count: elementCount };
}
