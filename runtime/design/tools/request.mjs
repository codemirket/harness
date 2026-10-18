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
export function redactSecrets(value) {
  let output = String(value);
  for (const name of ['MIRKET_PENPOT_PASSWORD', 'MIRKET_PENPOT_EMAIL']) {
    const secret = process.env[name];
    if (secret && secret.length >= 4) output = output.split(secret).join('[redacted]');
  }
  return output;
}
