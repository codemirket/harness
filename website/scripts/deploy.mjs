// Same release-directory + atomic symlink convention as mirket.dev.
import { execFileSync } from "node:child_process";
import { readFileSync, readdirSync, lstatSync } from "node:fs";
import { createHash } from "node:crypto";
import { fileURLToPath } from "node:url";
import path from "node:path";
const site = fileURLToPath(new URL("../", import.meta.url));
const repo = path.resolve(site, "..");
const options = process.argv.slice(2);
if (options.some((x) => !["--dry-run"].includes(x)))
  throw Error(
    "Only --dry-run is supported; destination is fixed to the harness host",
  );
const dry = options.includes("--dry-run");
const run = (cmd, args, opts = {}) =>
  execFileSync(cmd, args, { encoding: "utf8", ...opts });
if (
  run("git", ["status", "--porcelain", "--", "website"], { cwd: repo }).trim()
)
  throw Error("Commit website changes before deployment");
const sha = run("git", ["rev-parse", "HEAD"], { cwd: repo }).trim();
if (!/^[a-f0-9]{40}$/.test(sha)) throw Error("Invalid Git revision");
const hash = (data) => createHash("sha256").update(data).digest("hex");
const manifest = JSON.parse(readFileSync(path.join(site, "dist/build.json")));
const expected = [...Object.keys(manifest.files), "build.json"].sort();
if (
  JSON.stringify(readdirSync(path.join(site, "dist")).sort()) !==
  JSON.stringify(expected)
)
  throw Error("Unmanifested distribution files; rebuild before deployment");
for (const file of expected)
  if (
    !/^[a-zA-Z0-9_.-]+$/.test(file) ||
    !lstatSync(path.join(site, "dist", file)).isFile()
  )
    throw Error("Distribution must contain regular manifest-listed files only");
const tracked = run("git", ["ls-files", "--", "website/public"], { cwd: repo })
  .trim()
  .split("\n")
  .map((p) => p.replace("website/public/", ""))
  .sort();
if (
  JSON.stringify(tracked) !== JSON.stringify(Object.keys(manifest.files).sort())
)
  throw Error("Artifact files must exactly match committed public source");
for (const [file, digest] of Object.entries(manifest.files)) {
  if (!/^[a-zA-Z0-9_.-]+$/.test(file)) throw Error("Invalid artifact path");
  for (const folder of ["public", "dist"])
    if (hash(readFileSync(path.join(site, folder, file))) !== digest)
      throw Error(`Rebuild stale artifact: ${file}`);
}
const root = "/opt/harness.mirket.dev",
  release = `${root}/releases/${sha}`;
const ssh = [
  "-o",
  "BatchMode=yes",
  "-o",
  "StrictHostKeyChecking=yes",
  "-o",
  "ConnectTimeout=15",
  "admin@mirket.dev",
];
const quote = (value) => `'${value.replaceAll("'", "'\\''")}'`;
const remote = (script) =>
  dry
    ? (console.log(script), "")
    : run("ssh", [
        ...ssh,
        `sudo -n -u deploy bash -c ${quote("set -euo pipefail\n" + script)}`,
      ]);
// Never rsync into an active or already published release.
remote(`mkdir '${release}'`);
const rsync = [
  "-az",
  "--chmod=Du=rwx,Dgo=rx,Fu=rw,Fgo=r",
  "--rsync-path",
  "sudo -n -u deploy rsync",
  "-e",
  "ssh -o BatchMode=yes -o StrictHostKeyChecking=yes",
  `${site}dist/`,
  `admin@mirket.dev:${release}/`,
];
if (dry) console.log("rsync", ...rsync);
else run("rsync", rsync);
const checks = Object.entries(manifest.files)
  .map(([f, h]) => `${h}  ${f}`)
  .join("\n");
remote(
  `cd '${release}'\nchmod 755 .\nchmod 644 ${expected.map(quote).join(" ")}\nprintf '%s\\n' ${quote(checks)} | sha256sum --check --status`,
);
const previous = remote(
  `exec 9>'${root}/deploy.lock'\nflock -n 9\nreadlink '${root}/current' || true\nln -s '${release}' '${root}/.current-${sha}'\nmv -Tf '${root}/.current-${sha}' '${root}/current'`,
).trim();
if (!dry) {
  try {
    for (const [file, digest] of Object.entries(manifest.files)) {
      if (file === "404.html") continue;
      const response = await fetch(
        `https://harness.mirket.dev/${file === "index.html" ? "" : file}`,
        { signal: AbortSignal.timeout(15000), redirect: "error" },
      );
      if (
        !response.ok ||
        hash(Buffer.from(await response.arrayBuffer())) !== digest
      )
        throw Error(`Live bytes differ: ${file}`);
    }
    const missing = await fetch(
      "https://harness.mirket.dev/missing-verification-page",
      { signal: AbortSignal.timeout(15000) },
    );
    if (missing.status !== 404) throw Error("Missing pages must return 404");
  } catch (error) {
    // Roll back only our activation, never a subsequent deployment.
    if (new RegExp(`^${root}/releases/[a-f0-9]{40}$`).test(previous))
      remote(
        `exec 9>'${root}/deploy.lock'\nflock -n 9\nif [ "$(readlink '${root}/current')" = '${release}' ]; then ln -s '${previous}' '${root}/.rollback-${sha}'; mv -Tf '${root}/.rollback-${sha}' '${root}/current'; fi`,
      );
    else if (!previous)
      remote(
        `exec 9>'${root}/deploy.lock'\nflock -n 9\nif [ "$(readlink '${root}/current')" = '${release}' ]; then rm '${root}/current'; fi`,
      );
    throw error;
  }
  console.log(`Verified live release ${sha}: https://harness.mirket.dev`);
}
