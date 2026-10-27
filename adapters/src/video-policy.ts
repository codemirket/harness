/** Explicit source-patch hooks for the exact HyperFrames candidate. This is an
 * application request policy and process-ownership aid, never an OS sandbox. */
import fs from "node:fs";
import path from "node:path";

export const VIDEO_POLICY_VERSION = 1;
const ownedOrigins = new Set<string>();
function root(): string {
  if (process.env.MIRKET_VIDEO_POLICY_VERSION !== "1") throw new Error("mirket_video_policy_not_selected");
  const cwd = process.cwd();
  if (fs.realpathSync(cwd) !== cwd) throw new Error("mirket_video_private_root_changed");
  return cwd;
}
export function localUrl(value: string): URL {
  const url = new URL(value);
  if (url.protocol !== "http:" || url.hostname !== "127.0.0.1" || url.username || url.password || !url.port) {
    throw new Error("mirket_video_remote_dependency");
  }
  return url;
}
export function mirketSourceFile(candidate: string, selectedRoot: string): boolean {
  root();
  try {
    return fs.realpathSync(selectedRoot) === selectedRoot && fs.realpathSync(candidate) === candidate
      && candidate.startsWith(selectedRoot + path.sep) && fs.lstatSync(candidate).isFile() && !fs.lstatSync(candidate).isSymbolicLink();
  } catch { return false; }
}
export async function mirketVideoFetch(input: string | URL | Request, init?: RequestInit): Promise<Response> {
  root();
  const url = localUrl(input instanceof Request ? input.url : String(input));
  if (!ownedOrigins.has(url.origin)) throw new Error("mirket_video_unowned_loopback_origin");
  const response = await globalThis.fetch(url, { ...init, redirect: "manual" });
  if (response.status >= 300 && response.status < 400) throw new Error("mirket_video_redirect_refused");
  return response;
}
export function mirketRegisterServer(value: string): void {
  root(); ownedOrigins.add(localUrl(value).origin);
}
export function mirketLaunchOptions(options: Record<string, unknown>): Record<string, unknown> {
  const cwd = root();
  const expected = process.env.MIRKET_VIDEO_CHROMIUM;
  if (!expected || options.executablePath !== expected) throw new Error("mirket_video_private_chromium_required");
  const args = (options.args as string[]).filter((arg) => !["--no-sandbox", "--disable-setuid-sandbox"].includes(arg) && !arg.startsWith("--user-data-dir="));
  args.push(`--crash-dumps-dir=${path.join(cwd, "crash")}`, "--disable-breakpad", "--disable-crash-reporter", "--noerrdialogs", "--disable-background-networking");
  return { ...options, args, userDataDir: path.join(cwd, "profile"), handleSIGINT: false, handleSIGTERM: false, handleSIGHUP: false };
}
interface RequestLike { url(): string; isInterceptResolutionHandled(): boolean; continue(): Promise<void>; abort(reason: string): Promise<void> }
interface PageLike {
  setRequestInterception(enabled: boolean): Promise<void>;
  setBypassServiceWorker(enabled: boolean): Promise<void>;
  on(name: string, listener: (request: RequestLike) => void): void;
  evaluateOnNewDocument(script: string): Promise<unknown>;
}
export async function mirketPreparePage(page: PageLike, serverUrl: string): Promise<void> {
  root(); const origin = localUrl(serverUrl).origin;
  await page.setBypassServiceWorker(true);
  await page.setRequestInterception(true);
  page.on("request", (request) => {
    if (request.isInterceptResolutionHandled()) return;
    const text = request.url(); let permitted = /^(data:|blob:|about:blank$)/.test(text);
    try { permitted ||= localUrl(text).origin === origin; } catch { /* deny */ }
    void (permitted ? request.continue() : request.abort("blockedbyclient")).catch(() => {});
  });
  await page.evaluateOnNewDocument(`(() => {
    for (const name of ['WebSocket','EventSource','Worker','SharedWorker']) {
      Object.defineProperty(globalThis, name, {value: class {constructor(){throw new Error('mirket_video_dynamic_network_refused')}}, writable:false, configurable:false});
    }
    Object.defineProperty(window, 'open', {value:()=>{throw new Error('mirket_video_popup_refused')}, writable:false, configurable:false});
  })()`);
}
