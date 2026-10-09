# Personal plugin bundles

`registry/harness.json` defines composable export bundles. The Codex marketplace is generated from reviewed selections; they contain skill instructions and their companions, not installed browser/cloud/MCP runtimes. Direct global setup is the default route on this Mac. Use bundles on a client where a marketplace is preferable, avoiding duplicate installation of the same skills. `personal-workbench` is the single-plugin option: the 14 globals plus motion and SVG craft, including executable tools and the specimen.

## Build a marketplace

From this repository:

```sh
mkdir -p build
python3 ai.py export --bundle personal-workbench --output build/personal-marketplace
```

Use `py -3` on Windows and create `build` with PowerShell's `New-Item -ItemType Directory -Force build`. Select any declared bundle; repeat `--bundle` to combine them. Bundles can share skills, so enable only the necessary combination in an app. Project manifests provide finer composition without those cross-plugin duplicates.

The parent directory must exist. The output must be absent, with no linked path ancestors. All payloads are prepared and checked before publishing the new directory. On macOS, `/tmp` and `/var` are symbolic links; use their resolved paths for exports into those trees. Export refuses to replace existing output. Choose a new directory for a new build, inspect it, and switch the client deliberately.

Output layout:

```text
.agents/plugins/marketplace.json   Codex marketplace
plugins/<bundle>/plugin.json      Portable Agent Plugins manifest
plugins/<bundle>/.codex-plugin/plugin.json
plugins/<bundle>/skills/           Reviewed payloads and source receipts
plugins/<foundation-bearing-bundle>/_harness/  Portable registry engine and authored sources
build-lock.json                   Source, installed and exported hashes; file modes
```

The foundation's skill-catalog locator points inside its plugin. It can discover and register additional project skills after the export is moved. Plugin registration alone does not install global instruction files; `ai.py install` handles guidance, skills and portable settings. Keep one authoritative checkout when maintaining source pins or regenerating bundles.

## Codex

With a client version supporting marketplace management:

```sh
codex plugin marketplace add /absolute/path/to/personal-marketplace
codex plugin marketplace list
```

Choose the desired plugin in the supporting desktop client's plugin interface and verify it after refresh. For a trusted repository, the documented enable setting is `[plugins."personal-foundation@personal-ai"]` with `enabled = true` in `.codex/config.toml`. Merge that entry with existing configuration only when choosing this installation route. Client surfaces and managed policies differ; the exporter does not claim activation from manifest creation. See [OpenAI's current plugin packaging and marketplace documentation](https://developers.openai.com/plugins/build/plugins).

Claude plugin marketplace export is unsupported. Claude Desktop Code and the CLI use direct skill registrations; Chat/Cowork use the manual `ai.py handoff` packages. The CLI also supports the [bounded delegation runner](../skills/agent-coordination/references/claude-code.md). Older generated exports may contain Claude manifests; build a fresh current export instead of reusing them.

## Update and verify

Review the registry diff, update its declared semantic version for a released bundle, and build a new output. Check `build-lock.json`, plugin manifests, retained licenses and source receipts. Test discovery and one bounded workflow in the actual client before distributing the build. A successful byte check establishes reproducible packaging, not toolchain availability or runtime correctness.

No marketplace has been published remotely by this task. Upstream runtime packages and restricted/unresolved sources remain outside automatic export. Their next steps are in [runtime integrations](runtime-integrations.md) and [source reviews](source-review.md).

The foundation snapshot includes `scripts/workbench/` and `examples/craft-lab/`.
Its installed skill-catalog launcher can run browser, office, vector and Markdown
operations after local runtimes are configured. Generated reports stay outside
the plugin source. This gives portable invocation, not automatic quality gating.
