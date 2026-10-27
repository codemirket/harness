/** One finite navigation request. Project paths and configuration never enter this leaf. */
import { navigateCode, type NavigationRequest } from "./code-navigation.js";

const MAX_REQUEST = 2 * 1024 * 1024, MAX_RESPONSE = 65_536;
const identity = /^[A-Za-z0-9._:-]{1,256}$/;
const encoder = new TextEncoder();
let release = "invalid", job: string | null = null;
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
    const chunks: Uint8Array[] = []; let length = 0, done = false;
    const finish = (code?: string): void => {
      if (done) return;
      done = true; clearTimeout(timer); abortRead = undefined;
      if (code) { process.stdin.destroy(); reject(new ProtocolError(code)); return; }
      try {
        const bytes = new Uint8Array(length); let offset = 0;
        for (const chunk of chunks) { bytes.set(chunk, offset); offset += chunk.length; }
        if (bytes.at(-1) !== 10 || bytes.subarray(0, -1).includes(10)) throw new Error("framing");
        accept(JSON.parse(new TextDecoder("utf-8", { fatal: true, ignoreBOM: true }).decode(bytes)));
      } catch { reject(new ProtocolError("invalid_frame")); }
    };
    const timer = setTimeout(() => finish("request_timeout"), 5000);
    abortRead = () => finish("cancelled");
    process.stdin.on("data", (chunk: Uint8Array) => {
      if (done) return;
      length += chunk.length; if (length > MAX_REQUEST) { finish("request_limit"); return; } chunks.push(chunk);
    });
    process.stdin.on("end", () => finish()); process.stdin.on("error", () => finish("invalid_frame"));
  });
}
async function main(): Promise<void> {
  try {
    const args = process.argv.slice(2);
    if (args.length !== 2 || args[0] !== "--release-id" || !identity.test(args[1] ?? "")) throw new ProtocolError("invalid_startup");
    release = args[1]!;
    const value = await readRequest();
    if (!value || typeof value !== "object" || Array.isArray(value) || Object.keys(value).length !== 4 ||
        !["protocol_version", "release_id", "job_id", "request"].every((key) => Object.hasOwn(value, key))) throw new ProtocolError("invalid_request");
    const request = value as Record<string, unknown>;
    if (request.protocol_version !== 1 || request.release_id !== release || typeof request.job_id !== "string" ||
        !identity.test(request.job_id)) throw new ProtocolError("invalid_request");
    job = request.job_id;
    emit({ protocol_version: 1, release_id: release, job_id: job, result: navigateCode(request.request as NavigationRequest) });
  } catch (error: unknown) {
    emit({ protocol_version: 1, release_id: release, job_id: job, error: error instanceof ProtocolError ? error.code : "navigation_failed" });
    process.exitCode = 1; process.stdin.destroy();
  }
}
void main();
