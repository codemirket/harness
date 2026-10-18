import fs from "node:fs/promises";
import path from "node:path";
import { createHash } from "node:crypto";
import { fileURLToPath } from "node:url";
import assert from "node:assert/strict";

const root = fileURLToPath(new URL("../", import.meta.url));
const hash = (data) => createHash("sha256").update(data).digest("hex");
const compiled = JSON.parse(
  await fs.readFile(path.join(root, "design/compiled.json")),
);
assert.equal(
  hash(await fs.readFile(path.join(root, "design/tokens.json"))),
  compiled.source_sha256,
  "Recompile changed tokens through Mirket",
);
assert.equal(
  hash(await fs.readFile(path.join(root, "public/tokens.css"))),
  compiled.css_sha256,
  "Compiled token CSS has drifted",
);
const html = await fs.readFile(path.join(root, "public/index.html"), "utf8");
const prompt = (await fs.readFile(path.join(root, "public/install-prompt.txt"), "utf8")).trim();
const escapeHtml = (value) => value.replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll(">", "&gt;").replaceAll('"', "&quot;").replaceAll("'", "&#x27;");
assert.ok(html.includes(`>${escapeHtml(prompt)}</textarea>`), "Website installation prompt drifted");
const readme = await fs.readFile(path.join(root, "../README.md"), "utf8");
assert.ok(readme.includes(`<!-- installation-prompt:start -->\n\x60\x60\x60text\n${prompt}\n\x60\x60\x60\n<!-- installation-prompt:end -->`), "README installation prompt drifted");
await fs.access(path.join(root, "../docs/agent-installation.md"));
assert.match(html, /<html lang="en">/);
assert.equal((html.match(/<h1\b/g) || []).length, 1);
const ids = [...html.matchAll(/\bid="([^"]+)"/g)].map((match) => match[1]);
assert.equal(new Set(ids).size, ids.length, "Duplicate DOM IDs");
for (const [, id] of html.matchAll(/href="#([^"]+)"/g))
  assert.ok(ids.includes(id), `Broken anchor: ${id}`);
for (const [, file] of html.matchAll(/(?:src|href)="\.\/([^"]+)"/g))
  await fs.access(path.join(root, "public", file));
const files = {};
for (const entry of await fs.readdir(path.join(root, "public"), {
  withFileTypes: true,
})) {
  assert.ok(
    entry.isFile(),
    "Only reviewed regular static files may be published",
  );
  const bytes = await fs.readFile(path.join(root, "public", entry.name));
  assert.ok(
    bytes.length < 128 * 1024,
    `Oversized website asset: ${entry.name}`,
  );
  files[entry.name] = hash(bytes);
}
assert.ok(Object.keys(files).length <= 12);
if (!process.argv.includes("--check")) {
  const output = path.join(root, "dist");
  await fs.rm(output, { recursive: true, force: true });
  await fs.cp(path.join(root, "public"), output, { recursive: true });
  await fs.writeFile(
    path.join(output, "build.json"),
    JSON.stringify({ site: "harness.mirket.dev", files }, null, 2) + "\n",
  );
}
console.log(
  `Website verified: ${Object.keys(files).length} static assets; canonical tokens match.`,
);
