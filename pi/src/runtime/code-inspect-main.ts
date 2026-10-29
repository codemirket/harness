/** One finite private byte-only transport; no project paths or file access. */
import { createHash } from "node:crypto";
import { inspectCodeSource, CODE_SOURCE_LIMITS, type CodeSourceInspectRequest } from "./code-source-inspect.js";

const MAX_REQUEST = 2 * 1024 * 1024;
const MAX_RESPONSE = 65_536;
const identity = /^[A-Za-z0-9._:-]{1,256}$/;
const encoder = new TextEncoder();
let release = "invalid";
let job: string | null = null;
let abortRead: (() => void) | undefined;
class ProtocolError extends Error { constructor(readonly code: string) { super(code); } }
function emit(value: object): void {
  const line = JSON.stringify(value) + "\n";
  if (encoder.encode(line).length > MAX_RESPONSE) throw new ProtocolError("response_limit");
  process.stdout.write(line);
}
for (const signal of ["SIGINT", "SIGTERM"] as const) process.on(signal, () => { abortRead?.(); process.exitCode = 1; });
process.stdout.on("error", () => { abortRead?.(); process.exitCode = 1; });
async function readRequest(): Promise<unknown> {
  return new Promise((accept, reject) => {
    const chunks: Uint8Array[] = [];
    let length = 0, done = false;
    const finish = (code?: string): void => {
      if (done) return;
      done = true; clearTimeout(timer); abortRead = undefined;
      if (code) { process.stdin.destroy(); reject(new ProtocolError(code)); return; }
      try {
        const source = new Uint8Array(length); let offset = 0;
        for (const chunk of chunks) { source.set(chunk,offset); offset += chunk.length; }
        if (source.at(-1) !== 10 || source.subarray(0,-1).includes(10)) throw new Error("framing");
        accept(JSON.parse(new TextDecoder("utf-8",{fatal:true,ignoreBOM:true}).decode(source)));
      } catch { reject(new ProtocolError("invalid_frame")); }
    };
    const timer = setTimeout(() => finish("request_timeout"),5000);
    abortRead = () => finish("cancelled");
    process.stdin.on("data",(chunk: Uint8Array) => {
      if (done) return;
      length += chunk.length;
      if (length > MAX_REQUEST) { finish("request_limit"); return; }
      chunks.push(chunk);
    });
    process.stdin.on("end",() => finish());
    process.stdin.on("error",() => finish("invalid_frame"));
  });
}
function object(value: unknown, keys: string[]): Record<string,unknown> {
  if (!value || typeof value !== "object" || Array.isArray(value) ||
      Object.keys(value).length !== keys.length || !keys.every((key) => Object.hasOwn(value,key))) throw new ProtocolError("invalid_request");
  return value as Record<string,unknown>;
}
async function main(): Promise<void> {
  try {
    const args = process.argv.slice(2);
    if (args.length !== 2 || args[0] !== "--release-id" || !identity.test(args[1] ?? "")) throw new ProtocolError("invalid_startup");
    release = args[1]!;
    const request = object(await readRequest(),["protocol_version","release_id","job_id","source_revision","source_utf8","request"]);
    if (request.protocol_version !== 1 || typeof request.release_id !== "string" || request.release_id !== release ||
        typeof request.job_id !== "string" || !identity.test(request.job_id) || typeof request.source_utf8 !== "string" ||
        typeof request.source_revision !== "string" || !/^sha256:[0-9a-f]{64}$/.test(request.source_revision)) throw new ProtocolError("invalid_request");
    job = request.job_id;
    const bytes = encoder.encode(request.source_utf8);
    if (bytes.length > CODE_SOURCE_LIMITS.source_bytes || new TextDecoder("utf-8",{ignoreBOM:true}).decode(bytes) !== request.source_utf8) throw new ProtocolError("invalid_source");
    if ("sha256:" + createHash("sha256").update(bytes).digest("hex") !== request.source_revision) throw new ProtocolError("source_revision_mismatch");
    const selection = object(request.request,["language","pointer","expected_revision","start_child","limit","max_response_bytes"]);
    if (selection.expected_revision !== request.source_revision) throw new ProtocolError("source_revision_mismatch");
    if (typeof selection.language !== "string" || !["js","ts","jsx","tsx"].includes(selection.language)) throw new ProtocolError("invalid_request");
    const result = inspectCodeSource(bytes,selection as unknown as CodeSourceInspectRequest);
    emit({protocol_version:1,release_id:release,job_id:job,result});
  } catch (error: unknown) {
    emit({protocol_version:1,release_id:release,job_id:job,error:error instanceof ProtocolError ? error.code : "inspection_failed"});
    process.exitCode = 1;
    process.stdin.destroy();
  }
}
void main();
