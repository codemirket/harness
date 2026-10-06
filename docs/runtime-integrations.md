# Project-scoped design runtime recipes

These are operator recipes, not executed setup. They preserve the reviewed source revision and use commands actually present in pinned upstream documentation or source. Manual catalog status remains appropriate until the selected runtime/companions are reviewed and exercised on the target device. Source pinning alone does not pin a separately downloaded executable, browser, npm package, or container image.

## Common registration boundary

1. Select the target project and hosts. Inspect its existing skill/config directories and preserve unrelated content. Use a separate checked-out/vendor directory at the exact commit below; record its absolute location and source SHA. Do not update it implicitly during daily synchronization.
2. Register only the selected package in the project's supported skill directory: `.agents/skills/<name>` for the reviewed Codex-compatible layout and `.claude/skills/<name>` for Claude Code. Copy the whole skill subtree, including its references/scripts/assets, or link it if the host and operating system support that. A normal directory copy is the portable Windows option; a POSIX `ln -s` command is not a Windows recipe.
3. Copy required repository-level license/notices beside the registered package when not already included. Keep one canonical copy per host and avoid competing global copies with the same frontmatter name.
4. Inspect runtime requirements before executing anything. Use installed tools first; install missing dependencies only within authorized scope. Keep output and runtime caches distinct from source files.
5. Reload/start a fresh host session, verify the skill is discovered by its actual name, and run one bounded example in a scratch output directory. Distinguish registration, runtime availability, successful output validation and actual visual inspection. Stop at concrete missing capability; don't silently replace a failed check with a success claim.

The commands below use paths relative to the directory explicitly described. Substitute actual paths using the target shell's correct quoting. Do not run commands against another user's checkout or a production application as a smoke test.

## Diagram Design

**Source:** [cathrynlavery/diagram-design at 3996c1607503ec4bcdb60b018568359d20f71d15](https://github.com/cathrynlavery/diagram-design/tree/3996c1607503ec4bcdb60b018568359d20f71d15). Register **`skills/diagram-design/`**, actual name `diagram-design`. Preserve root `LICENSE` and `THIRD_PARTY_LICENSES.md`.

**Package and dependencies:** The skill directory is the portable unit, with 44 type references, templates, importers, SVG export and `scripts/self_check.py`. Python 3 runs its local helpers. Ordinary HTML/SVG viewing needs a modern browser. PNG rasterization uses Python Playwright + Chromium; upstream [README export instructions](https://github.com/cathrynlavery/diagram-design/blob/3996c1607503ec4bcdb60b018568359d20f71d15/README.md) give `pip install playwright` and `playwright install chromium`. Run installation only after choosing an appropriate isolated Python environment and confirming it is needed. Google Fonts require network unless a local/available substitute is configured.

**Setup:** Register the subtree in the project, retaining an external pinned full checkout if repository-only checks will be used. Resolve a project `.diagram-design` marker before editing shared style-guide files. The [profiles contract](https://github.com/cathrynlavery/diagram-design/blob/3996c1607503ec4bcdb60b018568359d20f71d15/skills/diagram-design/references/profiles.md) defines `profile: <slug>` and shared profile storage; do not modify a shared installed palette just to serve one project. Read applicable style/type/export references. Reuse an already supplied brand choice.

**Bounded verification:** From the installed skill directory, use the documented check on a generated HTML file:

```text
python3 scripts/self_check.py <file>
```

For an import smoke test, use one actual small fixture with its matching documented command:

```text
python3 scripts/drawio_extract.py <input>
python3 scripts/mermaid_extract.py <input>
python3 scripts/excalidraw_extract.py <input>
```

These extract content; they do not certify a subsequent redraw. Inspect the node/edge digest, redraw only the requested content, compare the fidelity ledger, run the self-check and visually inspect the HTML. Export SVG/PNG only when part of the test request, following `references/export.md` rather than inventing exporter flags.

The SKILL separately documents `python3 <repo-root>/scripts/verify-geometry.py <file>` and repository motion/skin checks. Those are **not** installed by copying the inner skill alone. Keep this distinction in the registration receipt. macOS and Windows can share source data, but interpreter naming and browser/font availability need local verification.

## Archify

**Source:** [tt-a1i/archify at 73aaa0696e8f72c232ea710e6fa94fd953f3e773](https://github.com/tt-a1i/archify/tree/73aaa0696e8f72c232ea710e6fa94fd953f3e773). Register the **entire `archify/` package**, actual name `archify`, not its SKILL alone. Preserve repository MIT license and applicable asset notices already inside the package.

**Package and dependencies:** `archify/package.json` declares Node **>=18**. Runtime generation uses bundled code/validators; the SKILL says no install is needed inside the package. Its development dependencies and repository test/build scripts are not prerequisites for normal generation. Automated browser checks use a locally installed Chrome/Chromium through the DevTools pipe, without requiring Playwright installation. [Browser implementation](https://github.com/tt-a1i/archify/blob/73aaa0696e8f72c232ea710e6fa94fd953f3e773/archify/bin/visual-check.mjs) detects macOS/Windows standard paths and accepts `ARCHIFY_CHROME` for an explicit executable path. Do not disable the browser sandbox to make a check pass.

**Setup:** Copy the entire package into the requested project skill directory. Keep normal cwd at the user's project; the SKILL explicitly allows replacing `bin/archify.mjs` with the installed package's absolute path. Set `ARCHIFY_UPDATE_CHECK_DISABLED=1` in the process environment if the selected project policy requires no update-check networking or reminder-state writes; this exact variable is documented in the pinned README. Do not silently install upgrades in response to an update receipt.

**Bounded verification:** From inside the installed package, upstream documents:

```text
node bin/archify.mjs doctor
node bin/archify.mjs guide "Show CI/CD checks, approval, deploy, and rollback"
```

Use a scratch output directory with the documented `demo <output-directory>` when testing examples. For a complete user-authored candidate, the current [SKILL](https://github.com/tt-a1i/archify/blob/73aaa0696e8f72c232ea710e6fa94fd953f3e773/archify/SKILL.md) uses:

```text
node bin/archify.mjs finalize <type> <candidate.json> <output.html> --quality showcase --json
```

For a repository-backed candidate include the documented `--repo-root <repo-root>` flag. Read the chosen schema/example before authoring; allowed modes are architecture, workflow, sequence, dataflow and lifecycle. Inspect exit status and receipt, then open the produced HTML and exercise one interaction. No Chrome/Chromium means browser verification is unavailable, not passed. A successful finalize with `visualReview: "not-requested"` is automated evidence, not perceptual review. PNG/WebM and other exports require their specific reference paths and separate validation.

## Impeccable

**Source:** [pbakaus/impeccable at 4e8504f10106a1cd7a99e37a401aa368d1d55576](https://github.com/pbakaus/impeccable/tree/4e8504f10106a1cd7a99e37a401aa368d1d55576). The checked-in Codex-compatible package is **`.agents/skills/impeccable/`**; Claude package is **`.claude/skills/impeccable/`**. Actual name is `impeccable`. Preserve Apache-2.0 `LICENSE` and `NOTICE.md` for MIT-derived platform references.

**Package and dependencies:** This revision has one consolidated skill plus command references, provider agent definitions, launcher, live-browser helpers and data. The launcher runs a native engine; Node is not required by that launcher. An npm-installer path does require Node (the package declares >=22.18.0). The source has separate SKILL, npm and engine versions, so record their observed values instead of assuming one version identifies them all.

**Setup choices:** For minimal project scope, copy only the matching provider skill folder with all contents, resolving runtime base paths to that folder. Do not copy the entire `.claude`/`.codex` tree over existing settings. Keep subagent definitions available in the package but check the host's ability/authorization before using them.

The [README](https://github.com/pbakaus/impeccable/blob/4e8504f10106a1cd7a99e37a401aa368d1d55576/README.md) also documents `npx impeccable install`, with `--providers=...` and `--scope=project|global`. It can install **provider-native project hooks**, not just skills. The unversioned `npx` command is an upstream convenience command, **not a source-pinned install recipe**. If the user chooses that installer, resolve the exact package/engine release first, preview/inspect intended changes, select project scope and requested providers, then verify hook configuration/trust separately. Do not infer blanket hook permission from “copy the skill.”

**Engine initialization and verification:** Inspect `scripts/VERSION`, the selected binary's origin and the [launcher](https://github.com/pbakaus/impeccable/blob/4e8504f10106a1cd7a99e37a401aa368d1d55576/.agents/skills/impeccable/scripts/impeccable). With no installed engine it may fetch the pinned engine version and SHA256 sidecar into a user cache. `IMPECCABLE_HOME` selects a writable cache location; `IMPECCABLE_BIN` selects a preinstalled binary. Overrides/cached binaries are not verified against the catalog Git commit, so record their independent identity.

The launcher itself documents `engine-probe` for setup. Use the Unix `scripts/impeccable` on macOS or documented `scripts/impeccable.cmd` on Windows without `sh`. Then run the skill's one-time-per-session `context` call from the target project. Use the host's skill invocation to request `init` only when durable PRODUCT.md setup is actually wanted; follow with one bounded audit/shape operation. Confirm expected files and report whether engine/context loading actually ran. A failure has a documented direct context-reading fallback; do not call fallback output engine-verified.

Live browser iteration, hook detectors and native subagents are optional further integration layers. Verify each independently before promising live selection or automatic post-edit checks.

## UI UX Pro Max

**Source:** [nextlevelbuilder/ui-ux-pro-max-skill at 477bcb28c9812b385cb51a4605ddf30d7b2266e2](https://github.com/nextlevelbuilder/ui-ux-pro-max-skill/tree/477bcb28c9812b385cb51a4605ddf30d7b2266e2). Main portable data/search package: **`.claude/skills/ui-ux-pro-max/`**, actual name `ui-ux-pro-max`. Preserve root MIT `LICENSE`. Its sibling skills are separate capabilities, not mandatory for a basic lookup.

**Dependencies:** Main search uses Python 3 standard library and local CSV datasets; it installs no packages and makes no network calls. Optional brand/token/image-generation siblings have separate Node/API/asset requirements and are not enabled by installing the main search package. Windows needs a working Python 3 interpreter and UTF-8 output where required by its instructions.

**Setup:** A pinned subtree copy with a catalog adaptation can be registered under the project host's skill path. Rewrite/override Claude-plugin-relative examples to resolve from the actual installed base directory. Copy scripts, data and references together. For the upstream generated installer route, [README](https://github.com/nextlevelbuilder/ui-ux-pro-max-skill/blob/477bcb28c9812b385cb51a4605ddf30d7b2266e2/README.md) documents `uipro init --ai codex`, `uipro init --ai claude`, `uipro init --ai universal`, and `uipro init --dry-run`. Do not use `--global` or `--ai all` for a project opt-in. The current npm package name is `ui-ux-pro-max-cli`; old `uipro-cli` releases are explicitly stale. Resolve an exact package version/source mapping before running an npm installer if reproducibility is required.

**Bounded verification:** Use an observed README command with its script path adjusted to the registered package:

```text
python3 .claude/skills/ui-ux-pro-max/scripts/search.py "error summary validation" --domain ux
python3 .claude/skills/ui-ux-pro-max/scripts/search.py "form validation" --stack react
```

Choose the actual project stack rather than blindly using React. Verify returned entries are local matches, not invented guidance. A broader documented design-system query is:

```text
python3 .claude/skills/ui-ux-pro-max/scripts/search.py "fintech banking" --design-system -f markdown
```

Leave `--persist` off for read-only lookup. When persistent design-system files are explicitly selected, inspect the generated destination and overwrite behavior first. A search result is a design suggestion, not proof of accessibility, CTR or rendering quality. Validate affected UI using the real project browser/tests.

## OpenDesign

**Source:** [nexu-io/open-design at 53231d40b778d88eba23f35547bf99485d3ae9fc](https://github.com/nexu-io/open-design/tree/53231d40b778d88eba23f35547bf99485d3ae9fc). This is both a skills collection and a full design application. Choose one of the following two distinct integration paths.

### Portable skill/template only

Copy a fully reviewed substantive **`skills/<selected-name>/`** subtree into the project, including required assets/references, and preserve its applicable root/file-specific license. Examples include reference-design-contract, research-decision-room and reviewed HTML recipes. Adapt OpenDesign `<artifact>` delivery to actual local files and available preview tools. Confirm the recipe renders and its input facts survive.

Do **not** install one of the 85 advertisement-only wrappers as functional support. Entries such as threejs, color-expert or flutter-animating-apps explicitly point to another repository for the real bundle. This path needs no OpenDesign daemon, account, MCP connection or global registry.

### Full local application and optional MCP bridge

[QUICKSTART.md](https://github.com/nexu-io/open-design/blob/53231d40b778d88eba23f35547bf99485d3ae9fc/QUICKSTART.md) and root package manifest require Node **24.x** and pnpm **10.33.x** (`packageManager` pins 10.33.2). Use the complete pinned checkout. The README's observed source-setup sequence is:

```text
corepack enable
pnpm install
pnpm tools-dev run web
```

`pnpm install` runs a postinstall and native-package builds; this is app setup, not a passive skill copy. Inspect the selected installation/runtime requirements before proceeding. Open the URL printed by tools-dev; ports are dynamically allocated. Configure one supported local agent CLI or a BYOK runtime through Settings, and verify the selected CLI actually launches/authenticates. Native Windows and WSL2 have separate troubleshooting guides; keep daemon and agent paths in the same intended environment. Do not accidentally point a native Windows process at WSL-only executables.

For MCP, the [README integration section](https://github.com/nexu-io/open-design/blob/53231d40b778d88eba23f35547bf99485d3ae9fc/README.md) documents `od mcp install <agent>` and **Settings → MCP server** snippets. The CLI installer writes user configuration; **there is no project-scope flag verified in this review**. For project-only registration, inspect the generated server snippet and merge it into the host's supported project MCP config. On macOS, `/usr/bin/od` can win on PATH, so use the app's actual generated command/path rather than blindly running bare `od`.

The daemon defaults to loopback/read-only external access. Inspect actual tool permissions and project/data access before registering; project-local host config alone does not prove server-side access is limited to a single OpenDesign project. Do not enable LAN exposure or relax internal-host restrictions as routine setup.

With the correct OpenDesign CLI resolved, upstream documents read-only checks:

```text
od project list --json
od files list <project-id> --json
od files read <project-id> <relative-path>
od plugin list --json
```

Use a deliberate test project/file and confirm the MCP client's discovery and one corresponding read operation. Only then enable a write workflow such as brand extraction, which specifically depends on `od brand preview <brandId>` and `od brand finalize <brandId> --json`. Plugin application or brand finalization changes OpenDesign state and is separate from merely connecting a reader.

Preserve root Apache-2.0 and any bundled file-specific MIT notices. Verify produced HTML/PDF/PPTX/MP4 with the relevant artifact workflow; application launch or MCP tool discovery alone does not verify export fidelity.


## Gstack: complete lifecycle and browser suite

Use [the pinned source](https://github.com/garrytan/gstack/tree/c285d88b90d39116ccfa2b901f80ea0fce0b26eb) and its actual host-specific setup, not a copy of a generated SKILL.md. Git and Bun are core requirements; Windows has additional Node/Bash requirements. Codex support is marked experimental in this snapshot. Browser work uses Aside on supported macOS or bundled Chromium; security scanning and outside-model review have further runtimes. Review the pinned setup script, selected host adapter, hooks, telemetry, update behavior and credential/data flows before executing setup within the requested scope. Preserve an independent receipt for built/downloaded runtime versions. Start a fresh host session and test one scoped review/browser task before depending on the suite. The catalog's four `gstack-openclaw-*` methods are separate, portable project selections; installing them does not install this runtime.

## Context-mode: output indexing and continuity

Use [the pinned source](https://github.com/mksglu/context-mode/tree/e5fcca6802d484db3f4ec04f6d2021938852b2a0), whose package requires Node >=22.5 and Elastic-2.0 licensing. Its MCP service, hooks, indexing database and routing skill are one integration. The README's `cp .../configs/codex/AGENTS.md ~/.codex/AGENTS.md` recipe would replace the shared instructions: merge only the needed project routing into the actual project configuration instead. Inspect host approvals because execution tools can run arbitrary code with network access; output containment is not an OS sandbox. Review postinstall/startup self-repair, registry/cache writes and storage retention. Verify a bounded synthetic indexing/search task, then session recovery, before using private corpora. Do not enable hooks or replace global guidance merely to register skills.

## Graphify: code and document intelligence

Use [the pinned Python package](https://github.com/Graphify-Labs/graphify/tree/5c7b84792f453582676548185aaec3824d51dfe2) in an isolated environment with its retained Apache/MIT notices. Choose corpus paths, exclusions, local/provider processing and credentials deliberately. Inspect installation and platform integration before executing it: upstream setup can change global instructions and those may be shared symlinks in this repository. Preserve those files and integrate only the requested project. Build a small synthetic index, inspect entities/edges and queries, then record source revision and refresh rules. Inferred edges and stale indexes are navigation aids, not proof of actual execution dependencies.

## Provider and specialist integrations

- [OpenAI plugins](https://github.com/openai/plugins/tree/5fd93af4cd0c623e020d0cc7e9ce178b4ac1f70f): official successor to deprecated openai/skills. Use the actual native plugin package for iOS/macOS/Expo, account connectors and provider tools when selected. A manifest label is not a substitute for reading the package license; Expo includes its own MIT terms. Review plugin permissions and tool availability on each device.
- [Anthropic document implementations](https://github.com/anthropics/skills/tree/683bc88e56f3e09ba94f7055977f3d3aa499f202): local document licenses restrict copying/adaptation. Use the host's available licensed document capability or the authored `office-authoring`/`document-parsing` workflows. The registry does not redistribute those implementations.
- [Anthropic MCP builder](https://github.com/anthropics/skills/tree/683bc88e56f3e09ba94f7055977f3d3aa499f202/skills/mcp-builder): Apache-2.0 skill with templates and evaluation scripts. Main workflow reviewed; full helper/reference review and target SDK verification remain before direct registration. The authored `mcp-integration` skill is available immediately.
- ECC Context7 lookup and specialist routers: inspect each catalog entry's named tool/companion requirement. A tool name in Markdown does not establish a configured MCP connection.

Manual entries with `adapt-first` or `license-review` status require the specific repair or provenance work recorded by `show <id>` before registration. Runtime setup is not a way to bypass a known content defect. Applying these recipes can change project or host configuration; carry out only the selected, authorized integration and retain its independent verification evidence.

## Agent-browser and Vercel packages

The [Vercel review](reviews/vercel.md) records version 0.38.2, exact source pins, native binary/browser dependencies, authentication boundaries and each runtime guide. `agent-browser` is not part of the default global installation. Copying its discovery stub does not install its executable or guide data.

For a deliberate integration, select the CLI, browser and version-matched `skill-data` as separate components. Verify downloaded artifact identity and preserve existing global shims: the reviewed npm postinstall can rewrite one during a local install. Choose named sessions and explicitly scoped authentication. Check `--version`, guide discovery and one bounded scratch browser task; confirm actual browser output. Do not use a user's live profile as a disposable test fixture. Windows ARM64 uses the distribution's x64 fallback and needs device verification. Cloud browser providers add separate credentials, permissions and costs.

The four curated React skills are self-contained guidance and require no Vercel account or runtime installation. Deploy/token skills and the larger optimization suite remain manual because of secret handling, external writes, incomplete script review and Windows path concerns recorded in the source report. No deploy or account configuration is implied by selecting React guidance.

## Planning-with-files

See the [planning review](reviews/planning-search.md) for all language variants, host mirrors, actual hook routes and partial runtime coverage. Hooks live in the skill frontmatter itself, so merely registering the upstream skill can activate behavior when a supporting host invokes it. A claim that the registrar does not execute code is not enough to approve that installation.

The default authored `work-planning` skill provides durable task state without these hooks. For a deliberate upstream integration, choose the target host and one canonical language, inspect all relevant hook implementations and output paths, and retain the full runtime/adapter/companion package. Explicitly bound optional continuation and history replay; protect session content. Check isolated startup, resume, stop and failure behavior on the actual host before promotion. POSIX helper availability and PowerShell behavior must be assessed separately on Windows.

## GEO / SEO workflows

The [search review](reviews/planning-search.md) records unsupported score formulas, factual assumptions and helper limitations in the pinned GEO package. The authored `search-visibility` project skill supports measurement-led discovery, indexing, content and answer-surface research without these claims.

Promoting an upstream GEO workflow requires correcting the specific skill and its templates/helpers, verifying robots directives and redirects against actual standards, and labeling unavailable evidence. Do not invent contact addresses, treat an untested platform as a negative result, or convert arbitrary scores to financial forecasts. Web fetches, model APIs and reporting dependencies are distinct integration steps. Test one local fixture and one authorized public-site case, checking source evidence and report claims before using it for customer decisions.

## Remotion video skills

The [media review](reviews/quality-media.md) covers all 12 canonical skill bodies and identifies companion-read limits. No explicit applicable source license was verified at the selected revision; the registry retains these as discoverable manual entries instead of exporting copies.

Before a future installation, resolve the skill license and preserve the chosen skill directory with its references/assets. Select compatible Node, React, Remotion, browser/rendering and media dependencies for the actual project; map tiles, fonts, fetched code and media have separate terms. Confirm local or paid rendering scope and inspect the produced video/still, including dimensions, codec, timing, text/captions and audio. Studio preview alone does not verify an exported artifact. Do not claim Windows compatibility until the actual render path is exercised there.
