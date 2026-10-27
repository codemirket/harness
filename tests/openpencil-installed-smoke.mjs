// Disposable macOS ARM64 installed OpenPencil release and real MCP journey.
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { chmod, copyFile, cp, mkdir, mkdtemp, readFile, rm, stat, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { isAbsolute, join } from "node:path";

const flags = new Map();
for (let i = 2; i < process.argv.length; i += 2) {
  assert.ok(process.argv[i]?.startsWith("--") && process.argv[i + 1] && !flags.has(process.argv[i]));
  flags.set(process.argv[i], process.argv[i + 1]);
}
const required = ["--mirket", "--node", "--node-license", "--adapter-dir", "--openpencil-runtime"];
assert.deepEqual([...flags.keys()].sort(), [...required].sort());
for (const flag of required) assert.ok(isAbsolute(flags.get(flag)), `Expected absolute ${flag}`);
for (const flag of required.slice(0, 3)) assert.ok((await stat(flags.get(flag))).isFile());
for (const flag of required.slice(3)) assert.ok((await stat(flags.get(flag))).isDirectory());
assert.equal(process.platform, "darwin"); assert.equal(process.arch, "arm64");

const root = await mkdtemp(join(tmpdir(), "mirket-openpencil-installed-"));
const source = join(root, "source"), home = join(root, "home");
let passed = false;
function cli(...command) {
  return JSON.parse(execFileSync(flags.get("--mirket"), command,
    { encoding: "utf8", timeout: 30000, stdio: ["ignore", "pipe", "pipe"] }));
}
try {
  for (const directory of [source, join(source, "bin"), join(source, "adapters"),
    join(source, "runtime"), join(source, "runtime/node"), join(source, "runtime/node/bin")]) {
    await mkdir(directory, { mode: 0o700 });
  }
  for (const [input, relative, mode] of [
    [flags.get("--mirket"), "bin/mirket", 0o700],
    [flags.get("--node"), "runtime/node/bin/node", 0o700],
    [flags.get("--node-license"), "runtime/node/LICENSE", 0o600],
  ]) {
    const output = join(source, relative);
    await copyFile(input, output); await chmod(output, mode);
  }
  for (const name of ["openpencil-main.js", "openpencil-operations.js",
    "openpencil-runtime.js", "openpencil-inventory.js"]) {
    const input = join(flags.get("--adapter-dir"), name);
    assert.ok((await stat(input)).isFile(), `Missing compiled adapter ${name}`);
    const output = join(source, "adapters", name);
    await copyFile(input, output); await chmod(output, 0o600);
  }
  await cp(flags.get("--openpencil-runtime"), join(source, "runtime/openpencil"), { recursive: true });
  const manifested = cli("dev", "manifest", "--directory", source);
  const release = manifested.release_id;
  assert.match(release, /^sha256:[a-f0-9]{64}$/);
  const manifest = JSON.parse(await readFile(join(source, "manifest.json"), "utf8"));
  assert.deepEqual(manifest.components, ["open_pencil"]);
  const setup = cli("dev", "setup-core", "--home", home, "--source", source);
  assert.equal(setup.activation.status, "active");
  assert.equal(cli("dev", "doctor-core", "--home", home, "--json").status, "healthy");
  const result = JSON.parse(execFileSync(process.execPath, [
    join(import.meta.dirname, "openpencil-mcp-smoke.mjs"), "--home", home, "--release-id", release,
  ], { encoding: "utf8", timeout: 90_000, stdio: ["ignore", "pipe", "pipe"] }));
  assert.equal(result.status, "pass");
  assert.equal(result.installation, "disposable_active_release");
  assert.equal(cli("dev", "doctor-core", "--home", home, "--json").status, "healthy");
  const catalogTest = join(import.meta.dirname, "deferred-catalog-smoke.mjs");
  const healthyCatalog = JSON.parse(execFileSync(process.execPath, [catalogTest,
    "--home", home, "--release-id", release, "--expect", "ready",
  ], { encoding: "utf8", timeout: 45_000, stdio: ["ignore", "pipe", "pipe"] }));
  assert.equal(healthyCatalog.status, "pass");
  const installedAdapter = join(home, "releases", `r-${release.slice(7)}`, "adapters/openpencil-main.js");
  await writeFile(installedAdapter, Buffer.concat([await readFile(installedAdapter), Buffer.from("\n// disposable tamper\n")]));
  const refusedCatalog = JSON.parse(execFileSync(process.execPath, [catalogTest,
    "--home", home, "--release-id", release, "--expect", "verification_failure",
  ], { encoding: "utf8", timeout: 45_000, stdio: ["ignore", "pipe", "pipe"] }));
  assert.equal(refusedCatalog.status, "pass");
  passed = true;
  console.log(JSON.stringify({ status: "pass", platform: process.platform, arch: process.arch,
    release_id: release, component: "open_pencil", output: result,
    early_catalog: { healthy: healthyCatalog, tampered: refusedCatalog } }));
} finally {
  if (passed) await rm(root, { recursive: true, force: true });
  else console.error(`Retained failed OpenPencil installed fixture: ${root}`);
}
