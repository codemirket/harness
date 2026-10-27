import { createHash } from "node:crypto";
import { inflateRawSync } from "node:zlib";
import { OpenPencilError, type PencilGraph, type PencilNode, type PencilRuntime, type ParsedFig } from "./openpencil-runtime.js";
export { OpenPencilError } from "./openpencil-runtime.js";
export const OPENPENCIL_VERSION = "0.16.0";
export const OPENPENCIL_MODE = "editable_file";
export const OPENPENCIL_MAX_INPUT = 1024 * 1024;
export const OPENPENCIL_MAX_OUTPUT = 16 * 1024 * 1024;
export const OPENPENCIL_MAX_NODES = 256;
export const OPENPENCIL_MAX_DEPTH = 16;
export const OPENPENCIL_MAX_EDITS = 64;
const MAX_EXPANDED = 4 * 1024 * 1024;
const encoder = new TextEncoder();
const namePattern = /^[^/\x00-\x1f\x7f-\x9f\ud800-\udfff]{1,128}$/u;
const kinds = { frame: "FRAME", component: "COMPONENT", rectangle: "RECTANGLE", ellipse: "ELLIPSE", text: "TEXT" } as const;
const containers = ["FRAME", "COMPONENT"];
const nativeKinds = ["FRAME", "SYMBOL", "RECTANGLE", "ELLIPSE", "TEXT"];
const layoutKeys = ["layout", "gap", "padding_top", "padding_right", "padding_bottom", "padding_left", "primary_align", "counter_align"];
const textKeys = ["text", "font_size", "line_height"];
export interface OpenPencilProperties {
    x?: number;
    y?: number;
    width?: number;
    height?: number;
    fill?: string;
    opacity?: number;
    corner_radius?: number;
    text?: string;
    font_size?: number;
    line_height?: number;
    layout?: "none" | "horizontal" | "vertical";
    gap?: number;
    padding_top?: number;
    padding_right?: number;
    padding_bottom?: number;
    padding_left?: number;
    primary_align?: "start" | "center" | "end" | "space_between";
    counter_align?: "start" | "center" | "end";
}
export interface OpenPencilNode extends OpenPencilProperties {
    kind: keyof typeof kinds;
    name: string;
    width: number;
    height: number;
    children?: OpenPencilNode[];
}
export interface OpenPencilScene {
    page: string;
    nodes: OpenPencilNode[];
}
export interface OpenPencilEdit {
    path: string;
    properties: OpenPencilProperties;
}
export interface OpenPencilResult {
    output: Uint8Array;
    format: "fig" | "svg";
    node_count: number;
    engine_version: typeof OPENPENCIL_VERSION;
    mode: typeof OPENPENCIL_MODE;
}
function fail(code: string, message: string): never { throw new OpenPencilError(code, message); }
function object(v: unknown): v is Record<string, unknown> { return v !== null && typeof v === "object" && !Array.isArray(v); }
function keys(v: Record<string, unknown>, allowed: readonly string[]): void {
    if (Object.keys(v).some(k => !allowed.includes(k)))
        fail("openpencil_unsupported", "Unknown fields or unsupported editable design features are unavailable.");
}
function finite(v: unknown, min: number, max: number): v is number { return typeof v === "number" && Number.isFinite(v) && v >= min && v <= max; }
function validPath(v: unknown): v is string { return typeof v === "string" && v.length <= 2048 && v.split("/").length <= OPENPENCIL_MAX_DEPTH && v.split("/").every(s => namePattern.test(s)); }
const propertyKeys = ["x", "y", "width", "height", "fill", "opacity", "corner_radius", ...textKeys, ...layoutKeys];
function properties(v: unknown): OpenPencilProperties {
    if (!object(v))
        fail("openpencil_invalid", "Properties must be an object.");
    keys(v, propertyKeys);
    for (const [key, value] of Object.entries(v)) {
        if (["x", "y"].includes(key) && !finite(value, -2048, 2048))
            fail("openpencil_bounds", "Position exceeds the editable-file limit.");
        if (["width", "height"].includes(key) && !finite(value, 1, 2048))
            fail("openpencil_bounds", "Dimensions must be between 1 and 2048 pixels.");
        if (key === "corner_radius" && !finite(value, 0, 512))
            fail("openpencil_bounds", "Corner radius exceeds its limit.");
        if (key === "opacity" && !finite(value, 0, 1))
            fail("openpencil_bounds", "Opacity must be between 0 and 1.");
        if (key === "fill" && (typeof value !== "string" || !/^#[0-9a-fA-F]{6}(?:[0-9a-fA-F]{2})?$/u.test(value)))
            fail("openpencil_invalid", "Fill must be a hexadecimal color.");
    }
    for (const key of ["gap", "padding_top", "padding_right", "padding_bottom", "padding_left", "line_height"])
        if (v[key] !== undefined && !finite(v[key], key === "line_height" ? 1 : 0, 512))
            fail("openpencil_bounds", "Layout spacing or line height exceeds its bound.");
    if (v.font_size !== undefined && !finite(v.font_size, 1, 256))
        fail("openpencil_bounds", "Font size must be 1–256 pixels.");
    if (v.text !== undefined && (typeof v.text !== "string" || !v.text.length || v.text.length > 4096 || /[\x00-\x1f\x7f-\x9f\ud800-\udfff\u202a-\u202e\u2066-\u2069]/u.test(v.text)))
        fail("openpencil_unsupported", "Text must be a single nonempty line of at most 4096 UTF-16 units without control characters.");
    if (v.layout !== undefined && !["none", "horizontal", "vertical"].includes(v.layout as string)
        || v.primary_align !== undefined && !["start", "center", "end", "space_between"].includes(v.primary_align as string)
        || v.counter_align !== undefined && !["start", "center", "end"].includes(v.counter_align as string))
        fail("openpencil_unsupported", "Only fixed no-wrap horizontal/vertical layout and bounded alignment are supported.");
    return v as OpenPencilProperties;
}
/** Duplicate keys and excess depth are refused before JSON.parse loses source information. */
export function decodeOpenPencilJson(input: Uint8Array): unknown {
    if (input.length > OPENPENCIL_MAX_INPUT)
        fail("input_too_large", "JSON input exceeds 1 MiB.");
    let source: string;
    try {
        source = new TextDecoder("utf-8", { fatal: true }).decode(input);
    }
    catch {
        fail("openpencil_invalid", "Input must be UTF-8 JSON.");
    }
    let at = 0;
    const ws = (): void => { while (at < source.length && /[\x20\t\n\r]/u.test(source[at] ?? ""))
        at++; };
    const string = (): string => { const start = at++; while (at < source.length) {
        const c = source[at++];
        if (c === "\\")
            at++;
        else if (c === '"')
            return JSON.parse(source.slice(start, at)) as string;
    } throw new Error("string"); };
    const walk = (depth: number): void => {
        if (depth > OPENPENCIL_MAX_DEPTH * 3 + 8)
            fail("openpencil_bounds", "JSON nesting exceeds its limit.");
        ws();
        const c = source[at];
        if (c === "{") {
            at++;
            ws();
            const seen = new Set<string>();
            if (source[at] === "}") {
                at++;
                return;
            }
            while (at < source.length) {
                if (source[at] !== '"')
                    throw new Error("key");
                const k = string();
                if (seen.has(k) || ["__proto__", "constructor", "prototype"].includes(k))
                    fail("openpencil_invalid", "Duplicate or unsafe JSON keys are unsupported.");
                seen.add(k);
                ws();
                if (source[at++] !== ":")
                    throw new Error("colon");
                walk(depth + 1);
                ws();
                if (source[at] === "}") {
                    at++;
                    return;
                }
                if (source[at++] !== ",")
                    throw new Error("comma");
                ws();
            }
            throw new Error("object");
        }
        if (c === "[") {
            at++;
            ws();
            if (source[at] === "]") {
                at++;
                return;
            }
            while (at < source.length) {
                walk(depth + 1);
                ws();
                if (source[at] === "]") {
                    at++;
                    return;
                }
                if (source[at++] !== ",")
                    throw new Error("comma");
            }
            throw new Error("array");
        }
        if (c === '"') {
            string();
            return;
        }
        const start = at;
        while (at < source.length && !/[\s,\]}]/u.test(source[at] ?? ""))
            at++;
        if (at === start)
            throw new Error("value");
    };
    try {
        walk(0);
        ws();
        if (at !== source.length)
            throw new Error("trailing");
        return JSON.parse(source);
    }
    catch (e) {
        if (e instanceof OpenPencilError)
            throw e;
        fail("openpencil_invalid", "Input must be valid JSON.");
    }
}
function scene(input: Uint8Array): OpenPencilScene {
    const value = decodeOpenPencilJson(input);
    if (!object(value))
        fail("openpencil_invalid", "Scene must be an object.");
    keys(value, ["page", "nodes"]);
    if (typeof value.page !== "string" || !namePattern.test(value.page) || !Array.isArray(value.nodes) || !value.nodes.length)
        fail("openpencil_invalid", "Scene needs a bounded page name and nodes.");
    let count = 0;
    const walk = (nodes: unknown[], depth: number): void => {
        if (depth > OPENPENCIL_MAX_DEPTH)
            fail("openpencil_bounds", "Scene nesting exceeds its limit.");
        const names = new Set<string>();
        for (const n of nodes) {
            if (!object(n))
                fail("openpencil_invalid", "Node must be an object.");
            keys(n, ["kind", "name", "children", ...propertyKeys]);
            if (typeof n.kind !== "string" || !Object.hasOwn(kinds, n.kind))
                fail("openpencil_unsupported", "Create supports frames, components, rectangles, ellipses and single-line text.");
            if (typeof n.name !== "string" || !namePattern.test(n.name) || names.has(n.name))
                fail("openpencil_invalid", "Sibling node names must be unique and bounded.");
            names.add(n.name);
            if (++count > OPENPENCIL_MAX_NODES)
                fail("openpencil_bounds", "Scene exceeds 256 nodes.");
            properties(Object.fromEntries(Object.entries(n).filter(([k]) => propertyKeys.includes(k))));
            if (!finite(n.width, 1, 2048) || !finite(n.height, 1, 2048))
                fail("openpencil_bounds", "Each node needs bounded width and height.");
            validateKind(kinds[n.kind as keyof typeof kinds], n);
            if ((n.layout === undefined || n.layout === "none") && layoutKeys.slice(1).some(key => n[key] !== undefined))
                fail("openpencil_unsupported", "Spacing and alignment require horizontal or vertical layout.");
            if (n.kind === "text" && typeof n.text !== "string")
                fail("openpencil_invalid", "Text nodes require text content.");
            if (n.children !== undefined) {
                if (!["frame", "component"].includes(n.kind as string) || !Array.isArray(n.children))
                    fail("openpencil_unsupported", "Only frames and components accept children.");
                walk(n.children, depth + 1);
            }
        }
    };
    walk(value.nodes, 1);
    return value as unknown as OpenPencilScene;
}
function paint(fill: string): Record<string, unknown> { return { type: "SOLID", color: { r: parseInt(fill.slice(1, 3), 16) / 255, g: parseInt(fill.slice(3, 5), 16) / 255, b: parseInt(fill.slice(5, 7), 16) / 255, a: fill.length === 9 ? parseInt(fill.slice(7), 16) / 255 : 1 }, opacity: 1, visible: true, blendMode: "NORMAL" }; }
function graphProperties(p: OpenPencilProperties): Record<string, unknown> {
    const out: Record<string, unknown> = {};
    for (const key of ["x", "y", "width", "height", "opacity"] as const)
        if (p[key] !== undefined)
            out[key] = p[key];
    if (p.fill !== undefined)
        out.fills = [paint(p.fill)];
    if (p.corner_radius !== undefined)
        out.cornerRadius = p.corner_radius;
    const direct = { text: "text", font_size: "fontSize", line_height: "lineHeight", gap: "itemSpacing", padding_top: "paddingTop", padding_right: "paddingRight", padding_bottom: "paddingBottom", padding_left: "paddingLeft" } as const;
    for (const [key, field] of Object.entries(direct))
        if (p[key as keyof OpenPencilProperties] !== undefined) out[field] = p[key as keyof OpenPencilProperties];
    if (p.layout !== undefined) out.layoutMode = p.layout.toUpperCase();
    const align = { start: "MIN", center: "CENTER", end: "MAX", space_between: "SPACE_BETWEEN" };
    if (p.primary_align !== undefined) out.primaryAxisAlign = align[p.primary_align];
    if (p.counter_align !== undefined) out.counterAxisAlign = align[p.counter_align];
    return out;
}
function validateKind(kind: string, p: OpenPencilProperties): void {
    if ((kind === "ELLIPSE" || kind === "TEXT") && p.corner_radius !== undefined)
        fail("openpencil_unsupported", "This node kind has no corner radius.");
    if (kind !== "TEXT" && textKeys.some(k => p[k as keyof OpenPencilProperties] !== undefined))
        fail("openpencil_unsupported", "Text properties require a text node.");
    if (!containers.includes(kind) && layoutKeys.some(k => p[k as keyof OpenPencilProperties] !== undefined))
        fail("openpencil_unsupported", "Layout properties require a frame or component.");
}
/** Exact fixed-size flex subset. No measuring, wrapping, shrinking or external layout engine. */
function applyFixedLayouts(graph: PencilGraph, verify = false): void {
    for (const parent of graph.nodes.values()) {
        if (parent.layoutMode === "NONE") continue;
        const row = parent.layoutMode === "HORIZONTAL", children = graph.getChildren(parent.id);
        const number = (key: string): number => parent[key] as number;
        const start = row ? number("paddingLeft") : number("paddingTop");
        const end = row ? number("paddingRight") : number("paddingBottom");
        const crossStart = row ? number("paddingTop") : number("paddingLeft");
        const crossEnd = row ? number("paddingBottom") : number("paddingRight");
        const main = row ? parent.width : parent.height, cross = row ? parent.height : parent.width;
        const gap = number("itemSpacing"), total = children.reduce((sum, child) => sum + (row ? child.width : child.height), 0);
        const spare = main - start - end - total - gap * Math.max(0, children.length - 1);
        if (spare < -0.0001 || children.some(child => (row ? child.height : child.width) > cross - crossStart - crossEnd + 0.0001))
            fail("openpencil_bounds", "Fixed children, padding and gap must fit inside their layout container.");
        const justify = parent.primaryAxisAlign, align = parent.counterAxisAlign;
        const spacing = justify === "SPACE_BETWEEN" && children.length > 1 ? gap + spare / (children.length - 1) : gap;
        let cursor = start + (justify === "CENTER" ? spare / 2 : justify === "MAX" ? spare : 0);
        for (const child of children) {
            const crossSpare = cross - crossStart - crossEnd - (row ? child.height : child.width);
            const other = crossStart + (align === "CENTER" ? crossSpare / 2 : align === "MAX" ? crossSpare : 0);
            const position = row ? { x: cursor, y: other } : { x: other, y: cursor };
            if (verify) {
                if (Math.abs(child.x - position.x) > 0.001 || Math.abs(child.y - position.y) > 0.001)
                    fail("openpencil_unsupported", "Saved child geometry does not match its fixed layout metadata.");
            } else graph.updateNode(child.id, position);
            cursor += (row ? child.width : child.height) + spacing;
        }
    }
}
function buffer(input: Uint8Array): ArrayBuffer { return Uint8Array.from(input).buffer; }
function crc32(bytes: Uint8Array): number { let crc = 0xffffffff; for (const b of bytes) {
    crc ^= b;
    for (let bit = 0; bit < 8; bit++)
        crc = (crc >>> 1) ^ ((crc & 1) ? 0xedb88320 : 0);
} return (crc ^ 0xffffffff) >>> 0; }
/** Only the bounded canonical OpenPencil 0.16 DEFLATE schema is compiled upstream. */
function preflight(input: Uint8Array, canonicalSchema: Uint8Array): void {
    if (!input.length || input.length > OPENPENCIL_MAX_INPUT)
        fail("input_too_large", ".fig input exceeds 1 MiB.");
    const data = Buffer.from(input);
    let offset = 0, total = 0, canvas: Buffer | undefined;
    const local = new Map<string, {
        offset: number;
        method: number;
        compressed: number;
        expanded: number;
        crc: number;
    }>();
    try {
        while (offset + 30 <= data.length && data.readUInt32LE(offset) === 0x04034b50) {
            if (local.size >= 3)
                throw new Error("entries");
            const flags = data.readUInt16LE(offset + 6), method = data.readUInt16LE(offset + 8), crc = data.readUInt32LE(offset + 14), compressed = data.readUInt32LE(offset + 18), expanded = data.readUInt32LE(offset + 22), nl = data.readUInt16LE(offset + 26), el = data.readUInt16LE(offset + 28);
            const name = data.subarray(offset + 30, offset + 30 + nl).toString("utf8"), start = offset + 30 + nl + el, end = start + compressed;
            if (flags !== 0 || ![0, 8].includes(method) || el !== 0 || end > data.length || expanded > MAX_EXPANDED || total + expanded > MAX_EXPANDED || !["canvas.fig", "thumbnail.png", "meta.json"].includes(name) || local.has(name))
                throw new Error("entry");
            const raw = method === 0 ? data.subarray(start, end) : inflateRawSync(data.subarray(start, end), { maxOutputLength: MAX_EXPANDED });
            if (raw.length !== expanded || crc32(raw) !== crc)
                throw new Error("crc");
            local.set(name, { offset, method, compressed, expanded, crc });
            if (name === "canvas.fig")
                canvas = raw;
            total += expanded;
            offset = end;
        }
        if (!canvas || canvas.length < 20 || canvas.subarray(0, 8).toString() !== "fig-kiwi" || canvas.readUInt32LE(8) !== 101)
            throw new Error("canvas");
        const cd = offset, names = new Set<string>();
        while (offset + 46 <= data.length && data.readUInt32LE(offset) === 0x02014b50) {
            const nl = data.readUInt16LE(offset + 28), el = data.readUInt16LE(offset + 30), cl = data.readUInt16LE(offset + 32), name = data.subarray(offset + 46, offset + 46 + nl).toString("utf8"), e = local.get(name);
            if (!e || names.has(name) || el !== 0 || cl !== 0 || data.readUInt16LE(offset + 8) !== 0 || data.readUInt16LE(offset + 10) !== e.method || data.readUInt32LE(offset + 16) !== e.crc || data.readUInt32LE(offset + 20) !== e.compressed || data.readUInt32LE(offset + 24) !== e.expanded || data.readUInt32LE(offset + 42) !== e.offset || data.readUInt16LE(offset + 34) !== 0)
                throw new Error("directory");
            names.add(name);
            offset += 46 + nl;
        }
        if (offset + 22 !== data.length || data.readUInt32LE(offset) !== 0x06054b50 || data.readUInt16LE(offset + 4) !== 0 || data.readUInt16LE(offset + 6) !== 0 || data.readUInt16LE(offset + 8) !== local.size || data.readUInt16LE(offset + 10) !== local.size || names.size !== local.size || data.readUInt32LE(offset + 12) !== offset - cd || data.readUInt32LE(offset + 16) !== cd || data.readUInt16LE(offset + 20) !== 0)
            throw new Error("end");
        let at = 12, count = 0, inner = 0;
        while (at < canvas.length) {
            const size = canvas.readUInt32LE(at);
            at += 4;
            if (++count > 2 || at + size > canvas.length)
                throw new Error("chunk");
            const raw = inflateRawSync(canvas.subarray(at, at + size), { maxOutputLength: MAX_EXPANDED });
            inner += raw.length;
            if (inner > MAX_EXPANDED)
                throw new Error("expanded");
            if (count === 1 && !raw.equals(Buffer.from(canonicalSchema)))
                throw new Error("schema");
            at += size;
        }
        if (at !== canvas.length || count !== 2)
            throw new Error("chunks");
    }
    catch {
        fail("openpencil_container", "Only bounded version-101 DEFLATE archives with the canonical OpenPencil 0.16 schema and no assets are supported.");
    }
}
const rawFields = ["guid", "phase", "type", "name", "visible", "opacity", "transform", "strokeWeight", "strokeAlign", "strokeJoin", "documentColorProfile", "parentIndex", "pageType", "size", "cornerRadius", "fillPaints", "pluginData", "frameMaskDisabled", "isSymbolPublishable", "stackMode", "stackSpacing", "stackVerticalPadding", "stackHorizontalPadding", "stackPaddingBottom", "stackPaddingRight", "stackPrimarySizing", "stackCounterSizing", "stackPrimaryAlignItems", "stackCounterAlignItems", "bordersTakeSpace", "fontSize", "fontName", "textData", "textAutoResize", "textAlignHorizontal", "textAlignVertical", "textUserLayoutVersion", "textExplicitLayoutVersion", "textBidiVersion", "textDecorationSkipInk", "fontVariantCommonLigatures", "fontVariantContextualLigatures", "fontVersion", "emojiImageSet", "lineHeight", "letterSpacing"];
function guid(value: unknown): string { if (!object(value) || Object.keys(value).length !== 2 || !finite(value.sessionID, 0, 0xffffffff) || !Number.isInteger(value.sessionID) || !finite(value.localID, 0, 0xffffffff) || !Number.isInteger(value.localID))
    fail("openpencil_invalid", "Invalid native record identity."); return `${value.sessionID}:${value.localID}`; }
function validateNativeFeatures(n: Record<string, unknown>): void {
    const isContainer = n.type === "FRAME" || n.type === "SYMBOL", isText = n.type === "TEXT";
    if (n.isSymbolPublishable !== undefined && (n.type !== "SYMBOL" || n.isSymbolPublishable !== false))
        fail("openpencil_unsupported", "Published or external components are unsupported.");
    const stackKeys = ["stackMode", "stackSpacing", "stackVerticalPadding", "stackHorizontalPadding", "stackPaddingBottom", "stackPaddingRight", "stackPrimarySizing", "stackCounterSizing", "stackPrimaryAlignItems", "stackCounterAlignItems", "bordersTakeSpace"];
    if (stackKeys.some(key => n[key] !== undefined)) {
        if (!isContainer || !["HORIZONTAL", "VERTICAL"].includes(n.stackMode as string)
            || n.stackPrimarySizing !== "FIXED" || n.stackCounterSizing !== "FIXED" || n.bordersTakeSpace !== false
            || !["MIN", "CENTER", "MAX", "SPACE_BETWEEN"].includes(n.stackPrimaryAlignItems as string)
            || !["MIN", "CENTER", "MAX"].includes(n.stackCounterAlignItems as string)
            || !["stackSpacing", "stackVerticalPadding", "stackHorizontalPadding", "stackPaddingBottom", "stackPaddingRight"].every(key => finite(n[key], 0, 512)))
            fail("openpencil_unsupported", "Only fixed no-wrap native horizontal/vertical layout is supported.");
    }
    const textFields = ["fontSize", "fontName", "textData", "textAutoResize", "textAlignHorizontal", "textAlignVertical", "textUserLayoutVersion", "textExplicitLayoutVersion", "textBidiVersion", "textDecorationSkipInk", "fontVariantCommonLigatures", "fontVariantContextualLigatures", "fontVersion", "emojiImageSet", "lineHeight", "letterSpacing"];
    if (!isText && textFields.some(key => n[key] !== undefined)) fail("openpencil_unsupported", "Text metadata on non-text nodes is unsupported.");
    if (isText) {
        if (!finite(n.fontSize, 1, 256) || !object(n.fontName) || JSON.stringify(n.fontName) !== JSON.stringify({ family: "Noto Sans", style: "Regular", postscript: "" })
            || !object(n.textData) || typeof n.textData.characters !== "string" || Object.keys(n.textData).some(key => !["characters", "lines"].includes(key))
            || JSON.stringify(n.textData.lines) !== JSON.stringify([{ lineType: "PLAIN", styleId: 0, indentationLevel: 0, sourceDirectionality: "AUTO", listStartOffset: 0, isFirstLineOfList: false }])
            || n.textAutoResize !== "NONE" || n.textAlignHorizontal !== "LEFT" || n.textAlignVertical !== "TOP"
            || n.textUserLayoutVersion !== 4 || n.textExplicitLayoutVersion !== 1 || n.textBidiVersion !== 1
            || n.textDecorationSkipInk !== true || n.fontVariantCommonLigatures !== true || n.fontVariantContextualLigatures !== true
            || n.fontVersion !== "" || n.emojiImageSet !== "APPLE"
            || !object(n.lineHeight) || Object.keys(n.lineHeight).length !== 2 || n.lineHeight.units !== "PIXELS" || !finite(n.lineHeight.value, 1, 512)
            || !object(n.letterSpacing) || Object.keys(n.letterSpacing).length !== 2 || n.letterSpacing.units !== "PIXELS" || n.letterSpacing.value !== 0)
            fail("openpencil_unsupported", "Only plain fixed single-line regular Noto Sans text metadata is supported.");
        properties({ text: n.textData.characters });
    }
    if (n.pluginData !== undefined) {
        if (!Array.isArray(n.pluginData) || n.pluginData.length > (isText ? 2 : 1)) fail("openpencil_unsupported", "Unsupported plugin data.");
        const seen = new Set<string>();
        for (const entry of n.pluginData) {
            if (!object(entry) || Object.keys(entry).length !== 3 || entry.pluginID !== "open-pencil" || entry.value !== "AUTO"
                || !["layoutDirection", ...(isText ? ["textDirection"] : [])].includes(entry.key as string) || seen.has(entry.key as string))
                fail("openpencil_unsupported", "Unsupported plugin data.");
            seen.add(entry.key as string);
        }
    }
}
function validateRecords(parsed: ParsedFig): Map<string, Record<string, unknown>> {
    if (parsed.images.length || parsed.blobs.length || parsed.nodeChanges.length > OPENPENCIL_MAX_NODES + 2)
        fail("openpencil_unsupported", "Images, embedded text glyphs and other resources are unavailable in editable-file mode.");
    const records = new Map<string, Record<string, unknown>>();
    let docs = 0, pages = 0;
    for (const n of parsed.nodeChanges) {
        keys(n, rawFields);
        const id = guid(n.guid);
        if (records.has(id))
            fail("openpencil_invalid", "Duplicate native record identity.");
        records.set(id, n);
        if (n.type === "DOCUMENT") {
            docs++;
            if (n.documentColorProfile !== "SRGB")
                fail("openpencil_unsupported", "Only sRGB documents are supported.");
        }
        else if (n.type === "CANVAS") {
            pages++;
            if (n.pageType !== "DESIGN")
                fail("openpencil_unsupported", "Only design pages are supported.");
        }
        else if (!nativeKinds.includes(n.type as string))
            fail("openpencil_unsupported", "Only fixed frames, components, rectangles, ellipses and text are supported.");
        if (typeof n.name !== "string" || !namePattern.test(n.name) || n.phase !== "CREATED" || n.visible !== true || !finite(n.opacity, 0, 1))
            fail("openpencil_unsupported", "Unsupported record state.");
        if (!object(n.transform) || Object.keys(n.transform).length !== 6 || n.transform.m00 !== 1 || n.transform.m01 !== 0 || n.transform.m10 !== 0 || n.transform.m11 !== 1 || !finite(n.transform.m02, -2048, 2048) || !finite(n.transform.m12, -2048, 2048))
            fail("openpencil_unsupported", "Only translations are supported.");
        if (n.type !== "DOCUMENT" && n.type !== "CANVAS" && (!object(n.size) || Object.keys(n.size).length !== 2 || !finite(n.size.x, 1, 2048) || !finite(n.size.y, 1, 2048)))
            fail("openpencil_bounds", "Unsupported node dimensions.");
        if (n.strokeWeight !== undefined && !finite(n.strokeWeight, 0, 2048) || n.strokeAlign !== undefined && !["CENTER", "INSIDE", "OUTSIDE"].includes(n.strokeAlign as string) || n.strokeJoin !== undefined && n.strokeJoin !== "MITER")
            fail("openpencil_unsupported", "Unsupported stroke metadata.");
        if (n.cornerRadius !== undefined && (!finite(n.cornerRadius, 0, 512) || ["ELLIPSE", "TEXT"].includes(n.type as string)))
            fail("openpencil_unsupported", "Unsupported corner radius.");
        if (n.frameMaskDisabled !== undefined && n.frameMaskDisabled !== true)
            fail("openpencil_unsupported", "Clipped frames are unsupported.");
        validateNativeFeatures(n);
        if (n.fillPaints !== undefined) {
            if (!Array.isArray(n.fillPaints) || n.fillPaints.length > 1)
                fail("openpencil_unsupported", "Only one solid fill is supported.");
            for (const f of n.fillPaints) {
                if (!object(f))
                    fail("openpencil_unsupported", "Unsupported fill.");
                keys(f, ["type", "color", "opacity", "visible", "blendMode"]);
                if (f.type !== "SOLID" || f.visible !== true || f.blendMode !== "NORMAL" || !finite(f.opacity, 0, 1) || !object(f.color) || Object.keys(f.color).length !== 4 || ![f.color.r, f.color.g, f.color.b, f.color.a].every(v => finite(v, 0, 1)))
                    fail("openpencil_unsupported", "Only bounded solid sRGB fills are supported.");
            }
        }
    }
    if (docs !== 1 || pages !== 1)
        fail("openpencil_unsupported", "Exactly one document and one page are supported.");
    for (const n of records.values()) {
        if (n.type === "DOCUMENT") {
            if (n.parentIndex !== undefined)
                fail("openpencil_invalid", "Document parent is invalid.");
            continue;
        }
        if (!object(n.parentIndex) || typeof n.parentIndex.position !== "string" || n.parentIndex.position.length > 128)
            fail("openpencil_invalid", "Record parent is invalid.");
        const parent = records.get(guid(n.parentIndex.guid));
        if (!parent || (n.type === "CANVAS" ? parent.type !== "DOCUMENT" : parent.type !== "CANVAS" && parent.type !== "FRAME" && parent.type !== "SYMBOL"))
            fail("openpencil_unsupported", "Unsupported parent hierarchy.");
    }
    for (const [id, record] of records) {
        if (record.type === "DOCUMENT")
            continue;
        const seen = new Set<string>();
        let current = record;
        let depth = 0;
        while (current.type !== "DOCUMENT") {
            const key = guid(current.guid);
            if (seen.has(key) || ++depth > OPENPENCIL_MAX_DEPTH + 1)
                fail("openpencil_bounds", "Native hierarchy is cyclic or too deep.");
            seen.add(key);
            if (!object(current.parentIndex))
                fail("openpencil_invalid", "Native parent is missing.");
            const parent = records.get(guid(current.parentIndex.guid));
            if (!parent || guid(parent.guid) === id)
                fail("openpencil_invalid", "Native hierarchy is cyclic.");
            current = parent;
        }
    }
    return records;
}
function indexGraph(graph: PencilGraph): Map<string, PencilNode> {
    const pages = graph.getPages();
    if (pages.length !== 1)
        fail("openpencil_unsupported", "Exactly one page is supported.");
    const map = new Map<string, PencilNode>(), ids = new Set<string>();
    const walk = (parent: PencilNode, path: string, depth: number): void => { if (depth > OPENPENCIL_MAX_DEPTH)
        fail("openpencil_bounds", "Document nesting exceeds its limit."); for (const n of graph.getChildren(parent.id)) {
        const key = path ? `${path}/${n.name}` : n.name;
        if (!validPath(key) || map.has(key) || ids.has(n.id))
            fail("openpencil_invalid", "Document names or hierarchy are ambiguous.");
        if (!["NONE", "HORIZONTAL", "VERTICAL"].includes(n.layoutMode) || !Object.values(kinds).includes(n.type as typeof kinds[keyof typeof kinds]))
            fail("openpencil_unsupported", "Unsupported native node or layout mode.");
        map.set(key, n);
        ids.add(n.id);
        if (map.size > OPENPENCIL_MAX_NODES)
            fail("openpencil_bounds", "Document exceeds 256 nodes.");
        walk(n, key, depth + 1);
    } };
    walk(pages[0]!, "", 1);
    if (graph.images.size || graph.variables.size || graph.variableCollections.size || graph.enabledLibraries.size || graph.documentColorSpace !== "srgb")
        fail("openpencil_unsupported", "Resources and variables are unsupported.");
    return map;
}
function openFig(runtime: PencilRuntime, input: Uint8Array): {
    graph: PencilGraph;
    map: Map<string, PencilNode>;
    parsed: ParsedFig;
    records: Map<string, Record<string, unknown>>;
} {
    preflight(input, runtime.kiwi.getSchemaBytes());
    try {
        const parsed = runtime.fig.parseFigBuffer(buffer(input)), records = validateRecords(parsed);
        const assembled = runtime.fig.materializeDocument(parsed.nodeChanges, parsed.blobs);
        const map = indexGraph(assembled.graph);
        applyFixedLayouts(assembled.graph, true);
        if (map.size !== parsed.nodeChanges.length - 2)
            fail("openpencil_unsupported", "Unresolved native records are unsupported.");
        return { graph: assembled.graph, map, parsed, records };
    }
    catch (e) {
        if (e instanceof OpenPencilError)
            throw e;
        fail("openpencil_invalid", "The native editable document could not be opened.");
    }
}
function result(output: Uint8Array, format: "fig" | "svg", count: number): OpenPencilResult { if (!output.length || output.length > OPENPENCIL_MAX_OUTPUT)
    fail("output_limit", "Output exceeds 16 MiB."); return { output, format, node_count: count, engine_version: OPENPENCIL_VERSION, mode: OPENPENCIL_MODE }; }
export function createOpenPencil(runtime: PencilRuntime, input: Uint8Array): OpenPencilResult {
    const value = scene(input), graph = new runtime.scene.SceneGraph(), page = graph.getPages()[0] ?? graph.addPage(value.page);
    graph.updateNode(page.id, { name: value.page });
    const add = (nodes: OpenPencilNode[], parent: PencilNode): void => { for (const n of nodes) {
        if (parent.layoutMode !== "NONE" && (n.x !== undefined || n.y !== undefined))
            fail("openpencil_unsupported", "Layout children use computed positions; omit x and y.");
        const node = graph.createNode(kinds[n.kind], parent.id, { name: n.name, layoutMode: "NONE", primaryAxisSizing: "FIXED", counterAxisSizing: "FIXED", primaryAxisAlign: "MIN", counterAxisAlign: "MIN", layoutWrap: "NO_WRAP", itemSpacing: 0, paddingTop: 0, paddingRight: 0, paddingBottom: 0, paddingLeft: 0, fontFamily: "Noto Sans", fontWeight: 400, fontSize: 16, lineHeight: 24, textAutoResize: "NONE", textAlignHorizontal: "LEFT", textAlignVertical: "TOP", clipsContent: false, fills: [paint(n.fill ?? (["frame", "component"].includes(n.kind) ? "#ffffff" : "#000000"))], ...graphProperties(n) });
        if (n.children)
            add(n.children, node);
    } };
    add(value.nodes, page);
    applyFixedLayouts(graph);
    const documentGuid = { sessionID: 0, localID: 0 }, pageGuid = { sessionID: 0, localID: 1 }, blobs: Uint8Array[] = [], counter = { value: 2 };
    const records = [runtime.nodeChange.makeDocumentNodeChange(documentGuid, "srgb"), runtime.nodeChange.makeCanvasNodeChange(pageGuid, documentGuid, "!", value.page)];
    for (const [i, node] of graph.getChildren(page.id).entries())
        records.push(...runtime.nodeChange.sceneNodeToKiwi(node, pageGuid, i, counter, graph, blobs));
    const output = runtime.fig.writeFigArchive({ schemaDeflated: runtime.fflate.deflateSync(runtime.kiwi.getSchemaBytes()), kiwiData: runtime.kiwi.getCompiledSchema().encodeMessage({ type: "NODE_CHANGES", sessionID: 0, ackID: 0, nodeChanges: records, blobs: blobs.map(bytes => ({ bytes })) }), thumbnailPNG: Buffer.from("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8/5+hHgAHggJ/PchI7wAAAABJRU5ErkJggg==", "base64"), metaJSON: "{}", images: [], figKiwiVersion: 101 });
    const reopened = openFig(runtime, output);
    return result(output, "fig", reopened.map.size);
}
export function patchOpenPencil(runtime: PencilRuntime, input: Uint8Array, edits: unknown): OpenPencilResult {
    if (!Array.isArray(edits) || !edits.length || edits.length > OPENPENCIL_MAX_EDITS)
        fail("openpencil_bounds", "Supply 1–64 typed design edits.");
    const changes = edits.map(e => { if (!object(e))
        fail("openpencil_invalid", "Edit must be an object."); keys(e, ["path", "properties"]); if (!validPath(e.path))
        fail("openpencil_invalid", "Edit path is invalid."); const p = properties(e.properties); if (!Object.keys(p).length)
        fail("openpencil_invalid", "Edit needs design properties."); return { path: e.path, properties: p }; });
    const opened = openFig(runtime, input);
    for (const edit of changes) {
        const node = opened.map.get(edit.path);
        if (!node) fail("openpencil_not_found", "Design target was not found.");
        validateKind(node.type, edit.properties);
        if ((edit.properties.layout === "none" || edit.properties.layout === undefined && node.layoutMode === "NONE")
            && layoutKeys.slice(1).some(key => edit.properties[key as keyof OpenPencilProperties] !== undefined))
            fail("openpencil_unsupported", "Spacing and alignment require horizontal or vertical layout.");
        const parent = node.parentId ? opened.graph.getNode(node.parentId) : undefined;
        if (parent && parent.layoutMode !== "NONE" && (edit.properties.x !== undefined || edit.properties.y !== undefined))
            fail("openpencil_unsupported", "Layout children use computed positions; edit the container layout.");
        opened.graph.updateNode(node.id, graphProperties(edit.properties));
    }
    applyFixedLayouts(opened.graph);
    const page = opened.graph.getPages()[0]!, pageRecord = [...opened.records.values()].find(n => n.type === "CANVAS")!;
    const nodeIdToGuid = new Map<string, object>();
    const nativePath = (record: Record<string, unknown>): string => {
        const parent = opened.records.get(guid((record.parentIndex as Record<string, unknown>).guid))!;
        return parent.type === "CANVAS" ? record.name as string : `${nativePath(parent)}/${record.name as string}`;
    };
    for (const record of opened.records.values()) {
        if (record.type === "DOCUMENT" || record.type === "CANVAS") continue;
        const node = opened.map.get(nativePath(record));
        if (!node) fail("openpencil_invalid", "Native path identity is missing.");
        nodeIdToGuid.set(node.id, record.guid as object);
    }
    const records: Array<Record<string, unknown>> = [], blobs: Uint8Array[] = [];
    for (const [index, node] of opened.graph.getChildren(page.id).entries())
        records.push(...runtime.nodeChange.sceneNodeToKiwi(node, pageRecord.guid as object, index, { value: OPENPENCIL_MAX_NODES + 2 }, opened.graph, blobs, { nodeIdToGuid }));
    if (blobs.length) fail("openpencil_unsupported", "Embedded glyph resources are unsupported.");
    const fields = records.map(record => {
        // Upstream expands a uniform imported radius into redundant per-corner fields.
        // Retain our admitted uniform-radius representation, without broadening imports.
        for (const key of ["rectangleTopLeftCornerRadius", "rectangleTopRightCornerRadius", "rectangleBottomLeftCornerRadius", "rectangleBottomRightCornerRadius", "rectangleCornerRadiiIndependent"]) delete record[key];
        const previous = opened.records.get(guid(record.guid));
        if (!previous) fail("openpencil_invalid", "Native identity was not preserved.");
        return { guid: record.guid, set: Object.fromEntries(Object.entries(record).filter(([key]) => key !== "guid")), unset: Object.keys(previous).filter(key => key !== "guid" && !Object.hasOwn(record, key)) };
    });
    const output = runtime.fig.patchFigArchive(buffer(input), { records: [], fields, blobs: [], removed: [], replaceVariables: false }, { thumbnailPNG: opened.parsed.thumbnailPNG ?? new Uint8Array(), metaJSON: opened.parsed.metaJSON ?? "{}", images: [] });
    return result(output, "fig", openFig(runtime, output).map.size);
}
export function exportOpenPencilSvg(runtime: PencilRuntime, input: Uint8Array, targetPath: string): OpenPencilResult {
    if (!validPath(targetPath))
        fail("openpencil_invalid", "SVG export needs an explicit bounded design path.");
    const opened = openFig(runtime, input), target = opened.map.get(targetPath);
    if (!target)
        fail("openpencil_not_found", "SVG design target was not found.");
    const svg = runtime.svg.renderNodesToSVG(opened.graph, opened.graph.getPages()[0]!.id, [target.id]);
    if (!svg)
        fail("openpencil_export", "Shape could not be exported.");
    const dimensions = /<svg\b[^>]*\bwidth="([0-9.]+)"[^>]*\bheight="([0-9.]+)"/u.exec(svg);
    if (!dimensions || !finite(Number(dimensions[1]), 1, 2048) || !finite(Number(dimensions[2]), 1, 2048))
        fail("openpencil_bounds", "SVG visual bounds exceed 2048 pixels.");
    return result(encoder.encode(svg), "svg", opened.map.size);
}
