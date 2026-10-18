// Use an explicitly installed Chromium; never resolve or download a browser.
const { chromium } = require(process.env.MIRKET_PLAYWRIGHT_MODULE || 'playwright-core');
exports.connect = async function connect() {
  const executablePath = process.env.MIRKET_CHROMIUM_PATH;
  if (!executablePath || !require('node:path').isAbsolute(executablePath)) throw new Error('Set MIRKET_CHROMIUM_PATH to an installed Chromium executable.');
  return chromium.launch({executablePath, headless: true, timeout: 30000});
};
