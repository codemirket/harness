// Disposable core-only setup/update journey with a live MCP session.
import assert from "node:assert/strict";
import { spawn, execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { chmod, copyFile, mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, isAbsolute, join } from "node:path";
import { fileURLToPath } from "node:url";

const paths = new Map();
for (let i = 2; i < process.argv.length; i += 2) {
  const key = process.argv[i], value = process.argv[i + 1];
  assert.ok(["--mirket", "--node", "--node-license", "--codex"].includes(key) && value && isAbsolute(value) && !paths.has(key));
  paths.set(key, value);
}
assert.ok([3, 4].includes(paths.size) && ["--mirket", "--node", "--node-license"].every((key) => paths.has(key)),
  "Supply absolute --mirket, --node and --node-license paths; --codex is optional");
assert.equal(process.platform, "darwin", "This fixture currently qualifies the macOS installed layout");
assert.equal(process.arch, "arm64", "This fixture currently qualifies macOS ARM64 only");
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
const root = await mkdtemp(join(tmpdir(), "mirket-installed-maintenance-"));
const home = join(root, "home");
let server;
let passed = false;

function cli(...args) {
  return JSON.parse(execFileSync(paths.get("--mirket"), args,
    { encoding: "utf8", timeout: 30_000, stdio: ["ignore", "pipe", "pipe"] }));
}

async function payload(name, alterLicense) {
  const directory = join(root, name);
  for (const part of ["", "bin", "runtime", "runtime/node", "runtime/node/bin"]) {
    const path = join(directory, part);
    await mkdir(path, { mode: 0o700 });
    await chmod(path, 0o700);
  }
  for (const [key, relative, mode] of [
    ["--mirket", "bin/mirket", 0o700],
    ["--node", "runtime/node/bin/node", 0o700],
    ["--node-license", "runtime/node/LICENSE", 0o600],
  ]) {
    const destination = join(directory, relative);
    await copyFile(paths.get(key), destination);
    await chmod(destination, mode);
  }
  if (alterLicense) {
    const license = join(directory, "runtime/node/LICENSE");
    await writeFile(license, Buffer.concat([await readFile(license), Buffer.from("\nmaintenance fixture B\n")]));
    await chmod(license, 0o600);
  }
  return { directory, id: cli("dev", "manifest", "--directory", directory).release_id };
}

async function openSession() {
  server = spawn(join(home, "bin/mirket"), ["serve", "--home", home], { stdio: ["pipe", "pipe", "pipe"] });
  let buffer = "", stderr = "";
  server.stderr.setEncoding("utf8").on("data", (chunk) => { stderr += chunk; });
  const initialized = new Promise((resolve, reject) => {
    const timer = setTimeout(() => reject(new Error(`MCP initialization timed out: ${stderr}`)), 10_000);
    server.stdout.setEncoding("utf8").on("data", (chunk) => {
      buffer += chunk;
      const end = buffer.indexOf("\n");
      if (end < 0) return;
      clearTimeout(timer);
      try {
        const reply = JSON.parse(buffer.slice(0, end));
        assert.equal(reply.id, 1);
        assert.ok(reply.result, JSON.stringify(reply.error));
        resolve();
      } catch (error) { reject(error); }
    });
    server.once("error", reject);
    server.once("exit", (code) => reject(new Error(`MCP exited ${code}: ${stderr}`)));
  });
  server.stdin.write(JSON.stringify({ jsonrpc: "2.0", id: 1, method: "initialize", params: {
    protocolVersion: "2025-11-25", capabilities: {}, clientInfo: { name: "installed-maintenance", version: "0.0.1" },
  } }) + "\n");
  await initialized;
}

async function closeSession() {
  if (!server) return;
  const child = server;
  server = undefined;
  const closed = new Promise((resolve, reject) => {
    const timer = setTimeout(() => { child.kill("SIGKILL"); reject(new Error("MCP shutdown timed out")); }, 5000);
    child.once("close", (code, signal) => {
      clearTimeout(timer);
      if (code === 0 && signal === null) resolve();
      else reject(new Error(`MCP shutdown ${code ?? signal}`));
    });
  });
  child.stdin.end();
  await closed;
}

try {
  const a = await payload("payload-a", false);
  const b = await payload("payload-b", true);
  assert.notEqual(a.id, b.id);
  const setup = cli("dev", "setup-core", "--home", home, "--source", a.directory);
  assert.equal(setup.activation.status, "active");
  assert.equal(setup.activation.release_id, a.id);
  assert.equal(cli("dev", "doctor-core", "--home", home, "--json").status, "healthy");
  const originalLauncher = hash(await readFile(join(home, "bin/mirket")));

  await openSession();
  const busy = cli("dev", "update-core", "--home", home, "--source", b.directory);
  assert.equal(busy.activation.status, "staged");
  assert.equal(cli("dev", "doctor-core", "--home", home, "--json").active_release_id, a.id);
  assert.equal(hash(await readFile(join(home, "bin/mirket"))), originalLauncher);
  await closeSession();

  const updated = cli("dev", "update-core", "--home", home, "--source", b.directory);
  assert.equal(updated.activation.status, "active");
  assert.equal(updated.activation.release_id, b.id);
  const doctor = cli("dev", "doctor-core", "--home", home, "--json");
  assert.equal(doctor.status, "healthy");
  assert.equal(doctor.active_release_id, b.id);
  assert.equal(doctor.launcher_healthy, true);
  assert.equal(hash(await readFile(join(home, "bin/mirket"))), originalLauncher);
  if (paths.has("--codex")) {
    const smoke = join(dirname(fileURLToPath(import.meta.url)), "codex-app-server-smoke.mjs");
    execFileSync(paths.get("--node"), [smoke, "--codex", paths.get("--codex"),
      "--mirket", join(home, "bin/mirket"), "--home", home,
      "--registration-mode", "managed"], { stdio: "inherit", timeout: 90_000 });
  }
  const launcher = join(home, "bin/mirket");
  await writeFile(launcher, Buffer.from("changed managed launcher\n"));
  await chmod(launcher, 0o700);
  assert.throws(() => cli("dev", "update-core", "--home", home, "--source", a.directory),
    /existing core installation is not healthy/);
  assert.throws(() => cli("codex-doctor", "--home", home, "--config", join(root, "config.toml")),
    /installation health check failed/);
  assert.equal((await readFile(launcher)).toString(), "changed managed launcher\n");
  assert.equal(JSON.parse(await readFile(join(home, "active.json"), "utf8")).release_id, b.id);
  passed = true;
  console.log(`PASS installed setup, busy staging, idle activation, launcher health and drift refusal (${a.id} → ${b.id})`);
} finally {
  if (server) {
    server.kill("SIGKILL");
    await new Promise((resolve) => server.once("close", resolve));
  }
  if (passed) await rm(root, { recursive: true, force: true });
  else console.error(`Retained failed installed-maintenance fixture: ${root}`);
}
