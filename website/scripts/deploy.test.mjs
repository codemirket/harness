import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs/promises";
import path from "node:path";
import os from "node:os";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";

test("deployment admits only exact committed artifact bytes", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "mirket-site-deploy-"));
  const source = fileURLToPath(new URL("../", import.meta.url));
  const run = (command, args) =>
    spawnSync(command, args, { cwd: root, encoding: "utf8", timeout: 15000 });
  try {
    await fs.mkdir(path.join(root, "website"));
    for (const dir of ["public", "design", "scripts"])
      await fs.cp(path.join(source, dir), path.join(root, "website", dir), {
        recursive: true,
      });
    await fs.copyFile(path.join(source, "../README.md"), path.join(root, "README.md"));
    await fs.mkdir(path.join(root, "docs"));
    for (const file of await fs.readdir(path.join(source, "../docs")))
      if (file.endsWith(".md"))
        await fs.copyFile(path.join(source, "../docs", file), path.join(root, "docs", file));
    await fs.copyFile(path.join(source, "../LICENSE"), path.join(root, "LICENSE"));
    await fs.writeFile(path.join(root, ".gitignore"), "website/dist/\n");
    for (const args of [
      ["init", "-q"],
      ["add", "."],
      [
        "-c",
        "user.name=Fixture",
        "-c",
        "user.email=fixture@example.invalid",
        "commit",
        "-qm",
        "fixture",
      ],
    ])
      assert.equal(run("git", args).status, 0);
    const build = () => run(process.execPath, ["website/scripts/build.mjs"]);
    const built = build();
    assert.equal(built.status, 0, built.stderr);
    const deploy = () =>
      run(process.execPath, ["website/scripts/deploy.mjs", "--dry-run"]);
    const good = deploy();
    assert.equal(good.status, 0, good.stderr);
    const guide = path.join(root, "docs/visual-quality.md");
    const guideBytes = await fs.readFile(guide);
    await fs.unlink(guide);
    const brokenLink = build();
    assert.notEqual(brokenLink.status, 0);
    assert.match(brokenLink.stderr, /visual-quality\.md/);
    await fs.writeFile(guide, guideBytes);
    const extra = path.join(root, "website/dist/unmanifested.txt");
    await fs.writeFile(extra, "must never upload");
    assert.match(deploy().stderr, /Unmanifested distribution files/);
    await fs.unlink(extra);
    const css = path.join(root, "website/dist/style.css");
    await fs.unlink(css);
    await fs.symlink(path.join(root, "website/public/style.css"), css);
    assert.match(deploy().stderr, /regular manifest-listed files/);
    await fs.unlink(css);
    await fs.copyFile(path.join(root, "website/public/style.css"), css);
    await fs.writeFile(path.join(root, "website/dist/app.js"), "stale bytes");
    assert.match(deploy().stderr, /Rebuild stale artifact/);
  } finally {
    await fs.rm(root, { recursive: true, force: true });
  }
});
