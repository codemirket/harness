// Installed Codex CLI backend journey. No model turn or existing client configuration.
import assert from "node:assert/strict";
import { spawn, execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { chmod, mkdir, mkdtemp, readFile, realpath, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { isAbsolute, join } from "node:path";

const flags = new Map();
for (let i = 2; i < process.argv.length; i += 2) {
  const key = process.argv[i], value = process.argv[i + 1];
  assert.ok(["--codex", "--mirket", "--node", "--home", "--mode", "--registration-mode", "--native-skills", "--native-user-home", "--existing-registration"].includes(key) && value && !flags.has(key));
  if (key === "--mode") assert.ok(["read", "policy"].includes(value));
  else if (key === "--registration-mode") assert.equal(value, "managed");
  else if (key === "--native-skills") assert.equal(value, "yes");
  else if (key === "--existing-registration") assert.equal(value, "yes");
  else assert.ok(isAbsolute(value), `${key} must be absolute`);
  flags.set(key, value);
}
assert.ok(flags.has("--codex") && flags.has("--mirket"), "Supply --codex and --mirket");
assert.ok(flags.has("--home") ? !flags.has("--node") : flags.has("--node"),
  "Supply an installed --home, or a private --node development path");

const installed = flags.has("--home")
  ? JSON.parse(execFileSync(flags.get("--mirket"), ["dev", "doctor-core", "--home", flags.get("--home"), "--json"], { encoding: "utf8" }))
  : null;
const release = installed ? installed.active_release_id : "codex-host-smoke";
const navigationReady = installed?.active_components?.includes("design") ?? false;
assert.ok(release, "Selected installation has no active verified release");
const policy = flags.get("--mode") === "policy";
const managed = flags.get("--registration-mode") === "managed";
assert.ok(!managed || flags.has("--home") && !policy, "Managed registration requires an installed home and read mode");
const nativeSkills = flags.get("--native-skills") === "yes";
assert.ok(!nativeSkills || managed, "Native skill activation requires managed installed registration");
assert.ok(!flags.has("--native-user-home") || nativeSkills, "Native user home is only for skill activation");
assert.ok(!nativeSkills || flags.has("--native-user-home"), "Supply the disposable user home already bound to this installation's global skill receipt");
const existingRegistration = flags.get("--existing-registration") === "yes";
assert.ok(!existingRegistration || managed && nativeSkills, "Existing registration requires managed native skill qualification");
const serverName = managed ? "mirket" : "mirket_m0";
const selected = ["runtime_status", "memory_prepare", "memory_status", "memory_read", "code_navigate"];
if (!policy) selected.push("memory_commit");
const root = await realpath(await mkdtemp(join(tmpdir(), "mirket-codex-host-")));
const userHome = flags.get("--native-user-home") ?? join(root, "user-home");
const codexHome = existingRegistration ? join(userHome, ".codex") : join(root, "codex-home");
const project = join(root, "project"), state = join(root, "state");
let child, stderr = "", stream = "", nextId = 0;
let managedRegistered = false;
const pending = new Map();

async function request(method, params) {
  const id = ++nextId;
  child.stdin.write(JSON.stringify({ id, method, params }) + "\n");
  let timer;
  try {
    return await Promise.race([
      new Promise((resolve) => pending.set(id, resolve)),
      new Promise((_, reject) => { timer = setTimeout(() => reject(new Error(`${method} timed out: ${stderr.slice(-1000)}`)), 15_000); }),
    ]);
  } finally {
    clearTimeout(timer);
    pending.delete(id);
  }
}

function result(reply) {
  assert.ok(!reply.error, JSON.stringify(reply.error));
  return reply.result;
}

function toolValue(reply) {
  const value = result(reply);
  assert.ok(!value.isError, JSON.stringify(value));
  return JSON.parse(value.content.find((part) => part.type === "text").text);
}

try {
  await Promise.all([...(existingRegistration ? [] : [mkdir(codexHome, { mode: 0o700 })]), mkdir(project, { mode: 0o700 }), mkdir(state, { mode: 0o700 }), ...(flags.has("--native-user-home") ? [] : [mkdir(userHome, { mode: 0o700 })])]);
  await Promise.all([...(existingRegistration ? [] : [chmod(codexHome, 0o700)]), chmod(state, 0o700)]);
  const skill = join(project, ".agents", "skills", "qualification-method");
  await mkdir(skill, { recursive: true });
  await writeFile(join(skill, "SKILL.md"), "---\nname: qualification-method\ndescription: Apply the local qualification method.\n---\n\nRead only the explicitly selected saved source and verify its revision.\n");
  const navigationSource = join(project, "navigation.ts"), navigationText = "const answer = 42;\nanswer;\n";
  if (navigationReady) {
    await writeFile(navigationSource, navigationText);
  }
  const args = flags.has("--home")
    ? ["serve", "--home", flags.get("--home")]
    : ["dev", "serve", "--state-dir", state, "--release-id", release, "--node", flags.get("--node")];
  const configPath = join(codexHome, "config.toml");
  if (managed) {
    if (!existingRegistration) {
      await writeFile(configPath, "# unrelated Codex setting stays\nmodel = \"gpt\"\n");
      await chmod(configPath, 0o600);
      const registered = JSON.parse(execFileSync(flags.get("--mirket"), ["codex-register", "--home", flags.get("--home"), "--config", configPath], { encoding: "utf8" }));
      assert.equal(registered.status, "registered");
      managedRegistered = true;
    }
    const report = JSON.parse(execFileSync(flags.get("--mirket"), ["codex-doctor", "--home", flags.get("--home"), "--config", configPath], { encoding: "utf8" }));
    assert.equal(report.status, "managed");
    if (!existingRegistration) assert.match(await readFile(configPath, "utf8"), /# unrelated Codex setting stays/);
    assert.match(await readFile(configPath, "utf8"), /startup_timeout_sec = 60/);
    assert.match(await readFile(configPath, "utf8"), /required = true/);
  } else {
    await writeFile(configPath, `[mcp_servers.mirket_m0]\ncommand = ${JSON.stringify(flags.get("--mirket"))}\nargs = ${JSON.stringify(args)}\nstartup_timeout_sec = 60\nenabled_tools = ${JSON.stringify(selected)}\ndisabled_tools = ${JSON.stringify(policy ? ["memory_commit"] : ["job_cancel"])}\n`);
  }

  if (nativeSkills) {
    const global = JSON.parse(execFileSync(flags.get("--mirket"), ["skills", "global-sync", "--home", flags.get("--home"), "--user-home", userHome], { encoding: "utf8" }));
    const selected = JSON.parse(execFileSync(flags.get("--mirket"), ["skills", "project-select", "--home", flags.get("--home"), "--project-root", project, "--include", "architecture"], { encoding: "utf8" }));
    assert.ok(["synced", "unchanged"].includes(global.status));
    assert.equal(global.plan.selected.length, 9);
    assert.equal(selected.status, "synced");
    assert.deepEqual(selected.plan.selected, ["architecture"]);
  }

  child = spawn(flags.get("--codex"), ["app-server", "--strict-config", "--listen", "stdio://"], {
    cwd: project, env: { ...process.env, CODEX_HOME: codexHome, ...(nativeSkills ? { HOME: userHome } : {}) }, stdio: ["pipe", "pipe", "pipe"],
  });
  child.stdout.setEncoding("utf8").on("data", (data) => {
    stream += data;
    assert.ok(stream.length < 4 * 1024 * 1024, "App-server response stream exceeded fixture budget");
    for (let index; (index = stream.indexOf("\n")) >= 0;) {
      const line = stream.slice(0, index); stream = stream.slice(index + 1);
      if (!line.trim()) continue;
      const message = JSON.parse(line);
      if (message.id != null && pending.has(message.id)) pending.get(message.id)(message);
    }
  });
  child.stderr.setEncoding("utf8").on("data", (data) => { stderr = (stderr + data).slice(-32_768); });
  const initialized = result(await request("initialize", { clientInfo: { name: "mirket-qualification", version: "0.0.1" }, capabilities: { experimentalApi: true } }));
  assert.equal(initialized.codexHome, codexHome);
  child.stdin.write(JSON.stringify({ method: "initialized", params: {} }) + "\n");
  const skills = result(await request("skills/list", { cwds: [project], forceReload: true }));
  assert.ok(skills.data[0].skills.some((item) => item.name === "qualification-method" && item.scope === "repo" && item.enabled));
  if (nativeSkills) {
    assert.ok(skills.data[0].skills.some((item) => item.name === "architecture" && item.scope === "repo" && item.enabled), "Project-native skill was not discovered");
    assert.ok(skills.data[0].skills.some((item) => item.name === "runtime-operations" && item.enabled), "Global-native skill was not discovered");
  }
  const inventory = result(await request("mcpServerStatus/list", { detail: "toolsAndAuthOnly" }));
  const server = inventory.data.find((entry) => entry.name === serverName);
  assert.ok(server?.serverInfo, "Mirket did not start through Codex");
  assert.ok(!Object.keys(server.tools).some(name => name === "sql_navigate" || name.startsWith("browser_")),
    "Retired browser or SQL tools appeared in Codex's installed MCP inventory");
  if (managed) {
    for (const name of selected) assert.ok(name in server.tools, `Missing registered tool ${name}`);
  } else assert.deepEqual(Object.keys(server.tools).sort(), selected.sort());

  const thread = result(await request("thread/start", { cwd: project, ephemeral: true })).thread.id;
  const tool = (name, arguments_) => request("mcpServer/tool/call", { threadId: thread, server: serverName, tool: name, arguments: arguments_ });
  const status = toolValue(await tool("runtime_status", {}));
  assert.equal(status.release_id, release);
  if (navigationReady) {
    const navigation = toolValue(await tool("code_navigate", { expected_release_id: release,
      project_root: project, request: { sources: [{ path: navigationSource, language: "ts",
        expected_revision: `sha256:${createHash("sha256").update(navigationText).digest("hex")}` }],
      source: 0, position_byte: 19, operation: "definitions", limit: 10, typescript_model: { imports: [] } } }));
    assert.equal(navigation.navigation.outcome.status, "locations");
    assert.deepEqual(navigation.navigation.outcome.locations[0].location, { start_byte: 6, end_byte: 12 });
  }
  const scope = { kind: "project", project_root: project };
  const record = { kind: "fact", basis: "observed", content: "Codex host fixture fact.",
    sources: [{ source_id: "fixture:codex-host", revision: "fixture-1", locator: "fixture://codex-host", captured_at: 123 }],
    status: "current", critical: false,
    applicability: { task_key: null, checkout_id: null, operation: null },
    valid_from: null, valid_until: null };
  const mutation = { kind: "create", record, supersedes: null };
  const stale = result(await tool("memory_prepare", { expected_release_id: "stale", scope, mutation }));
  assert.equal(stale.isError, true);
  assert.match(stale.content[0].text, /release changed/i);
  if (policy) {
    const prepared = toolValue(await tool("memory_prepare", { expected_release_id: release, scope, mutation }));
    assert.equal(typeof prepared.operation_id, "string");
    const operation = { expected_release_id: release, scope, operation_id: prepared.operation_id };
    const denied = await tool("memory_commit", operation);
    assert.match(denied.error?.message ?? "", /tool 'memory_commit' is disabled for MCP server 'mirket_m0'/);
    const receipt = toolValue(await tool("memory_status", operation));
    assert.equal(receipt.operation_id, prepared.operation_id);
    assert.equal(receipt.status, "prepared");
    console.log("PASS Codex low-level host filter denied a valid prepared memory commit; no model turn");
  } else {
    const prepared = toolValue(await tool("memory_prepare", { expected_release_id: release, scope, mutation }));
    const committed = toolValue(await tool("memory_commit", { expected_release_id: release, scope,
      operation_id: prepared.operation_id }));
    assert.equal(committed.status, "committed");
    const saved = toolValue(await tool("memory_read", { expected_release_id: release, scope,
      record_id: committed.record.record_id, revision: committed.record.revision }));
    assert.equal(saved.record.data.content, record.content);
    console.log(`PASS Codex app-server ${managed ? "managed installation" : flags.has("--home") ? "installed release" : "development route"}: skill discovery${nativeSkills ? " (global and project native copies)" : ""}, scoped memory${navigationReady ? ", TypeScript navigation" : ""}, stale release refusal; no model turn`);
  }
} finally {
  if (child && child.exitCode === null) {
    child.stdin.end();
    await Promise.race([
      new Promise((resolve) => child.once("close", resolve)),
      new Promise((resolve) => setTimeout(() => { child.kill("SIGTERM"); resolve(); }, 3_000)),
    ]);
  }
  if (managedRegistered) {
    const configPath = join(codexHome, "config.toml");
    const removed = JSON.parse(execFileSync(flags.get("--mirket"), ["codex-remove", "--home", flags.get("--home"), "--config", configPath], { encoding: "utf8" }));
    assert.equal(removed.status, "removed");
    const config = await readFile(configPath, "utf8");
    assert.match(config, /# unrelated Codex setting stays/);
    assert.doesNotMatch(config, /\[mcp_servers\.mirket\]/);
  }
  await rm(root, { recursive: true, force: true });
}
