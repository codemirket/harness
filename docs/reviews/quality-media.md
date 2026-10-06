# Remotion and Addy Osmani source review

Reviewed 2026-10-06. Static source inspection only: no upstream programs, installers, hooks, npm packages or examples were run; no installed skills were changed during source review. All 37 canonical `SKILL.md` bodies were read. The 13 project recommendations also had all bundled local companions and required root shared references read in full. Remotion's large transitive markup/maps assets and scripts were inventoried but not fully audited; no direct install is recommended for them.

## Source pins and licensing

| Source | Exact commit | Inventory | License evidence |
|---|---|---|---|
| [Remotion skills](https://github.com/remotion-dev/skills/tree/473352613039e718e46655a26df224851e84c4aa) | `473352613039e718e46655a26df224851e84c4aa` | 12 canonical skills, 287 tracked files | No LICENSE or explicit license declaration found in this checkout, package metadata or SKILL bodies. Keep `NOASSERTION`. |
| [Addy agent-skills](https://github.com/addyosmani/agent-skills/tree/1401c8b8030e023baeebb31781a6653fe8e93026) | `1401c8b8030e023baeebb31781a6653fe8e93026` | 25 canonical skills, 210 tracked files | [Root MIT LICENSE, copyright 2025 Addy Osmani](https://github.com/addyosmani/agent-skills/blob/1401c8b8030e023baeebb31781a6653fe8e93026/LICENSE). Retain with every copied/adapted payload. |

Remotion's README explicitly invites installation, and its package metadata points back to the Remotion monorepo. That supports intended skill usage but does not identify a verified redistributable license for this detached skill snapshot. The separate Remotion runtime/monorepo licensing must not be labeled MIT by inference. Fonts, music, map imagery, Whisper models, remote elements, paid platforms and runtime dependencies retain their own terms regardless of the eventual skill-text license.

## Integration result

[catalog.json](../../registry/catalog.json) retains all 37 source entries: **13 adapted Addy project options, 12 Addy manual alternatives/repair candidates, and 12 Remotion manual capabilities**. No blanket always-on upstream router is recommended. This preserves the full selection surface without silently loading a complete engineering lifecycle for a small request. Project here means a candidate after its explicit adaptation and payload remapping are applied; it does not mean the raw source has been runtime validated.

Useful global principles worth keeping in original prose are already compatible with personal guidance: compare evidence against the intended outcome; keep uncertainty and source claims distinct; verify installed API versions; make recovery and consumer contracts explicit; preserve task state across handoffs; choose checks from risk. These sources do not justify new universal percentages, mandatory approvals, line-count caps, model selection, or whole-repository process for every task.

## Addy: all skills and disposition

Exact paths are `skills/<name>` and frontmatter names match the table. All carry the root MIT license. The catalog contains exact permanent body URLs, dependencies, caveats and concrete adaptation text for every entry.

| Name | Disposition | Useful contribution / decisive caveat |
|---|---|---|
| api-and-interface-design | Adapted project | Contract/error/boundary thinking; strongest section is intent-based idempotency, atomic claim and unknown outcome. Scope idempotency keys to tenant/operation and design reconciliation; database ownership does not make data automatically trusted. |
| code-review-and-quality | Adapted project | Prioritized multi-axis review and evidence checks. Default review must remain read-only; its mutation experiment cannot edit a shared working tree merely because review was requested. Avoid structural nits becoming blockers. |
| context-engineering | Adapted project | Bounded retrieval and handoffs. File class is not instruction authority; fixed context percentages/line counts and mandatory session resets are heuristics, not proven universal constraints. |
| debugging-and-error-recovery | Adapted project | Preserve/reproduce/localize/reduce/regress. Avoid repeated permission for independently verified diagnostics; React try/catch around returning JSX is not a descendant error boundary. |
| documentation-and-adrs | Adapted project | Good preservation of existing ADR conventions and historical rationale. Correct illustrative SQLite rejection and mislabeled sliding-window counter before treating examples as facts. |
| idea-refine | Adapted project | Divergence/convergence, assumptions and MVP brief. Replace provider-specific tools and do not restart an intake for already answered questions. Optional initializer is only `mkdir -p docs/ideas`. |
| incremental-implementation | Adapted project | Meaningful slices and scoped work. Remove implicit commit/clean-tree requirement and universal down-migration promise; retain unrelated changes and existing authorization. |
| observability-and-instrumentation | Adapted project | Questions before signals, cardinality, entry-point identity and checking emitted data. Preserve existing tools/severity scheme; sampling cannot retain errors discarded upstream; alert testing needs safe destinations. |
| performance-optimization | Adapted project | Comparable measurements, noise, cache keys, query plans, pools, rejected-experiment record. Keep privacy cache headers; query-plan shape alone is not the acceptance metric; field results must not be inferred from lab runs. |
| planning-and-task-breakdown | Adapted project | Dependency graph, vertical slices, risk-first order, acceptance evidence. Avoid mandatory separate plan approvals/file-count caps or automatic external issue creation. |
| security-and-hardening | Adapted project | Trust boundaries, authorization, SSRF/path races, supply chain. Explicit corrections needed for SameSite/CSRF, denylist response spread, raw secret grep, categorical CORS ban and universal consent claims. |
| source-driven-development | Adapted project | Official version-aware lookup for consequential uncertain APIs. Manifest ranges are not exact installed versions; latest docs are not automatic authority to replace existing compatible code. |
| test-driven-development | Adapted project | Red/green evidence, outcome assertions, isolation and test-double judgment. Opt-in TDD, not universal test-first policy; test proportions are examples, tests are bounded evidence, host browser capabilities may substitute. |
| browser-testing-with-devtools | Manual runtime | Chrome DevTools MCP/Node/Chrome setup and profile isolation. Existing host tools may already suffice; setup currently fetches `@latest`. Blanket navigation/JS reconfirmations and project-code trust categorization need adaptation. |
| ci-cd-and-automation | Manual repair | Rich pipeline/rollout sketch, but direct `${{ inputs.version }}` shell interpolation, fork-unavailable DB secrets and absent policy/isolation decisions should not be shipped as a generic template. Prefer authored CI skill. |
| code-simplification | Manual repair | Behavioral preservation goal is sound; its object-to-Map sample changes value shape, and removing async wrappers can alter promise/error behavior. Needs precise example changes. |
| constraint-driven-development | Manual integration | Explicit quality-contract setup can be valuable. Tools/hooks/ratchets require real configuration; the regex floor guard is a starting point, not a complete checker, and prints matched text despite redaction claims. |
| deprecation-and-migration | Manual repair | Consumer inventory, adoption windows and strangler transitions useful. New DDL/indexes are not universally safe, and down migrations do not recover lost data. Prefer authored migrations guidance. |
| doubt-driven-development | Manual workflow | Optional adversarial reviewer can be useful. Complete process includes personas, root reference and repeated cross-model prompts; never install as automatic per-branching-decision law. |
| frontend-ui-engineering | Manual repair | Basic frontend checklist overlaps authored skill; examples have non-modal `dialog open`, undefined `refetch`, omitted props and race-prone optimistic rollback. |
| git-workflow-and-versioning | Manual workflow | Atomic intent and consumer changelogs useful. Always-on auto-commit behavior, raw secret grep and `git reset --hard HEAD` recovery conflict with preserving unrelated work. |
| interview-me | Manual workflow | Useful only for an explicitly requested interview. Rejects ordinary clear confirmations/delegation, asserts an uncalibrated 95% threshold and forces stopping after approval. |
| shipping-and-launch | Manual repair | Staged rollout, budgets and health checks useful. Fixed thresholds/waits are not universal; rollback auto-push and incomplete ErrorBoundary example need repair. Prefer authored release skill. |
| spec-driven-development | Manual workflow | Structured specifications/capability maps useful when requested. Mandatory later-turn approval, task artifacts and downstream skill chain make this an explicit lifecycle alternative. |
| using-agent-skills | Manual full-package alternative | Router delegates whole workflow chains and introduces non-negotiable process. Conflicts with the personal chooser and task-based judgment if loaded globally. |

### Required shared-file handling

The [upstream README itself identifies](https://github.com/addyosmani/agent-skills/blob/1401c8b8030e023baeebb31781a6653fe8e93026/README.md) the single-skill portability problem: a skill directory alone omits root `references/`. The catalog includes an `extra_files` map `{source-root-relative: installed-skill-relative}` and ordered literal `replacements` with `path`, `old`, `new`, `count`.

- Planning and incremental implementation need `definition-of-done.md`.
- Observability needs `observability-checklist.md`.
- TDD needs `testing-patterns.md`.
- Review needs `security-checklist.md` and `performance-checklist.md`.
- Security needs `security-checklist.md`; nested `references/hardening-patterns.md` links must be remapped too.
- Performance needs `performance-checklist.md`; nested `references/optimization-patterns.md` links must be remapped too.

Eight per-package extra-file copies and ten exact substitutions were checked against the pin. The shared files above have no further required local path references outside those packages; references to other skill names are optional follow-up guidance, not a requirement to install the entire source. Copy root LICENSE through the normal license mechanism. Keep original and transformed hashes separately.

The copied security/performance reference corrections also apply to **code-review-and-quality**: its catalog integration note explicitly covers redacted secret scanning, response allowlists, CSRF/CORS distinctions, jurisdiction-specific privacy and security-preserving performance work. The note applies to all bundled references as well as the main body.

### Runtime/plugin boundary

Native [Claude manifest](https://github.com/addyosmani/agent-skills/blob/1401c8b8030e023baeebb31781a6653fe8e93026/.claude-plugin/plugin.json) exposes skill and command directories; [Codex manifest](https://github.com/addyosmani/agent-skills/blob/1401c8b8030e023baeebb31781a6653fe8e93026/.codex-plugin/plugin.json) exposes skills. Installing selected Markdown is not equivalent to installing all plugin commands/personas/hooks. The session-start script is explicitly **not wired by default** for hosts with native discovery. Optional `sdd-cache` stores prompt-shaped summaries keyed only by URL, so a 304 does not prove an earlier summary answers a different question. Optional `simplify-ignore` mutates files in place during Read and documents crash/rename recovery limitations. Neither hook should be inherited by the personal harness without a separate explicit integration review.

Most selected Markdown is portable to macOS/Windows and Claude/Codex after tool/path adaptation. Bash/jq/curl/brew hook/setup examples and `/dev/null` floor-guard logic are not native Windows instructions. No runtime package is required merely to read the selected conceptual guidance.

## Remotion: all skills and disposition

All 12 are retained as manual reference entries under `NOASSERTION` pending license clarification. Exact paths are `skills/<name>`, and all declare version `4.0.533`. The runtime installed version must match the documented APIs; a generic v4 label is insufficient.

| Name | Coverage/use | Runtime/integration caveat |
|---|---|---|
| remotion-best-practices | All-in-one Remotion router | Includes every other skill as nested REFERENCE.md trees (141 files, ~1.17 MB). Choose bundle or focused entries; installing both duplicates guidance. |
| remotion-create | New video project/composition | Node/Git, scaffold at latest and install packages. Preserve existing directory/lockfile; honor artifact request instead of mandatory Studio-only preview. |
| remotion-markup | Compositions/timelines, deterministic frame animation, audio/video/images/text/effects | 66-file package, ~605 KB including duplicated maps. Preserving frame determinism is strong advice; Studio-editable source constraints are specific, not general React style. |
| remotion-interactivity | Source-node editing, schema controls, connected compositions | `calculateMetadata` sample incorrectly uses async `useMemo`; construct the required callback, do not copy as-is. |
| remotion-captions | Whisper transcription, Caption JSON, SRT, captions styling | WebGPU/ONNX/model download and compatible platform; English default requires explicit multilingual model/language changes. Basic Captions fetches remote source with a separate license boundary. |
| remotion-maps | Static, MapLibre, Mapbox, MapTiler, Cesium terrain/city flyover | 32 files, ~488 KB with scripts/assets/data. Provider keys, billing, GL capture, attribution and geography provenance. MapTiler asks to hide logo/use unrestricted key; override with provider requirements. |
| remotion-multimedia | Audio/video duration, dimensions via Mediabunny | Can be used in Node/Bun/browser; dispose inputs and inspect supported codecs. No need to scaffold React for standalone metadata queries. |
| remotion-docs | Public Algolia search and `.md` docs retrieval | Query leaves device; embedded search-only key is not a user's private credential. Prefer existing fetch/search capabilities. |
| remotion-render | Video/still/image frames and transparent codecs | Compatible local CLI/headless browser. Transparent VP9 section's metadata sample says vp8; verify container/codec. Output must be inspected. |
| remotion-studio | Live preview | Long-lived local server/browser, actual printed port, process ownership. Do not start duplicates or unrequested exposed servers. |
| remotion-saas | Player, framework templates, server/cloud rendering | Chosen provider/account/credentials/cost/auth policy. Cloud deployment is separately authorized; source calls Lambda fastest/scalable without evidence. |
| remotion-upgrade | Version alignment and auxiliary package compatibility | Explicitly runs skills update for installed skills; would bypass personal pin/adaptation provenance. Never run the skill-refresh part on catalog-managed payloads. |

Useful techniques: frame-derived animation for deterministic rendering; rendered-frame checks as distinct from smooth Studio playback; treat composition metadata/timing as data; preserve caption whitespace/timestamps; choose static map plates when moving live tiles shimmer; report geography provenance; separate preview Player from production render service; inspect in-flight external rendering cost/retry/output privacy.

MapLibre/Mapbox examples remove map attribution controls while prose says preserve attribution, do not remove resources on cleanup, fetch workers remotely, and truncate antimeridian MultiLineStrings to a longest segment. These are concrete reasons to inspect selected techniques and resulting frames instead of assuming templates are production complete. The MapTiler/Cesium preprocessors and full TSX asset bodies were not fully reviewed; this is explicitly reflected in coverage, not presented as validation.

## Verification and limits

- Git commit pins captured directly from both read-only clones.
- Canonical paths, frontmatter names, descriptions, package file/byte counts enumerated into [source-index.json](../../registry/source-index.json).
- Project candidate extra-file existence and exact replacement occurrence counts checked against pinned source.
- Full selected instructions and companions reviewed, including optional idea initializer (not executed).
- No examples, runtime compatibility, GPU transcription, media output, provider credentials, cloud resources, Chrome MCP, optional hooks, or quality gates were executed/validated.
- Repository popularity and claimed productivity ratios were not used as selection evidence.

## Persistent registry integration

Source pins, disposition, paths, dependencies, caveats and exact adaptations are recorded in [the catalog](../../registry/catalog.json); physical source entries and canonical/mirror distinctions are in [the source index](../../registry/source-index.json). Project payloads have separate raw and installed hashes. Manual entries are discoverable references and are excluded from installable profiles. No runtime validation is implied by a source hash or static review.
