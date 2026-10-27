import { createHash } from "node:crypto";
import { constants } from "node:fs";
import { lstat, open, realpath } from "node:fs/promises";
import { dirname, extname, isAbsolute, resolve } from "node:path";
import { loadOpenPencilRuntime, OpenPencilError, type PencilRuntime } from "./openpencil-runtime.js";
import { createOpenPencil, decodeOpenPencilJson, exportOpenPencilSvg, OPENPENCIL_MAX_INPUT, OPENPENCIL_MAX_OUTPUT, OPENPENCIL_MODE, OPENPENCIL_VERSION, patchOpenPencil } from "./openpencil-operations.js";
const PROTOCOL = 1, MAX_FRAME = 65536, MIN_DEADLINE = 100, MAX_DEADLINE = 60000;
const token = /^[A-Za-z0-9._:-]{1,256}$/u, revision = /^sha256:[a-f0-9]{64}$/u;
type Operation = "openpencil.create" | "openpencil.patch" | "openpencil.export_svg";
interface Request {
    type: "run";
    protocol_version: 1;
    release_id: string;
    job_id: string;
    job_root: string;
    input_path: string;
    source_revision: string;
    output_path: string;
    operation: Operation;
    deadline_ms: number;
    edits?: unknown;
    target_path?: string;
}
let releaseId = "invalid", jobId: string | null = null, outputMayExist = false, cancelled = false, deadlineAt = Infinity, abortRead: (() => void) | undefined;
function fail(code: string, message: string): never { throw new OpenPencilError(code, message); }
function check(): void { if (cancelled)
    fail("cancelled", "Operation was cancelled; inspect its stage before retrying."); if (Date.now() >= deadlineAt)
    fail("deadline_exceeded", "Operation exceeded its deadline; inspect its stage before retrying."); }
for (const signal of ["SIGINT", "SIGTERM"] as const)
    process.on(signal, () => { cancelled = true; abortRead?.(); });
process.stdout.on("error", () => { cancelled = true; process.exitCode = 1; abortRead?.(); });
function emit(message: object): void { const frame = JSON.stringify(message) + "\n"; if (Buffer.byteLength(frame) > MAX_FRAME)
    fail("invalid_frame", "Response exceeds frame limit."); process.stdout.write(frame); }
function readRequest(): Promise<unknown> { return new Promise((accept, reject) => { const chunks: Buffer[] = []; let bytes = 0, done = false; const finish = (error?: OpenPencilError): void => { if (done)
    return; done = true; clearTimeout(timer); abortRead = undefined; if (error) {
    process.stdin.destroy();
    reject(error);
    return;
} try {
    const all = Buffer.concat(chunks);
    if (!all.length || all[all.length - 1] !== 10 || all.subarray(0, -1).includes(10))
        throw new Error("frame");
    accept(decodeOpenPencilJson(all));
}
catch (e) {
    reject(e instanceof OpenPencilError ? e : new OpenPencilError("invalid_frame", "Expected one bounded UTF-8 JSON line and EOF."));
} }; const timer = setTimeout(() => finish(new OpenPencilError("request_timeout", "Request input deadline exceeded.")), 5000); abortRead = () => finish(new OpenPencilError("cancelled", "Operation was cancelled.")); process.stdin.on("data", (data: Buffer) => { if (done)
    return; bytes += data.length; if (bytes > MAX_FRAME) {
    finish(new OpenPencilError("invalid_frame", "Request exceeds frame limit."));
    return;
} chunks.push(data); }); process.stdin.on("end", () => finish()); process.stdin.on("error", () => finish(new OpenPencilError("invalid_frame", "Request input could not be read."))); }); }
function absolute(v: unknown): v is string { return typeof v === "string" && v.length <= 4096 && !v.includes("\0") && isAbsolute(v) && resolve(v) === v; }
function validate(v: unknown): Request {
    if (v === null || typeof v !== "object" || Array.isArray(v))
        fail("invalid_request", "Request must be an object.");
    const x = v as Record<string, unknown>;
    const base = ["type", "protocol_version", "release_id", "job_id", "job_root", "input_path", "source_revision", "output_path", "operation", "deadline_ms"];
    const fields = x.operation === "openpencil.patch" ? [...base, "edits"] : x.operation === "openpencil.export_svg" ? [...base, "target_path"] : base;
    if (Object.keys(x).length !== fields.length || !fields.every(k => Object.hasOwn(x, k)) || x.type !== "run" || x.protocol_version !== PROTOCOL || !["openpencil.create", "openpencil.patch", "openpencil.export_svg"].includes(x.operation as string) || typeof x.release_id !== "string" || !token.test(x.release_id) || typeof x.job_id !== "string" || !token.test(x.job_id) || !absolute(x.job_root) || !absolute(x.input_path) || !absolute(x.output_path) || typeof x.source_revision !== "string" || !revision.test(x.source_revision) || !Number.isInteger(x.deadline_ms) || typeof x.deadline_ms !== "number" || x.deadline_ms < MIN_DEADLINE || x.deadline_ms > MAX_DEADLINE)
        fail("invalid_request", "Request does not match the finite shape-file contract.");
    jobId = x.job_id as string;
    if (x.release_id !== releaseId)
        fail("release_mismatch", "Request release differs from this adapter.");
    const request = x as unknown as Request;
    if (dirname(request.input_path) !== request.job_root || dirname(request.output_path) !== request.job_root || request.input_path === request.output_path || extname(request.input_path) !== (request.operation === "openpencil.create" ? ".json" : ".fig") || extname(request.output_path) !== (request.operation === "openpencil.export_svg" ? ".svg" : ".fig") || (request.operation === "openpencil.export_svg" && typeof request.target_path !== "string"))
        fail("invalid_request", "Source and new stage must be format-matched private job siblings.");
    deadlineAt = Date.now() + request.deadline_ms;
    return request;
}
const hash = (bytes: Uint8Array): string => createHash("sha256").update(bytes).digest("hex");
async function source(request: Request): Promise<Uint8Array> {
    check();
    const root = await lstat(request.job_root);
    if (!root.isDirectory() || root.isSymbolicLink() || await realpath(request.job_root) !== request.job_root)
        fail("invalid_source", "Job root must be a canonical real directory.");
    const before = await lstat(request.input_path);
    if (!before.isFile() || before.isSymbolicLink() || before.size > OPENPENCIL_MAX_INPUT || await realpath(request.input_path) !== request.input_path)
        fail("invalid_source", "Source must be a bounded canonical regular file.");
    const file = await open(request.input_path, constants.O_RDONLY | (constants.O_NOFOLLOW ?? 0));
    const bytes = Buffer.alloc(OPENPENCIL_MAX_INPUT + 1);
    let length = 0;
    try {
        const opened = await file.stat();
        if (!opened.isFile() || opened.dev !== before.dev || opened.ino !== before.ino)
            fail("invalid_source", "Source changed while opening.");
        while (length < bytes.length) {
            check();
            const read = await file.read(bytes, length, bytes.length - length, length);
            if (!read.bytesRead)
                break;
            length += read.bytesRead;
        }
        const after = await file.stat();
        if (after.size !== before.size || after.mtimeMs !== before.mtimeMs || after.ctimeMs !== before.ctimeMs)
            fail("stale_source", "Source changed during read.");
    }
    finally {
        await file.close();
    }
    if (length > OPENPENCIL_MAX_INPUT)
        fail("input_too_large", "Source exceeds 1 MiB.");
    const result = bytes.subarray(0, length);
    if (`sha256:${hash(result)}` !== request.source_revision)
        fail("stale_source", "Source revision changed; inspect current source.");
    return result;
}
async function run(runtime: PencilRuntime, request: Request): Promise<object> {
    const input = await source(request);
    check();
    const result = request.operation === "openpencil.create" ? createOpenPencil(runtime, input) : request.operation === "openpencil.patch" ? patchOpenPencil(runtime, input, request.edits) : exportOpenPencilSvg(runtime, input, request.target_path!);
    check();
    if (result.output.length > OPENPENCIL_MAX_OUTPUT)
        fail("output_limit", "Output exceeds 16 MiB.");
    await source(request);
    check();
    const stage = await open(request.output_path, constants.O_WRONLY | constants.O_CREAT | constants.O_EXCL, 0o600);
    outputMayExist = true;
    try {
        check();
        await stage.writeFile(result.output);
        check();
        await stage.sync();
    }
    finally {
        await stage.close();
    }
    check();
    return { type: "result", protocol_version: PROTOCOL, release_id: releaseId, job_id: jobId, operation: request.operation, status: "staged", output_path: request.output_path, input_bytes: input.length, output_bytes: result.output.length, input_sha256: hash(input), output_sha256: hash(result.output), format: result.format, node_count: result.node_count, engine_version: result.engine_version, mode: result.mode };
}
async function main(): Promise<void> { try {
    const args = process.argv.slice(2);
    if (args.length !== 6 || args[0] !== "--release-id" || !args[1] || !token.test(args[1]) || args[2] !== "--runtime-root" || !absolute(args[3]) || args[4] !== "--runtime-sha256" || !args[5] || !/^[a-f0-9]{64}$/u.test(args[5]))
        fail("invalid_startup", "Supply exact release and private runtime identities.");
    releaseId = args[1];
    const runtime = await loadOpenPencilRuntime(args[3], args[5]);
    emit({ type: "ready", protocol_version: PROTOCOL, release_id: releaseId, operations: ["openpencil.create", "openpencil.patch", "openpencil.export_svg"], max_frame_bytes: MAX_FRAME, max_input_bytes: OPENPENCIL_MAX_INPUT, max_output_bytes: OPENPENCIL_MAX_OUTPUT, min_deadline_ms: MIN_DEADLINE, max_deadline_ms: MAX_DEADLINE, engine_version: OPENPENCIL_VERSION, mode: OPENPENCIL_MODE });
    const request = validate(await readRequest());
    emit(await run(runtime, request));
}
catch (e: unknown) {
    const error = e instanceof OpenPencilError ? e : e !== null && typeof e === "object" && "code" in e && e.code === "EEXIST" ? new OpenPencilError("output_exists", "Output stage already exists.") : new OpenPencilError("io_error", "Shape-file operation failed; inspect its stage before retrying.");
    emit({ type: "error", protocol_version: PROTOCOL, release_id: releaseId, job_id: jobId, code: error.code, message: error.message.slice(0, 256), output_may_exist: outputMayExist });
    process.exitCode = 1;
} }
void main();
