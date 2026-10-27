#!/usr/bin/env node
// Bounded, no-model-turn qualification of the Codex-native model controls used
// by motion authoring. The app-server protocol exposes selected model/effort
// and effective config without touching the user's Codex or Mirket homes.
import assert from "node:assert/strict";
import { execFile } from "node:child_process";
import { promisify } from "node:util";
import { createHash } from "node:crypto";
import { access, chmod, mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { spawn } from "node:child_process";

const execFileAsync = promisify(execFile);
const ROOT_MODEL = "gpt-6-astra";
const ROOT_EFFORT = "high";
const LOWER_MODEL = "gpt-6-luna";
const LOWER_EFFORT = "low";

function parseFlags(argv) {
  const flags = new Map();
  for (let i = 0; i < argv.length; i += 2) {
    const key = argv[i];
    const value = argv[i + 1];
    assert.ok(key === "--codex" || key === "--evidence-root", `unknown argument: ${key}`);
    assert.ok(value && !flags.has(key), `missing or repeated value for ${key}`);
    flags.set(key, value);
  }
  return flags;
}

function sha256(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

async function spawnServer({ codex, home, project, config, sessionFlags = [] }) {
  await mkdir(home, { recursive: true, mode: 0o700 });
  await chmod(home, 0o700);
  await writeFile(join(home, "config.toml"), config, { mode: 0o600 });
  const child = spawn(codex, [
    "app-server", "--strict-config", "--listen", "stdio://", ...sessionFlags,
  ], {
    cwd: project,
    env: { ...process.env, CODEX_HOME: home, HOME: home },
    stdio: ["pipe", "pipe", "pipe"],
  });
  child.stdout.setEncoding("utf8");
  child.stderr.setEncoding("utf8");
  let stream = "";
  let stderr = "";
  let spawnError = null;
  let closed = false;
  let nextId = 0;
  const pending = new Map();
  const events = [];
  const closePromise = new Promise((resolve) => child.once("close", (code, signal) => {
    closed = true;
    resolve({ code, signal });
  }));
  child.stdout.on("data", (data) => {
    stream += data;
    assert.ok(stream.length < 4 * 1024 * 1024, "app-server stream exceeded bounded fixture size");
    for (let index; (index = stream.indexOf("\n")) >= 0;) {
      const line = stream.slice(0, index);
      stream = stream.slice(index + 1);
      if (!line.trim()) continue;
      const message = JSON.parse(line);
      if (message.id != null && pending.has(message.id)) pending.get(message.id).resolve(message);
      else events.push(message);
    }
  });
  child.stderr.on("data", (data) => { stderr = (stderr + data).slice(-32_768); });
  child.on("error", (error) => {
    spawnError = error;
    stderr = (stderr + `\nprocess error: ${error.message}`).slice(-32_768);
    for (const [id, entry] of pending) {
      clearTimeout(entry.timer);
      pending.delete(id);
      entry.reject(error);
    }
  });

  async function request(method, params) {
    const id = ++nextId;
    if (spawnError) throw spawnError;
    const reply = new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        pending.delete(id);
        reject(new Error(`${method} timed out; stderr: ${stderr.slice(-1000)}`));
      }, 5_000);
      pending.set(id, { timer, resolve: (message) => { clearTimeout(timer); pending.delete(id); resolve(message); }, reject });
    });
    child.stdin.write(JSON.stringify({ id, method, params }) + "\n");
    return reply;
  }

  async function close() {
    if (!closed) {
      child.stdin.end();
      const exited = await Promise.race([
        closePromise.then(() => true),
        new Promise((resolve) => setTimeout(() => resolve(false), 2_000)),
      ]);
      if (!exited && !closed) {
        child.kill("SIGTERM");
        const terminated = await Promise.race([
          closePromise.then(() => true),
          new Promise((resolve) => setTimeout(() => resolve(false), 1_000)),
        ]);
        if (!terminated && !closed) child.kill("SIGKILL");
        await closePromise;
      }
    }
  }

  return { child, events, request, close, stderr: () => stderr };
}

async function runProbe({ codex, root, name, config, sessionFlags, threadParams, role }) {
  const project = join(root, name, "project");
  const home = join(root, name, "codex-home");
  await mkdir(project, { recursive: true, mode: 0o700 });
  await chmod(project, 0o700);
  if (role) {
    await mkdir(join(project, ".codex", "agents"), { recursive: true, mode: 0o700 });
    await writeFile(join(project, ".codex", "agents", "motion-reviewer.toml"), role, { mode: 0o600 });
  }
  const server = await spawnServer({ codex, home, project, config, sessionFlags });
  try {
    const initialized = await server.request("initialize", {
      clientInfo: { name: "mirket-motion-model-controls", version: "0.0.1" },
      capabilities: { experimentalApi: true },
    });
    assert.ok(!initialized.error, JSON.stringify(initialized.error));
    server.child.stdin.write(JSON.stringify({ method: "initialized", params: {} }) + "\n");
    const configReply = await server.request("config/read", { cwd: project, includeLayers: true });
    assert.ok(!configReply.error, JSON.stringify(configReply.error));
    const threadReply = await server.request("thread/start", { cwd: project, ephemeral: true, ...threadParams });
    assert.ok(!threadReply.error, JSON.stringify(threadReply.error));
    const result = { config: configReply.result, thread: threadReply.result };
    if (role) {
      const rolePath = join(project, ".codex", "agents", "motion-reviewer.toml");
      const roleBytes = await readFile(rolePath);
      result.role = { path: rolePath, sha256: sha256(roleBytes), source: role };
    }
    return result;
  } finally {
    await server.close();
  }
}

function baseConfig(project, agents = "") {
  return `${agents}\n[projects.${JSON.stringify(project)}]\ntrust_level = "trusted"\n`;
}

async function main() {
  const flags = parseFlags(process.argv.slice(2));
  const codex = flags.get("--codex");
  const evidenceRoot = flags.get("--evidence-root");
  assert.ok(codex && evidenceRoot, "supply explicit --codex and --evidence-root paths");
  assert.ok(codex.startsWith("/Applications/ChatGPT.app/"), "qualification requires the Desktop-bundled CLI");
  const version = (await execFileAsync(codex, ["--version"], { encoding: "utf8" })).stdout.trim();
  assert.equal(version, "codex-cli 0.164.0-alpha.11.1");
  const codexHash = sha256(await readFile(codex));
  assert.equal(codexHash, "a04752e4d46a4d89e53e6f61c59c5349a638c8ce172ffbf391fd482b4d902a4e");
  const root = await mkdtemp(join(tmpdir(), "mirket-codex-motion-controls-"));
  await mkdir(evidenceRoot, { recursive: true, mode: 0o700 });
  const records = [];
  try {
    const defaults = `[agents]\ndefault_subagent_model = "${ROOT_MODEL}"\ndefault_subagent_reasoning_effort = "${ROOT_EFFORT}"\n`;
    const rootProbe = await runProbe({
      codex, root, name: "root-explicit", config: baseConfig(join(root, "root-explicit", "project"), defaults),
      sessionFlags: ["-c", `model="${ROOT_MODEL}"`, "-c", `model_reasoning_effort="${ROOT_EFFORT}"`],
      threadParams: {},
    });
    assert.equal(rootProbe.config.config.model, ROOT_MODEL);
    assert.equal(rootProbe.config.config.model_reasoning_effort, ROOT_EFFORT);
    assert.equal(rootProbe.thread.model, ROOT_MODEL);
    assert.equal(rootProbe.thread.reasoningEffort, ROOT_EFFORT);
    records.push({ name: "root-model-and-effort", status: "passed", observed: {
      model: rootProbe.thread.model, reasoning_effort: rootProbe.thread.reasoningEffort,
      config_origins: rootProbe.config.origins,
    } });

    const role = `name = "motion-reviewer"\ndescription = "Review saved motion artifacts without editing production files."\nmodel = "${ROOT_MODEL}"\nmodel_reasoning_effort = "${ROOT_EFFORT}"\ndeveloper_instructions = "Review only assigned saved artifacts and report exact evidence."\n`;
    const roleProject = join(root, "custom-role", "project");
    const roleProbe = await runProbe({
      codex, root, name: "custom-role", config: baseConfig(roleProject, defaults),
      sessionFlags: ["-c", `model="${ROOT_MODEL}"`, "-c", `model_reasoning_effort="${ROOT_EFFORT}"`],
      threadParams: {}, role,
    });
    assert.match(roleProbe.role.source, /^name = "motion-reviewer"/);
    assert.match(roleProbe.role.source, /model = "gpt-6-astra"/);
    assert.match(roleProbe.role.source, /model_reasoning_effort = "high"/);
    records.push({ name: "custom-native-role-file", status: "fixture-written-unqualified", observed: {
      role_sha256: roleProbe.role.sha256,
      role_path: ".codex/agents/motion-reviewer.toml",
      parent_model: roleProbe.thread.model,
      parent_reasoning_effort: roleProbe.thread.reasoningEffort,
      qualification: "file-and-parent-config only; native spawn selection was not claimed",
    } });

    const overrideProbe = await runProbe({
      codex, root, name: "explicit-lower-override", config: baseConfig(join(root, "explicit-lower-override", "project"), defaults),
      sessionFlags: ["-c", `model="${ROOT_MODEL}"`, "-c", `model_reasoning_effort="${ROOT_EFFORT}"`],
      threadParams: { model: LOWER_MODEL },
    });
    assert.equal(overrideProbe.thread.model, LOWER_MODEL);
    records.push({ name: "explicit-weaker-user-override", status: "passed", observed: {
      root_model: ROOT_MODEL, selected_model: overrideProbe.thread.model,
      root_effort: ROOT_EFFORT, selected_thread_effort: overrideProbe.thread.reasoningEffort,
      interpretation: "explicit user/session selection is honored; guidance cannot override it",
    } });

    const unavailableProbe = await runProbe({
      codex, root, name: "unavailable-model", config: baseConfig(join(root, "unavailable-model", "project"), defaults),
      sessionFlags: ["-c", `model="${ROOT_MODEL}"`, "-c", `model_reasoning_effort="${ROOT_EFFORT}"`],
      threadParams: { model: "mirket-model-that-is-not-real" },
    });
    assert.equal(unavailableProbe.thread.model, "mirket-model-that-is-not-real");
    records.push({ name: "unavailable-model-protocol", status: "observed-no-provider-validation", observed: {
      requested_model: unavailableProbe.thread.model,
      provider: unavailableProbe.thread.modelProvider,
      limitation: "thread/start accepts an opaque model slug; no provider request was made",
    } });

    const deniedProject = join(root, "agents-denied", "project");
    const denied = await runProbe({
      codex, root, name: "agents-denied", config: baseConfig(deniedProject, `${defaults}\nenabled = false\n`),
      sessionFlags: ["-c", `model="${ROOT_MODEL}"`, "-c", `model_reasoning_effort="${ROOT_EFFORT}"`],
      threadParams: {},
    });
    assert.equal(denied.config.config.agents.enabled, false);
    records.push({ name: "agents-disabled-config", status: "configured-unqualified", observed: {
      agents_enabled: denied.config.config.agents.enabled,
      multi_agent_mode: denied.thread.thread.multiAgentMode,
      limitation: "no native spawn was attempted; this is config/protocol denial evidence only",
    } });

    const catalogHome = join(root, "catalog-home");
    await mkdir(catalogHome, { recursive: true, mode: 0o700 });
    let catalog = null;
    try {
      const raw = (await execFileAsync(codex, ["debug", "models", "--bundled"], {
        encoding: "utf8", env: { ...process.env, CODEX_HOME: catalogHome, HOME: catalogHome },
      })).stdout;
      const parsed = JSON.parse(raw);
      const all = Array.isArray(parsed) ? parsed : (parsed.models ?? parsed.data ?? []);
      const astra = all.find((item) => item.slug === ROOT_MODEL || item.id === ROOT_MODEL || item.model === ROOT_MODEL);
      catalog = astra ? { model: ROOT_MODEL, advertised: true, entry: astra } : { model: ROOT_MODEL, advertised: false };
    } catch (error) {
      catalog = { model: ROOT_MODEL, advertised: null, error: String(error.message ?? error) };
    }
    records.push({ name: "bundled-model-catalog", status: catalog.advertised === true ? "passed" : "unverified", observed: catalog });
    const report = {
      schema: 1, generated_at: new Date().toISOString(), platform: `${process.platform}-${process.arch}`,
      cli: { path: codex, version, sha256: codexHash },
      model_turns: 0,
      records,
      limits: [
        "No provider model turn was run; unavailable-model validation stops at protocol acceptance.",
        "Custom-role file pins were checked, but Desktop-native spawn selection was not claimed.",
        "No PreToolUse hook or Desktop GUI spawn path was configured or qualified.",
      ],
    };
    const output = join(evidenceRoot, "protocol-controls.json");
    try { await access(output); throw new Error(`refusing to overwrite existing evidence: ${output}`); } catch (error) {
      if (error.code !== "ENOENT") throw error;
    }
    await writeFile(output, JSON.stringify(report, null, 2) + "\n", { mode: 0o600 });
    console.log(JSON.stringify({ status: "passed", evidence: output, records: records.map(({ name, status }) => ({ name, status })) }, null, 2));
  } finally {
    await rm(root, { recursive: true, force: true });
  }
}

await main();
