// Reference studies use the same Browserless service as production visual QA.
const { chromium } = require('playwright-core');
exports.connect = async function connect() {
  const endpoint = process.env.BROWSERLESS_WS;
  if (!endpoint) throw new Error('Set BROWSERLESS_WS to the managed Browserless WebSocket endpoint.');
  const url = new URL(endpoint);
  if (!['ws:', 'wss:'].includes(url.protocol)) throw new Error('Browserless requires a WebSocket URL.');
  url.pathname = '/chromium/playwright';
  return chromium.connect(url.toString(), {timeout: 30000});
};
