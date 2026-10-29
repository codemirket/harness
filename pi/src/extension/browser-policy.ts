import { isAbsolute, join, resolve } from 'node:path';

export const BROWSER_TOOLS = Object.freeze(['browser_navigate', 'browser_snapshot', 'browser_take_screenshot', 'browser_resize', 'browser_press_key', 'browser_wait_for', 'browser_close'] as const);
export interface BrowserBindingIdentity {
  peerPath: string; peerIdentity: string; browserPath: string; browserIdentity: string;
}
export interface BrowserGrant extends BrowserBindingIdentity {
  project: string; release: string;
  origin: string; output: string; expires: number; maxCalls: number;
  methods: readonly string[]; tools: readonly string[];
}
export function assertBrowserBinding(expected: BrowserBindingIdentity, current: BrowserBindingIdentity): void {
  for (const key of ['peerPath', 'peerIdentity', 'browserPath', 'browserIdentity'] as const) {
    if (current[key] !== expected[key]) throw new Error(`Browser binding ${key} changed after authorization`);
  }
}
const exact = (value: unknown, keys: string[]): value is Record<string, unknown> => !!value && typeof value === 'object' && !Array.isArray(value) && Object.keys(value).every(key => keys.includes(key));
export function browserOrigin(value: string): string {
  const url = new URL(value);
  if (url.protocol !== 'http:' || url.hostname !== '127.0.0.1' || !url.port || url.username || url.password || url.origin !== value)
    throw new Error('Select one exact literal http://127.0.0.1:PORT origin');
  return url.origin;
}
export function createBrowserPolicy(grant: BrowserGrant, recheck: () => void, now = () => Date.now()) {
  const started = now();
  browserOrigin(grant.origin);
  if (![grant.peerPath, grant.browserPath].every(path => typeof path === 'string' && isAbsolute(path) && resolve(path) === path)) throw new Error('Browser grant requires exact absolute executable paths');
  if (JSON.stringify(grant.methods) !== JSON.stringify(['GET', 'HEAD']) || JSON.stringify(grant.tools) !== JSON.stringify(BROWSER_TOOLS)) throw new Error('Browser grant methods or tools differ from the admitted set');
  if (!Number.isSafeInteger(grant.expires) || grant.expires <= started || grant.expires > started + 300000 || !Number.isSafeInteger(grant.maxCalls) || grant.maxCalls < 1 || grant.maxCalls > 64)
    throw new Error('Browser grant exceeds its expiry or call bounds');
  let revoked = false, remaining = grant.maxCalls;
  return {
    revoke() { revoked = true; },
    get active() { return !revoked && now() < grant.expires && remaining > 0; },
    get remaining() { return remaining; },
    network(method: string, target: string) {
      try { const url = new URL(target); return !revoked && now() < grant.expires && grant.methods.includes(method) && url.origin === grant.origin && !url.username && !url.password; }
      catch { return false; }
    },
    check(tool: string, input: unknown, context: {project: string; release: string}): Record<string, unknown> {
      if (revoked || now() >= grant.expires || remaining < 1 || context.project !== grant.project || context.release !== grant.release)
        throw new Error('Browser authorization is missing, exhausted, expired or belongs to another project/release');
      if (!(BROWSER_TOOLS as readonly string[]).includes(tool)) throw new Error('Browser tool is outside the admitted seven-tool set');
      recheck();
      let args: Record<string, unknown>;
      if (tool === 'browser_navigate') {
        if (!exact(input, ['url']) || typeof input.url !== 'string' || input.url.length > 8192 || !this.network('GET', input.url)) throw new Error('Navigation leaves the authorized origin');
        args = {...input};
      } else if (tool === 'browser_take_screenshot') {
        if (!exact(input, ['type', 'scale', 'filename', 'fullPage']) || input.type !== 'png' || input.scale !== 'css' || (input.fullPage !== undefined && input.fullPage !== false)) throw new Error('Only viewport CSS-scale PNG screenshots are admitted; fullPage must be omitted or false');
        args = {...input};
        if (input.filename !== undefined) {
          if (typeof input.filename !== 'string' || !/^[-a-zA-Z0-9_]{1,80}\.png$/u.test(input.filename)) throw new Error('Screenshot filename must be a private-output PNG basename');
          args.filename = join(grant.output, input.filename);
        }
      } else if (tool === 'browser_resize') {
        if (!exact(input, ['width', 'height']) || !['width', 'height'].every(key => Number.isSafeInteger(input[key]) && Number(input[key]) >= 240 && Number(input[key]) <= 2560)) throw new Error('Viewport dimensions must be integers between 240 and 2560');
        args = {...input};
      } else if (tool === 'browser_press_key') {
        if (!exact(input, ['key']) || !['Tab', 'Shift+Tab', 'Enter', 'Space', 'Escape', 'ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight', 'Home', 'End', 'PageUp', 'PageDown'].includes(String(input.key))) throw new Error('Browser key is outside the scoped navigation key set');
        args = {...input};
      } else if (tool === 'browser_wait_for') {
        if (!exact(input, ['text', 'textGone']) || Object.keys(input).length !== 1 || !Object.values(input).every(value => typeof value === 'string' && value.length > 0 && value.length <= 200)) throw new Error('Wait requires one bounded observed text condition');
        args = {...input};
      } else {
        if (!exact(input, []) || Object.keys(input).length) throw new Error('This browser tool takes no arguments or file destinations');
        args = {};
      }
      remaining--;
      return args;
    },
  };
}
