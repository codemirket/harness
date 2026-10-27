// Actual installed MCP/private Node/same-binary navigation over disposable sources.
import assert from "node:assert/strict";
import { execFileSync, spawn } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdtemp, mkdir, readFile, readdir, realpath, rm, stat, symlink, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { isAbsolute, join } from "node:path";

const usage = "Usage: node tests/code-navigation-installed-smoke.mjs --home /absolute/disposable-home --release-id sha256:...";
const args = new Map();
for (let i = 2; i < process.argv.length; i += 2) {
  const key = process.argv[i], value = process.argv[i + 1];
  assert.ok(["--home", "--release-id"].includes(key) && value && !args.has(key), usage); args.set(key, value);
}
assert.equal(args.size, 2, usage);
const home = args.get("--home"), release = args.get("--release-id");
assert.ok(isAbsolute(home) && (await stat(home)).isDirectory(), usage); assert.match(release, /^sha256:[0-9a-f]{64}$/);
const binary = join(home, "bin/mirket");
const doctor = JSON.parse(execFileSync(binary, ["dev", "doctor-core", "--home", home, "--json"], { encoding: "utf8", timeout: 30000, stdio: ["ignore", "pipe", "pipe"] }));
assert.equal(doctor.status, "healthy"); assert.equal(doctor.active_release_id, release); assert.ok(doctor.active_components.includes("design"));
const fixture = await realpath(await mkdtemp(join(tmpdir(), "mirket-code-navigation-installed-")));
const project = join(fixture, "project"); await mkdir(project, { mode: 0o700 });
const revision = (text) => "sha256:" + createHash("sha256").update(text).digest("hex");
const position = (text, target, last = true) => Buffer.byteLength(text.slice(0, last ? text.lastIndexOf(target) : text.indexOf(target)));
const span = (text, target, last = true) => ({ start_byte: position(text, target, last), end_byte: position(text, target, last) + Buffer.byteLength(target) });
const mainText = "\ufeff// 😀\r\nconst çay = 1; function f(çay: number) { return çay; }\r\nçay;";
const globalText = "const selectedGlobal = çay;";
const importsText = 'import { çay } from "./source"; çay;';
const moduleText = "export function selam(kişi: string) { return kişi; }";
const consumerText = 'import { selam } from "./module"; const message = selam("Merhaba");';
const declarationText = "export declare function prepared(x: number): number;";
const declarationUseText = 'import { prepared } from "prepared"; prepared(2);';
const files = new Map([["source.ts", mainText], ["selected.ts", globalText], ["imports.ts", importsText],
  ["module.ts", moduleText], ["consumer.ts", consumerText],
  ["prepared.d.ts", declarationText], ["declaration-use.ts", declarationUseText],
  ["tsconfig.json", JSON.stringify({ compilerOptions: { plugins: [{ name: "./never-plugin.cjs" }] }, typeAcquisition: { enable: true } })],
  ["package.json", JSON.stringify({ scripts: { install: "should-never-execute", prepare: "should-never-execute" } })],
  ["never-plugin.cjs", `require("node:fs").writeFileSync(${JSON.stringify(join(project, "plugin-executed"))}, "unexpected"); throw new Error("never load project plugin");`]]);
for (const language of ["js", "ts", "jsx", "tsx"]) {
  files.set(`kind.${language}`, "\ufeff// 😀\r\n" + (language.endsWith("x") ? "const Panel = () => <div/>; const view = <Panel/>;" : "const café = 1; café;") + "\r\n");
}
for (const [filename, text] of files) await writeFile(join(project, filename), text);
await writeFile(join(fixture, "outside.ts"), mainText); await symlink(join(fixture, "outside.ts"), join(project, "link.ts"));
const filenames = (await readdir(project)).sort();
const child = spawn(binary, ["serve", "--home", home], { stdio: ["pipe", "pipe", "pipe"] });
let next = 0, buffer = "", stderr = "", failure;
const pending = new Map();
const closed = new Promise((resolve, reject) => {
  child.on("error", reject); child.on("close", (code, signal) => {
    for (const request of pending.values()) { clearTimeout(request.timer); request.reject(new Error(`MCP closed: ${stderr}`)); }
    pending.clear(); resolve({ code, signal });
  });
});
child.stdout.setEncoding("utf8").on("data", (chunk) => {
  try {
    buffer += chunk; assert.ok(Buffer.byteLength(buffer) <= 1024 * 1024);
    for (;;) {
      const end = buffer.indexOf("\n"); if (end < 0) break;
      const frame = JSON.parse(buffer.slice(0, end)); buffer = buffer.slice(end + 1);
      const request = pending.get(frame.id); if (!request) continue;
      clearTimeout(request.timer); pending.delete(frame.id);
      if (frame.error) request.reject(new Error(JSON.stringify(frame.error))); else request.resolve(frame.result);
    }
  } catch (error) { failure = error; for (const request of pending.values()) request.reject(error); child.kill("SIGKILL"); }
});
child.stderr.setEncoding("utf8").on("data", (chunk) => { stderr += chunk; if (Buffer.byteLength(stderr) > 65536) { failure = new Error("MCP stderr bound"); child.kill("SIGKILL"); } });
function rpc(method, params) {
  if (failure) throw failure;
  const id = ++next;
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => { pending.delete(id); reject(new Error(`${method} deadline: ${stderr}`)); }, 45000);
    pending.set(id, { resolve, reject, timer }); child.stdin.write(JSON.stringify({ jsonrpc: "2.0", id, method, params }) + "\n");
  });
}
async function tool(name, parameters = {}, error = false) {
  const response = await rpc("tools/call", { name, arguments: parameters });
  assert.equal(response.isError === true, error, JSON.stringify(response)); if (error) return response;
  const text = JSON.parse(response.content.find((part) => part.type === "text").text);
  if (response.structuredContent) assert.deepEqual(response.structuredContent, text);
  const result = response.structuredContent ?? text;
  if (name === "code_navigate") assert.ok(Buffer.byteLength(JSON.stringify(result)) <= 32768);
  return result;
}
function selection(filename, language = "ts") { return { path: join(project, filename), language, expected_revision: revision(files.get(filename)) }; }
function query(sources, source, position_byte, operation = "definitions") {
  return { expected_release_id: release, project_root: project, request: { sources, source, position_byte, operation, limit: 100, typescript_model: { imports: [] } } };
}
let passed = false;
try {
  await rpc("initialize", { protocolVersion: "2025-11-25", capabilities: {}, clientInfo: { name: "code-navigation-installed-smoke", version: "0.0.1" } });
  child.stdin.write(JSON.stringify({ jsonrpc: "2.0", method: "notifications/initialized", params: {} }) + "\n");
  const listing = await rpc("tools/list", {}), info = listing.tools.find((entry) => entry.name === "code_navigate");
  assert.ok(info); assert.equal(info.annotations.readOnlyHint, true);
  const status = await tool("runtime_status"); assert.equal(status.release_id, release);
  const sources = [selection("source.ts"), selection("selected.ts")];
  const args = query(sources, 0, position(mainText, "çay"));
  const definition = (await tool("code_navigate", args)).navigation.outcome;
  assert.equal(definition.status, "locations"); assert.equal(definition.model_complete, false); assert.equal(definition.complete, true);
  assert.deepEqual(definition.backend, { name: "typescript-language-service", version: "5.9.3" });
  assert.deepEqual(definition.locations, [{ source: 0, revision: revision(mainText), location: span(mainText, "çay", false) }]);
  const refs = (await tool("code_navigate", { ...args, request: { ...args.request, operation: "references" } })).navigation.outcome;
  assert.deepEqual(refs.locations.map((entry) => [entry.source, entry.location]), [[0, span(mainText, "çay", false)], [0, span(mainText, "çay")], [1, span(globalText, "çay")]]);
  for (const entry of refs.locations) assert.equal(typeof entry.is_write, "boolean");
  for (const language of ["js", "ts", "jsx", "tsx"]) {
    const filename = `kind.${language}`, text = files.get(filename), target = language.endsWith("x") ? "Panel" : "café";
    const outcome = (await tool("code_navigate", query([selection(filename, language)], 0, position(text, target)))).navigation.outcome;
    assert.equal(outcome.status, "locations"); assert.deepEqual(outcome.locations[0].location, span(text, target, false));
  }
  for (const operation of ["definitions", "references"]) {
    const imports = (await tool("code_navigate", query([selection("source.ts"), selection("imports.ts")], 1, position(importsText, "çay"), operation))).navigation.outcome;
    assert.equal(imports.status, "refused"); assert.equal(imports.reason, "missing_import_binding"); assert.equal(imports.locations, undefined);
  }
  const selectedImports = query([selection("module.ts"), selection("consumer.ts")], 1,
    position(consumerText, "selam"));
  selectedImports.request.typescript_model.imports = [{ source: 1, specifier: "./module", target: 0 }];
  const imported = (await tool("code_navigate", selectedImports)).navigation.outcome;
  assert.equal(imported.status, "locations", JSON.stringify(imported));
  assert.equal(imported.model_complete, false); assert.match(imported.model_revision, /^sha256:[0-9a-f]{64}$/);
  assert.ok(imported.locations.some(entry => entry.source === 0
    && entry.revision === revision(moduleText)
    && entry.location.start_byte === span(moduleText, "selam", false).start_byte));
  const selectedDeclaration = query([selection("prepared.d.ts", "ts_declaration"),
    selection("declaration-use.ts")], 1, position(declarationUseText, "prepared"));
  selectedDeclaration.request.typescript_model.imports = [{ source: 1, specifier: "prepared", target: 0 }];
  const declared = (await tool("code_navigate", selectedDeclaration)).navigation.outcome;
  assert.equal(declared.status, "locations", JSON.stringify(declared));
  assert.ok(declared.locations.some(entry => entry.source === 0
    && entry.revision === revision(declarationText)
    && entry.location.start_byte === span(declarationText, "prepared", false).start_byte));
  await writeFile(join(project, "prepared.d.ts"), declarationText + "\n// changed");
  const changedDeclaration = (await tool("code_navigate", selectedDeclaration)).navigation.outcome;
  assert.equal(changedDeclaration.status, "changed"); assert.equal(changedDeclaration.locations, undefined);
  await writeFile(join(project, "prepared.d.ts"), declarationText);
  await writeFile(join(project, "selected.ts"), "const changed = 2;");
  const changed = (await tool("code_navigate", args)).navigation.outcome;
  assert.equal(changed.status, "changed"); assert.equal(changed.locations, undefined);
  assert.equal(changed.current_revisions[1], revision("const changed = 2;")); await writeFile(join(project, "selected.ts"), globalText);
  for (const path of [join(fixture, "outside.ts"), join(project, "link.ts")]) {
    await tool("code_navigate", query([{ ...sources[0], path }], 0, position(mainText, "çay")), true);
  }
  await tool("code_navigate", query([sources[0], sources[0]], 0, position(mainText, "çay")), true);
  await tool("code_navigate", { ...args, expected_release_id: "stale" }, true);
  for (const [filename, text] of files) assert.equal(await readFile(join(project, filename), "utf8"), text, `${filename} changed`);
  assert.deepEqual((await readdir(project)).sort(), filenames, "navigation created project files or executed the plugin");
  child.stdin.end(); assert.equal((await closed).code, 0, stderr); passed = true;
  console.log(JSON.stringify({ status: "passed", release_id: release, target: doctor.active_target,
    coverage: "installed MCP/private Node/owned worker, all four script kinds, exact Unicode CRLF spans, lexical shadowing, selected imports/declarations and their revision drift, selected globals, unresolved-import/stale/scope/duplicate refusal, source preservation" }));
} finally {
  for (const request of pending.values()) clearTimeout(request.timer);
  if (!passed) { child.stdin.end(); await Promise.race([closed, new Promise((resolve) => setTimeout(resolve, 3000))]); if (child.exitCode === null) child.kill("SIGKILL"); console.error(`Retained failed navigation fixture: ${fixture}`); }
  else await rm(fixture, { recursive: true, force: true });
}
