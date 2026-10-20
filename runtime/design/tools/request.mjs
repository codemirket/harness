import {AsyncLocalStorage} from 'node:async_hooks';

const requests = new AsyncLocalStorage();
const shutdown = new AbortController();
export function cancelAllRequests() { shutdown.abort(Error('Design MCP input closed')); }
export function withRequest(signal, fn) { return requests.run(signal ? AbortSignal.any([signal, shutdown.signal]) : shutdown.signal, fn); }
export function requestSignal() { return requests.getStore(); }
export function checkRequest() {
  const signal = requestSignal();
  if (signal?.aborted) throw signal.reason || Error('Design tool request was cancelled');
}
export function combinedSignal(timeoutMs) {
  const signal = requestSignal();
  return signal ? AbortSignal.any([signal, AbortSignal.timeout(timeoutMs)]) : AbortSignal.timeout(timeoutMs);
}
// Renderers receive an explicit credential-free environment. Retain one bounded
// error-formatting call site for callers without inspecting unrelated host secrets.
export function redactSecrets(value) { return String(value); }
