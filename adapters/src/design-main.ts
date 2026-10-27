import { createHash } from "node:crypto";
import { lstat, open, realpath } from "node:fs/promises";
import { basename, dirname, isAbsolute, resolve } from "node:path";
import { pathToFileURL } from "node:url";
import {
  compileTokens,
  DesignOperationError,
  MAX_INPUT_BYTES,
  MAX_OUTPUT_BYTES,
  optimizeSvg,
} from "./design-operations.js";

const PROTOCOL = 1;
const MAX_FRAME = 65_536;
const MAX_RASTER_INPUT_BYTES = 16 * 1024 * 1024;
const MAX_DIMENSION = 2048;
const MAX_DEADLINE_MS = 120_000;
const MIN_DEADLINE_MS = 100;
const encoder = new TextEncoder();
const token = /^[A-Za-z0-9._:-]{1,256}$/u;

type RasterFormat = "png" | "jpg" | "webp" | "avif";
type DesignOperation = "tokens.compile" | "svg.optimize" | "raster.transform";

interface RunRequest {
  type: "run";
  protocol_version: 1;
  release_id: string;
  job_id: string;
  input_path: string;
  output_path: string;
  operation: DesignOperation;
  format?: "css" | "json";
  source_format?: RasterFormat;
  output_format?: RasterFormat;
  width?: number;
  height?: number;
  fit?: "fill" | "inside";
  deadline_ms: number;
}

class AdapterError extends Error {
  constructor(readonly code: string, message: string) {
    super(message);
  }
}

let releaseId = "invalid";
let jobId: string | null = null;
let outputMayExist = false;
let cancelled = false;
let deadlineAt = Number.POSITIVE_INFINITY;
let rasterModule: string | undefined;
let abortRead: (() => void) | undefined;

function checkCancelled(): void {
  if (cancelled) throw new AdapterError("cancelled", "Operation was cancelled; inspect its stage before retrying.");
  if (Date.now() >= deadlineAt) {
    cancelled = true;
    throw new AdapterError("deadline_exceeded", "Operation exceeded its bounded deadline; inspect its stage before retrying.");
  }
}

for (const signal of ["SIGINT", "SIGTERM"] as const) {
  process.on(signal, () => {
    cancelled = true;
    abortRead?.();
  });
}
process.stdout.on("error", () => {
  cancelled = true;
  process.exitCode = 1;
  abortRead?.();
});

function emit(message: object): void {
  const frame = JSON.stringify(message) + "\n";
  if (encoder.encode(frame).length > MAX_FRAME) throw new AdapterError("invalid_frame", "Response exceeds frame limit.");
  process.stdout.write(frame);
}

function readRequest(): Promise<unknown> {
  return new Promise((accept, reject) => {
    const chunks: Uint8Array[] = [];
    let bytes = 0;
    let done = false;
    const finish = (error?: AdapterError): void => {
      if (done) return;
      done = true;
      clearTimeout(timer);
      abortRead = undefined;
      if (error) {
        process.stdin.destroy();
        reject(error);
        return;
      }
      try {
        const data = new Uint8Array(bytes);
        let offset = 0;
        for (const chunk of chunks) {
          data.set(chunk, offset);
          offset += chunk.length;
        }
        if (data.length === 0 || data[data.length - 1] !== 10 || data.subarray(0, -1).includes(10)) {
          throw new Error("framing");
        }
        const json = new TextDecoder("utf-8", { fatal: true }).decode(data);
        accept(JSON.parse(json));
      } catch {
        reject(new AdapterError("invalid_frame", "Expected one bounded UTF-8 JSON line followed by EOF."));
      }
    };
    const timer = setTimeout(() => finish(new AdapterError("request_timeout", "Request did not finish before its deadline.")), 5_000);
    abortRead = () => finish(new AdapterError("cancelled", "Operation was cancelled; inspect its stage before retrying."));
    process.stdin.on("data", (chunk) => {
      if (done) return;
      bytes += chunk.length;
      if (bytes > MAX_FRAME) {
        finish(new AdapterError("invalid_frame", "Request exceeds frame limit."));
        return;
      }
      chunks.push(chunk);
    });
    process.stdin.on("end", () => finish());
    process.stdin.on("error", () => finish(new AdapterError("invalid_frame", "Request input could not be read.")));
  });
}

function validPath(value: unknown): value is string {
  return typeof value === "string" && value.length <= 4096 && !value.includes("\0") && isAbsolute(value);
}

function validDeadline(value: unknown): value is number {
  return typeof value === "number" && Number.isInteger(value) && value >= MIN_DEADLINE_MS && value <= MAX_DEADLINE_MS;
}

function validate(value: unknown): RunRequest {
  const invalid = () => new AdapterError("invalid_request", "Request fields are invalid.");
  if (value === null || typeof value !== "object" || Array.isArray(value)) throw invalid();
  const v = value as Record<string, unknown>;
  const baseKeys = ["type", "protocol_version", "release_id", "job_id", "input_path", "output_path", "operation", "deadline_ms"];
  const keys = v.operation === "tokens.compile" ? [...baseKeys, "format"] :
    v.operation === "raster.transform" ? [...baseKeys, "source_format", "output_format", "width", "height", "fit"] : baseKeys;
  if (Object.keys(v).length !== keys.length || !keys.every((key) => Object.hasOwn(v, key))) throw invalid();
  if (v.type !== "run" || v.protocol_version !== PROTOCOL ||
      (v.operation !== "tokens.compile" && v.operation !== "svg.optimize" && v.operation !== "raster.transform") ||
      typeof v.release_id !== "string" || !token.test(v.release_id) ||
      typeof v.job_id !== "string" || !token.test(v.job_id) ||
      !validPath(v.input_path) || !validPath(v.output_path) ||
      !validDeadline(v.deadline_ms) ||
      (v.operation === "tokens.compile" && v.format !== "css" && v.format !== "json") ||
      (v.operation === "raster.transform" &&
        (rasterModule === undefined || !["png", "jpg", "webp", "avif"].includes(v.source_format as string) ||
         !["png", "jpg", "webp", "avif"].includes(v.output_format as string) ||
         typeof v.width !== "number" || !Number.isInteger(v.width) || v.width < 1 || v.width > MAX_DIMENSION ||
         typeof v.height !== "number" || !Number.isInteger(v.height) || v.height < 1 || v.height > MAX_DIMENSION ||
         v.width * v.height > MAX_DIMENSION * MAX_DIMENSION ||
         (v.fit !== "fill" && v.fit !== "inside")))) throw invalid();
  jobId = v.job_id;
  if (v.release_id !== releaseId) throw new AdapterError("release_mismatch", "Request release does not match this adapter.");
  if (resolve(v.input_path) === resolve(v.output_path)) throw invalid();
  deadlineAt = Date.now() + v.deadline_ms;
  return v as unknown as RunRequest;
}

async function readInput(path: string, limit: number): Promise<Uint8Array> {
  checkCancelled();
  const sourceMetadata = await lstat(path);
  if (!sourceMetadata.isFile() || sourceMetadata.isSymbolicLink() || sourceMetadata.size > limit) {
    throw new AdapterError("invalid_source", "Source must be a regular file within the byte limit.");
  }
  const input = await realpath(path);
  const source = await open(input, "r");
  const buffer = new Uint8Array(limit + 1);
  let inputBytes = 0;
  try {
    if (!(await source.stat()).isFile()) throw new AdapterError("invalid_source", "Source must be a regular file within the byte limit.");
    while (inputBytes < buffer.length) {
      checkCancelled();
      const result = await source.read(buffer, inputBytes, buffer.length - inputBytes, inputBytes);
      if (result.bytesRead === 0) break;
      inputBytes += result.bytesRead;
    }
  } finally {
    await source.close();
  }
  if (inputBytes > limit) throw new AdapterError("input_too_large", "Source exceeds its operation byte limit.");
  return buffer.subarray(0, inputBytes);
}

interface RasterResult {
  output: Uint8Array;
  format: RasterFormat;
  width: number;
  height: number;
}

interface RasterPipeline {
  metadata(): Promise<{ format?: string; compression?: string; width?: number; height?: number; pages?: number }>;
  rotate(): RasterPipeline;
  resize(width: number, height: number, options: object): RasterPipeline;
  toColourspace(space: string): RasterPipeline;
  flatten(options: object): RasterPipeline;
  png(options: object): RasterPipeline;
  jpeg(options: object): RasterPipeline;
  webp(options: object): RasterPipeline;
  avif(options: object): RasterPipeline;
  toBuffer(options: { resolveWithObject: true }): Promise<{
    data: Uint8Array;
    info: { format: string; width: number; height: number };
  }>;
}

type RasterFactory = (input: Uint8Array, options: object) => RasterPipeline;

async function transformRaster(source: Uint8Array, request: RunRequest): Promise<RasterResult> {
  if (rasterModule === undefined) throw new AdapterError("raster_unavailable", "Private raster module is not configured.");
  const sourceFormat = request.source_format;
  const outputFormat = request.output_format;
  const width = request.width;
  const height = request.height;
  const fit = request.fit;
  if (!sourceFormat || !outputFormat || !width || !height || !fit) throw new AdapterError("invalid_request", "Raster transform is incomplete.");
  let sharp: RasterFactory;
  try {
    const loaded = await import(pathToFileURL(rasterModule).href);
    sharp = loaded.default as RasterFactory;
    if (typeof sharp !== "function") throw new Error("module");
  } catch {
    throw new AdapterError("raster_unavailable", "Selected private raster module could not be loaded.");
  }
  try {
    const image = sharp(source, { failOn: "error", limitInputPixels: 16_777_216, sequentialRead: true });
    const metadata = await image.metadata();
    const actualFormat = metadata.format === "jpeg" ? "jpg" :
      metadata.format === "heif" && metadata.compression === "av1" ? "avif" : metadata.format;
    if (actualFormat !== sourceFormat || !metadata.width || !metadata.height ||
        metadata.width > 8192 || metadata.height > 8192 || (metadata.pages ?? 1) !== 1) {
      throw new AdapterError("raster_invalid", "Source raster format or dimensions differ from the request.");
    }
    let transformed = image.rotate().resize(width, height, {
      fit,
      withoutEnlargement: fit === "inside",
      kernel: "lanczos3",
    }).toColourspace("srgb");
    switch (outputFormat) {
      case "png": transformed = transformed.png({ compressionLevel: 9 }); break;
      case "jpg": transformed = transformed.flatten({ background: "#ffffff" }).jpeg({ quality: 90, mozjpeg: true }); break;
      case "webp": transformed = transformed.webp({ quality: 85, effort: 4, alphaQuality: 100 }); break;
      case "avif": transformed = transformed.avif({ quality: 65, effort: 4, chromaSubsampling: "4:2:0" }); break;
    }
    const result = await transformed.toBuffer({ resolveWithObject: true });
    const actualOutput = result.info.format === "jpeg" ? "jpg" :
      result.info.format === "heif" && outputFormat === "avif" ? "avif" : result.info.format;
    if (actualOutput !== outputFormat || !result.info.width || !result.info.height ||
        (outputFormat === "avif" && new TextDecoder("ascii").decode(result.data.subarray(4, 12)) !== "ftypavif") ||
        (fit === "fill" && (result.info.width !== width || result.info.height !== height)) ||
        (fit === "inside" && (result.info.width > width || result.info.height > height))) {
      throw new AdapterError("raster_invalid", "Raster output did not match the requested format and dimensions.");
    }
    return { output: result.data, format: outputFormat, width: result.info.width, height: result.info.height };
  } catch (error) {
    if (error instanceof AdapterError) throw error;
    throw new AdapterError("raster_invalid", "Source raster could not be decoded or transformed.");
  }
}

function digest(value: Uint8Array): string {
  return createHash("sha256").update(value).digest("hex");
}

async function withDeadline<T>(operation: Promise<T>): Promise<T> {
  const remaining = deadlineAt - Date.now();
  if (remaining <= 0) throw new AdapterError("deadline_exceeded", "Operation exceeded its bounded deadline; inspect its stage before retrying.");
  let timer: ReturnType<typeof setTimeout> | undefined;
  const timeout = new Promise<never>((_, reject) => {
    timer = setTimeout(() => {
      cancelled = true;
      reject(new AdapterError("deadline_exceeded", "Operation exceeded its bounded deadline; inspect its stage before retrying."));
    }, remaining);
  });
  try {
    return await Promise.race([operation, timeout]);
  } finally {
    if (timer !== undefined) clearTimeout(timer);
  }
}

async function run(request: RunRequest): Promise<object> {
  checkCancelled();
  const source = await readInput(request.input_path, request.operation === "raster.transform" ? MAX_RASTER_INPUT_BYTES : MAX_INPUT_BYTES);
  const inputSha256 = digest(source);
  const result = request.operation === "tokens.compile"
    ? await withDeadline(compileTokens(source, request.format as "css" | "json"))
    : request.operation === "svg.optimize"
      ? await withDeadline(Promise.resolve(optimizeSvg(source)))
      : await withDeadline(transformRaster(source, request));
  checkCancelled();
  if (result.output.byteLength > MAX_OUTPUT_BYTES) throw new AdapterError("output_limit", "Generated output exceeds the 16 MiB limit.");

  const parent = await lstat(dirname(request.output_path));
  if (!parent.isDirectory() || parent.isSymbolicLink()) throw new AdapterError("invalid_request", "Output parent must be an existing real directory.");
  const output = resolve(await realpath(dirname(request.output_path)), basename(request.output_path));
  if (resolve(request.input_path) === output) throw new AdapterError("invalid_request", "Source and stage must differ.");
  checkCancelled();
  const stage = await open(request.output_path, "wx", 0o600);
  outputMayExist = true;
  try {
    checkCancelled();
    await stage.writeFile(result.output);
    checkCancelled();
    await stage.sync();
  } finally {
    await stage.close();
  }
  checkCancelled();
  const common = {
    type: "result",
    protocol_version: PROTOCOL,
    release_id: releaseId,
    job_id: jobId,
    operation: request.operation,
    status: "staged",
    output_path: request.output_path,
    input_bytes: source.byteLength,
    output_bytes: result.output.byteLength,
    input_sha256: inputSha256,
    output_sha256: digest(result.output),
  };
  if (request.operation === "tokens.compile") {
    const tokenResult = result as Awaited<ReturnType<typeof compileTokens>>;
    return { ...common, format: tokenResult.format, token_count: tokenResult.token_count, max_depth: tokenResult.max_depth };
  }
  if (request.operation === "raster.transform") {
    const rasterResult = result as RasterResult;
    return { ...common, format: rasterResult.format, width: rasterResult.width, height: rasterResult.height };
  }
  const svgResult = result as ReturnType<typeof optimizeSvg>;
  return { ...common, format: svgResult.format, element_count: svgResult.element_count };
}

async function main(): Promise<void> {
  try {
    const args = process.argv.slice(2);
    if ((args.length !== 2 && args.length !== 4) || args[0] !== "--release-id" || !args[1] || !token.test(args[1]) ||
        (args.length === 4 && (args[2] !== "--raster-module" || !validPath(args[3]) || !args[3].endsWith(".mjs")))) {
      throw new AdapterError("invalid_startup", "Supply an explicit valid release identity.");
    }
    releaseId = args[1];
    if (args.length === 4) {
      const selected = args[3] as string;
      const metadata = await lstat(selected);
      if (!metadata.isFile() || metadata.isSymbolicLink() || await realpath(selected) !== selected) {
        throw new AdapterError("invalid_startup", "Raster module must be a canonical regular file.");
      }
      rasterModule = selected;
    }
    emit({
      type: "ready",
      protocol_version: PROTOCOL,
      release_id: releaseId,
      operations: rasterModule === undefined ? ["tokens.compile", "svg.optimize"] : ["tokens.compile", "svg.optimize", "raster.transform"],
      max_frame_bytes: MAX_FRAME,
      max_input_bytes: rasterModule === undefined ? MAX_INPUT_BYTES : MAX_RASTER_INPUT_BYTES,
      max_output_bytes: MAX_OUTPUT_BYTES,
      min_deadline_ms: MIN_DEADLINE_MS,
      max_deadline_ms: MAX_DEADLINE_MS,
    });
    const request = validate(await readRequest());
    emit(await run(request));
  } catch (error: unknown) {
    const failure = error instanceof AdapterError ? error :
      error instanceof DesignOperationError ? new AdapterError(error.code, error.message.slice(0, 256)) :
      error !== null && typeof error === "object" && "code" in error && error.code === "EEXIST" ?
        new AdapterError("output_exists", "The output stage already exists.") :
        new AdapterError("io_error", "File operation failed; inspect the stage before retrying.");
    emit({
      type: "error",
      protocol_version: PROTOCOL,
      release_id: releaseId,
      job_id: jobId,
      code: failure.code,
      message: failure.message,
      output_may_exist: outputMayExist,
    });
    process.exitCode = 1;
  }
}

void main();
