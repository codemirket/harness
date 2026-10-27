import { createHash } from "node:crypto";
import { lstat, readFile, readdir, realpath } from "node:fs/promises";
import { registerHooks } from "node:module";
import { isAbsolute, join, relative, resolve, sep } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { OPENPENCIL_FILES, OPENPENCIL_SELECTION_SHA256 } from "./openpencil-inventory.js";
export class OpenPencilError extends Error {
    readonly name = "OpenPencilError";
    constructor(readonly code: string, message: string) { super(message); }
}
export interface PencilNode {
    id: string;
    name: string;
    type: string;
    parentId: string | null;
    childIds: string[];
    x: number;
    y: number;
    width: number;
    height: number;
    opacity: number;
    cornerRadius: number;
    layoutMode: string;
    source: {
        id: string | null;
    };
    [key: string]: unknown;
}
export interface PencilGraph {
    nodes: Map<string, PencilNode>;
    images: Map<string, unknown>;
    variables: Map<string, unknown>;
    variableCollections: Map<string, unknown>;
    enabledLibraries: Set<string>;
    documentColorSpace: string;
    getPages(): PencilNode[];
    addPage(name: string): PencilNode;
    createNode(kind: string, parent: string, properties: Record<string, unknown>): PencilNode;
    updateNode(id: string, properties: Record<string, unknown>): void;
    getNode(id: string): PencilNode | undefined;
    getChildren(id: string): PencilNode[];
}
export interface ParsedFig {
    nodeChanges: Array<Record<string, unknown>>;
    blobs: Uint8Array[];
    images: Array<[
        string,
        Uint8Array
    ]>;
    thumbnailPNG: Uint8Array | null;
    metaJSON: string | null;
    figKiwiVersion: number;
}
export interface PencilRuntime {
    fig: {
        parseFigBuffer(input: ArrayBuffer): ParsedFig;
        materializeDocument(records: Array<Record<string, unknown>>, blobs: Uint8Array[], options?: object): {
            graph: PencilGraph;
            sources: Map<string, string>;
        };
        writeFigArchive(input: object): Uint8Array;
        patchFigArchive(input: ArrayBuffer, patch: object, extras: object): Uint8Array;
    };
    nodeChange: {
        makeDocumentNodeChange(guid: object, colorSpace: string): Record<string, unknown>;
        makeCanvasNodeChange(guid: object, parent: object, position: string, name: string): Record<string, unknown>;
        sceneNodeToKiwi(node: PencilNode, parent: object, index: number, counter: {
            value: number;
        }, graph: PencilGraph, blobs: Uint8Array[], options?: object): Array<Record<string, unknown>>;
    };
    kiwi: {
        initCodec(): Promise<void>;
        getSchemaBytes(): Uint8Array;
        getCompiledSchema(): {
            encodeMessage(message: object): Uint8Array;
        };
    };
    scene: {
        SceneGraph: new () => PencilGraph;
    };
    fflate: {
        deflateSync(input: Uint8Array): Uint8Array;
    };
    svg: {
        renderNodesToSVG(graph: PencilGraph, page: string, nodes: string[]): string | null;
    };
}
function fail(): never { throw new OpenPencilError("openpencil_runtime", "Private OpenPencil runtime differs from its compiled file inventory."); }
const hash = (bytes: Uint8Array): string => createHash("sha256").update(bytes).digest("hex");
function inside(root: string, path: string): boolean { const r = relative(root, path); return r !== "" && !r.startsWith(`..${sep}`) && r !== ".." && !isAbsolute(r); }
/** Pins come from compiled inventory; runtime.json cannot authorize files or change hashes. */
export async function loadOpenPencilRuntime(root: string, expectedManifestSha256: string): Promise<PencilRuntime> {
    if (!isAbsolute(root) || resolve(root) !== root || !/^[a-f0-9]{64}$/u.test(expectedManifestSha256))
        fail();
    const metadata = await lstat(root);
    if (!metadata.isDirectory() || metadata.isSymbolicLink() || await realpath(root) !== root)
        fail();
    const entries = new Map(OPENPENCIL_FILES.map((entry) => [entry.path, entry]));
    const directories = new Set<string>();
    for (const key of entries.keys()) {
        const parts = key.split("/");
        for (let i = 1; i < parts.length; i++)
            directories.add(parts.slice(0, i).join("/"));
    }
    const actual: string[] = [];
    const walk = async (directory: string): Promise<void> => {
        for (const name of await readdir(directory)) {
            const path = join(directory, name), info = await lstat(path);
            if (info.isSymbolicLink())
                fail();
            if (info.isDirectory()) {
                if (!directories.has(relative(root, path).split(sep).join("/")))
                    fail();
                await walk(path);
                continue;
            }
            if (!info.isFile())
                fail();
            const key = relative(root, path).split(sep).join("/");
            actual.push(key);
            if (key === "runtime.json")
                continue;
            const entry = entries.get(key);
            if (!entry || info.size !== entry.bytes || hash(await readFile(path)) !== entry.sha256)
                fail();
        }
    };
    await walk(root);
    if (actual.length !== entries.size + 1 || !actual.includes("runtime.json"))
        fail();
    for (const key of entries.keys())
        if (!actual.includes(key))
            fail();
    const manifestBytes = await readFile(join(root, "runtime.json"));
    if (manifestBytes.length > 1024 * 1024 || hash(manifestBytes) !== expectedManifestSha256)
        fail();
    const manifest = JSON.parse(manifestBytes.toString("utf8")) as {
        schema?: unknown;
        selection_sha256?: unknown;
        files?: unknown;
    };
    if (manifest.schema !== 1 || manifest.selection_sha256 !== OPENPENCIL_SELECTION_SHA256 || JSON.stringify(manifest.files) !== JSON.stringify(OPENPENCIL_FILES))
        fail();
    const forbidden = /(?:canvaskit|yoga-layout|\.wasm(?:$|[?#]))/iu;
    registerHooks({ resolve(specifier, context, nextResolve) {
            if (forbidden.test(specifier))
                fail();
            const result = nextResolve(specifier, context);
            if (result.url.startsWith("file:")) {
                const path = fileURLToPath(result.url);
                const callerInRuntime = context.parentURL?.startsWith(pathToFileURL(root + sep).href) ?? false;
                if (inside(root, path)) {
                    const entry = entries.get(relative(root, path).split(sep).join("/"));
                    if (!entry || (entry.kind !== "module" && entry.kind !== "package_metadata"))
                        fail();
                }
                else if (callerInRuntime)
                    fail();
            }
            else if (!result.url.startsWith("node:"))
                fail();
            return result;
        } });
    // These are finite operation guardrails, not an OS sandbox or host permission boundary.
    globalThis.fetch = async () => { throw new OpenPencilError("openpencil_external", "Network loading is unavailable in editable-file mode."); };
    WebAssembly.compile = async () => fail();
    WebAssembly.instantiate = (async () => fail()) as typeof WebAssembly.instantiate;
    const load = async (path: string): Promise<unknown> => import(pathToFileURL(join(root, path)).href);
    const fig = await load("node_modules/@open-pencil/fig/dist/index.js") as PencilRuntime["fig"];
    const nodeChange = await load("node_modules/@open-pencil/fig/dist/node-change.js") as PencilRuntime["nodeChange"];
    const kiwi = await load("node_modules/@open-pencil/kiwi/dist/fig/codec.js") as PencilRuntime["kiwi"];
    const scene = await load("node_modules/@open-pencil/scene-graph/dist/index.js") as PencilRuntime["scene"];
    const fflate = await load("node_modules/fflate/esm/index.mjs") as PencilRuntime["fflate"];
    const svg = await load("node_modules/@open-pencil/core/dist/io/formats/svg/export.js") as PencilRuntime["svg"];
    await kiwi.initCodec();
    return { fig, nodeChange, kiwi, scene, fflate, svg };
}
