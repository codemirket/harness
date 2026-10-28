# Rewrite planning decisions

Status: the revised [consolidated plan](implementation-plan.md) governs the current `/Users/nazmiyilmaz/Projects/harness` checkout. Later decisions supersede the chronological entries below. Decisions 101–102 assign SQL and general browsing to Codex/project tools, removing Mirket's Python/SQLGlot and general-browser routes. Current local Mac qualification does not close public distribution, OS-floor or Windows gates. The subsequent deep-review authorization permits focused verified local commits; pushes, publication and live installation changes remain excluded.

## Initial constraints (historical)

- Fresh repository: `/Users/nazmiyilmaz/Projects/harness-rewrite`.
- Clean 0.0.1 with no legacy compatibility or migration burden. This is the intended capable product, not a deliberately reduced MVP; choose capabilities by user outcomes and engineering fit, not version number.
- Preserve the existing project, worktree and commit. No push or publication.
- Keep the harness powerful while reducing operational and conceptual complexity.
- Keep the selected references in `references.md`; source selection is not dependency or installation approval.
- Implement the approved plan and admitted dependencies; no commits or publication are authorized.
- Use tools running directly on macOS. Package private tool binaries and runtimes when manageable; acquire bulky fixed assets during setup under exact pins. A burdensome native tool may be selected from the host only through explicit identity and feature checks, with drift detection. Do not rely on an unqualified ordinary tool installation.
- Prefer headless operation when reliable; allow visible operation where needed. Headless is not a mandatory capability filter.

## Decisions — 2026-10-24 onward

Later decisions supersede the identified parts of earlier entries. Decision 79 is the current Office ownership contract, decision 80 makes Codex Desktop the sole first-release host target, decision 81 defines its exclusive language-tool scope, and decision 82 extends the bounded SQL profiles. Historical engine, packaging and host choices are decision history, not active implementation requirements.

### 1. Host-owned agents

Codex owns model selection, agent creation, coordination and execution lifecycle within Codex Desktop's supported capabilities and permissions. Mirket supplies guidance, project memory and specialist tools. It does not implement model-provider adapters, agent routing/budgets, an agent spawner/scheduler or model-worker checkpointing. Mirket owns the tool processes it launches; their cleanup and sessions are distinct from host-agent lifecycle.

### 2. Host execution with privately packaged tools — replaces selective VM placement

The user explicitly replaced the VM plan because of resource cost. Ordinary commands, Git, project builds, source, guidance, memory and specialist tools run on the host. Required specialist tool executables and their distributable dependencies belong to the Mirket package, using their own configuration, profiles, caches and temporary files where supported. They must not silently reuse or modify the user's normal browser/design installations or sessions.

This is installation and application-state separation, not OS sandboxing. Processes share macOS, hardware resources and the authority granted by the host. Normal macOS frameworks/services remain platform dependencies. Exact tool compatibility, native dependency closure and avoidance of shared preference/single-instance conflicts must be verified per tool.

### 3. macOS Apple Silicon first

Originally 0.0.1 targeted macOS Apple Silicon alone. Decision 69 supersedes Windows deferral: macOS Apple Silicon and native Windows are both first-release targets, with Codex Desktop as the intended client subject to per-platform qualification. Intel macOS and native Linux host releases remain deferred. Minimum macOS/client versions and signing/distribution details remain open.

### 4. Rust core + TypeScript tool adapters — placement revised

Retain the chosen languages: Rust for CLI/MCP, projects, memory and application behavior; TypeScript for browser/design adapters, now launched on the host. Decision 36 selects a private packaged Node.js LTS runtime for compiled adapters, without relying on a user-installed runtime. Its exact version and dependencies remain open. This is fresh implementation, not a port of the old runtime.

### 5. Private tool distribution — replaces Podman choice

The user requires Mirket's own copies of tool binaries/executables in package content. This means pinned distributable upstream tools or approved builds, not an obligation to fork upstream projects. No VM/container manager, guest image or GPU forwarding stack belongs to the current baseline. Decision 7 selects one complete release bundle; optional tool packs are outside the 0.0.1 packaging design.

### 6. Prefer reliable headless execution

Use CLI/library/headless modes for supported tasks. Visible operation is allowed when required for capability, correctness, authentication or requested inspection. It must use Mirket-owned state; opening a normal personal application/session is not an implicit fallback. An unsuccessful headless operation is not automatically replayed visibly when effects may already have occurred. Exact tool/session lifetime remains open.

### 7. One complete release package

The user chose option 1. Ship all selected 0.0.1 specialist tools and their required distributable runtimes together as one tested release, with one installation/update path. This does not include every retained reference project or settle the tool inventory. The package can contain multiple executables and app bundles; it need not be a single executable. Exact archive/installer format, signing and installation location remain open.

No optional-pack manager or first-use dependency downloads are proposed for this release. Installation includes tool binaries; launching them remains on demand, with process/session lifetime still to be decided. Larger download and disk footprint are accepted tradeoffs, not a requirement to run every tool at startup. This planning choice does not authorize dependency installation or release publication.

### 8. Local previews + public-web browsing

The user chose option 1. The packaged browser supports local project previews and public-web workflows: navigation, content extraction, screenshots and permitted interactions. General web browsing is included in 0.0.1, alongside engineering checks. This does not select a crawling platform or promise unrestricted crawling scale.

Decision 9 selects optional saved logins alongside temporary sessions. External actions still follow the user's authorization and host permissions; including browser interaction does not itself authorize submissions, purchases or publication. Network contracts, process/session lifetime and concrete browser dependencies remain open.

### 9. Temporary sessions by default + optional saved profiles

The user chose option 1. Normal browsing starts with a temporary session. The user may explicitly select a Mirket-owned saved profile, sign in through its dedicated browser window and reuse that profile later where the website permits it. Personal browser profiles/cookies are not imported or selected implicitly. This does not add a Mirket password manager or authorize account access during planning.

Saved login state stays outside the release package, project source and diagnostic exports. Decision 10 selects named profiles reusable across projects by explicit selection. Storage/deletion, concurrent-client ownership, expiration/MFA and headless/visible transitions remain implementation requirements to specify and verify. A saved login does not itself authorize future external actions.

### 10. Named saved profiles reusable across projects

The user chose option 1. Maintain one Mirket-owned collection of named browser profiles, such as Work or Client A. A session in any project can explicitly select a profile; fresh temporary sessions remain the default. Do not duplicate logins per project or infer an account from the current project. Personal browser state remains separate.

Cross-project availability does not imply concurrent access to one profile directory or shared live tabs across clients. Enforce exclusive ownership while a saved profile is in use; the precise busy/ownership contract remains an implementation detail to specify. Profile persistence is distinct from live browser-session lifetime, selected in decision 11.

### 11. Browser sessions spanning tool calls

The user chose option 1. Keep a live browser session across navigation, inspection, interaction and capture calls, owned by the initiating client connection. Provide explicit close, bounded idle cleanup and cleanup when that connection terminates. Temporary session state is discarded at session end; explicitly saved profiles persist after browser closure.

Protect active operations and intentional user interaction from idle termination. Exact idle timing/hold behavior, concurrency limits, cancellation semantics and crash cleanup remain to be specified. No cross-client sharing of live tabs, permanent browser daemon, durable browser-resume service or automatic replay after uncertain effects is implied. Saved-profile access remains exclusive while in use.

### 12. Editable design documents included

The user chose option 1. Include creating, inspecting and editing layouts/components in editable design documents, with preview and asset export, alongside code-based interface work. Prefer reliable headless file operations; visible use is allowed when needed under the established private-state requirement.

OpenPencil CLI/SDK remains the proposed implementation, subject to macOS ARM64 packaging, redistribution, coexistence and output-fidelity verification. This capability decision does not approve a particular dependency/version or promise complete Figma compatibility. Decision 43 selects the expanded output set; exact native/interchange format contracts, dependencies and representative acceptance artifacts remain to be specified.

### 13. Interface animation + standalone video export

The user chose option 1. Include authoring and inspecting interface animations, plus rendering authored compositions into standalone video files for demos, explainers and marketing. Video rendering/encoding dependencies must be included in the complete Mirket package if required by the selected implementation; they are launched on demand.

Remotion remains a retained reference, not an approved dependency/version. Decision 48 and the revised toolchain proposal supersede its runtime candidacy for the free/open-source baseline. Decision 41 selects audio and caption support. Decision 42 selects export formats; exact engine/codecs, input formats, frame/duration/resolution limits, hardware encoding assumptions, package size and resource limits remain open. Browser test recordings are distinct from authored video production. Verify actual rendered playback and cancellation rather than treating a successful render command as sufficient.

### 14. PDF + editable Office documents

The user chose option 1. Include document deliverables in PDF, DOCX, PPTX and XLSX: reports, presentations and spreadsheets, with editable Office-file authoring/editing and finished PDF output. Required distributable tools/runtimes belong in the complete Mirket package, preserving the host-tool/private-state design.

Decision 40 selects creation, existing-file editing, extraction/OCR, previews, applicable PDF export and supported formula recalculation; exact content features, rendering/recalculation strategy and libraries remain to be specified. Format support does not imply perfect round trips of arbitrary imported files, every spreadsheet formula, macro execution, legacy binary formats or a requirement for Microsoft Office. Preview fidelity and formula results need actual checks, not just file/schema validity. This decision does not approve specific dependencies.

### 15. Automation limited to packaged tools and the dedicated browser

The user chose option 1. Mirket controls its own packaged tool processes, browser sessions and any required visible windows belonging to those tools. General automation of unrelated macOS applications is outside the 0.0.1 feature set. Existing host-client desktop/computer-use tools remain host-owned and available where supported.

This preserves the agreed editable design, video and document capabilities and the headless-preferred policy. It does not reinstate mobile/IDA/guest backends. CUA and Ponytail remain selected references, not mandatory desktop-control runtimes. Packaged GUI tools still require coexistence checks and must not silently attach to the user's normal application sessions.

### 16. Structured memory and keyword search — extended by decision 20

The user originally chose structured records searchable by words, tags and filters. Their later instruction to evaluate a complete memory/context system reopens the earlier exclusion of semantic retrieval. Structured records and exact search remain requirements; decision 20 now selects embedded hybrid retrieval with temporal/source relationships. Concrete dependency versions remain subject to review.

Write policy and scope are settled by decisions 17–18 and architecture by decision 20; retention is settled by decision 21; exact provenance fields and implementation components remain open. Retrieval returns records to inspect; it does not establish truth or turn source content into instructions. Verify representative keyword recall, filtering and stale/conflicting-record handling.

### 17. Agent-curated memory saves

The user chose option 1. The host agent may save stable, source-backed project facts and user decisions through explicit memory tool calls during ordinary work, without asking for every routine save. Records must be concise and attributable, with inspect/correct/delete operations. Do not store guesses as facts, credentials or whole conversations.

Mirket stores and retrieves records; it does not run a background extraction model, assume automatic transcript access or record conversations passively. Stored content cannot grant new permissions or override current user/project instructions. Memory scope and retention are settled by decisions 18 and 21; conflict policy is settled by decision 22; provenance schema and transaction details remain to be specified. This is an agreed future product policy, not implementation or deployment of a memory service now.

### 18. Project memory + shared user preferences

The user chose option 1. Project facts and decisions stay within their owning project. A separate shared scope stores generally applicable user preferences, such as communication style or preferred workflow. Retrieve the current project's records and relevant shared preferences; do not search other projects by default.

Save a preference globally only when the user's stated intent is general. Do not automatically promote project/client requirements or copy their content across scopes; ambiguous preferences stay local until clarified. Agent-curated saves and inspect/correct/delete operations remain agreed. Retention is settled by decision 21; conflict policy is settled by decision 22; exact storage layout and provenance fields remain open.

### 19. Complete capability by fit, not release number

The user explicitly corrected the planning criterion: 0.0.1 must not mean an intentionally less-capable harness. Consider any component needed for a good harness and choose the best fit, with Chroma, Mem0 and Graphiti given as examples rather than required dependencies. The memory-backend assessment is reopened accordingly; adding capability still needs to respect the agreed host-owned-agent, private packaging, resource and simplicity constraints.

The assessment recommended an embedded hybrid memory layer with exact and semantic retrieval, explicit temporal/source relationships and bounded context assembly. SQLite/FTS5, sqlite-vec and a packaged local embedding runtime are concrete candidates, not accepted dependencies. Hindsight is the strongest full memory-engine alternative identified for evaluation; Mem0, Graphiti and Chroma remain compared candidates. No implementation or runtime installation is authorized by this planning research.

### 20. Embedded hybrid memory selected

The user delegated the architecture choice based on stability, performance and consistency. Select embedded hybrid memory: canonical local SQLite records, exact/keyword and semantic retrieval, explicit current/historical validity and source relationships, and bounded context delivery. This fits host-agent curation and privately packaged host execution without a separate memory service. It is an architecture choice, not a measured claim of superior speed or recall.

Canonical records are authoritative; vectors and search indexes are derived and rebuildable. Require atomic record changes and supersession, revision checks for concurrent edits, and retrieval that excludes deleted or superseded records from current answers even while derived indexes catch up. Unresolved contradictions remain explicit. Package local embedding assets; do not download them on first use or add a second generative memory agent.

SQLite/FTS5 is the selected storage direction. Exact SQLite bindings, vector component, embedding runtime/model and versions still require concrete review; sqlite-vec and FastEmbed-rs are candidates, not admitted dependencies. Keep one implementation, with no parallel full-engine backend or fallback framework. Retention is settled by decision 21; conflict policy is settled by decision 22; transaction details, supported scale and resource budgets remain to be specified and verified. Complete-plan agreement still precedes implementation.

### 21. Durable memory retained until corrected or explicitly forgotten

The user chose option 1. Durable facts, decisions, preferences and verified lessons do not expire merely because time passes. Corrections supersede earlier records; superseded records remain available for historical queries and are excluded from current answers. Retention does not establish continued validity: stale or uncertain evidence remains marked accordingly.

Explicit forgetting removes the targeted content from canonical memory and associated derived retrieval data, including retained revisions within the requested scope. It must not survive as searchable history or reappear during index rebuild. Temporary task data, logs and caches have a separate bounded cleanup policy; backup/recovery policy is settled by decision 23; precise cleanup limits and recovery mechanics remain to be specified. Cleanup must preserve durable records and recovery state needed by active work.

### 22. Clear corrections supersede automatically; genuine conflicts stay explicit

The user chose option 1. The host agent may apply an unambiguous, source-backed correction through the normal memory-write tools without requesting confirmation for each update. An explicit user change supersedes the earlier decision within its applicable scope; preserve the earlier version as history under decision 21. A project-specific exception does not silently replace a shared preference.

When evidence genuinely conflicts or the intended replacement is unclear, retain the competing claims with provenance and an unresolved status. Neither recency, semantic similarity nor model confidence alone determines which claim is authoritative. Ask the user only when resolving the ambiguity affects the current task; otherwise preserve it and continue independent work. Mirket validates scope, revision and explicit supersession links; the host agent supplies the interpretation. Concurrent revision conflicts must not silently overwrite records.

### 23. Automatic bounded local backups plus manual export/import

The user chose option 1. Automatically create consistent local memory backups with bounded retention, and support explicit manual export/import. Keep this within the existing local runtime; no cloud storage, synchronization service or always-running backup daemon is required. Decision 60 selects up to seven daily and four weekly recovery points plus pre-migration snapshots, under a configurable storage cap. Decision 61 selects rotation of eligible older snapshots at the cap. Decision 62 sets the configurable default cap to 5 GiB total across project-memory and shared-preference backups. Exact trigger mechanics, temporary-space accounting and archive format remain implementation-plan details.

Decision 63 selects replacement from the chosen recovery point as the default restore behavior. Restore and import must preserve project/shared-preference scopes, provenance, revisions, supersession and unresolved conflicts. Validate the archive before changing live state and preserve recoverable current state if replacement fails. Rebuild derived indexes from canonical records instead of treating old vectors as authoritative.

Explicit forgetting must cover Mirket-managed backups: remove or invalidate affected snapshots so they cannot restore forgotten content. Retain only content-free deletion identifiers where needed to reject known deleted records from older imports. Exported copies outside Mirket's control cannot be retroactively erased; do not promise that a fresh installation can infer deletions absent from its recovery data. Specify this boundary and the recovery format before implementation, without retaining forgotten content in deletion metadata.

### 24. Multilingual text and code retrieval

The user chose option 1. Require multilingual memory retrieval, including English-to-Turkish and Turkish-to-English recall, alongside exact code identifiers, paths and technical terms. Preserve original source text and provenance; cross-language matching must not silently translate or rewrite canonical records. Other languages may be supported by the selected model, but tested coverage must be reported rather than assuming universal parity.

Select the packaged local embedding model against representative bilingual, mixed-language and code-heavy fixtures. Measure retrieval quality, cold/warm latency, peak/idle memory and package size. Compare hybrid retrieval with exact-only retrieval on the same cases, including misleading near-matches, scope separation and superseded facts. This selects a capability requirement, not a particular model or new dependency.

### 25. Proactive focused recall for relevant work

The user chose option 1. Shared guidance directs the host agent to retrieve focused project context and applicable shared preferences when starting or resuming substantial work, and to retrieve further evidence when the task needs it. Skip trivial or self-contained requests; reuse retrieved context while its scope and source revisions remain applicable instead of repeating unchanged searches on every turn.

Use explicit memory tools with bounded, source-linked results. Current user/project instructions retain authority; recovered state is evidence, not permission to replay actions or assume incomplete work succeeded. A missing or incomplete retrieval result must not be described as complete memory coverage. Mirket supplies guidance and tools but cannot guarantee that every host invokes them, access transcripts passively or force host-context compaction. Verify actual invocation and usefulness in Codex Desktop.

### 26. Twelve on-demand specialist guides and one shared workflow

The user accepted the expanded skill structure: Engineering; Verification; Research and context; Interface and visual design; Motion; Product and marketing; Documents and artifacts; Security and privacy; Reliability and delivery; Data and analysis; AI systems and evaluation; Planning and coordination. Their responsibilities and boundaries are specified in proposal section 6.

Use one short shared workflow and central permission rules. Each guide has a concise entry document, focused supporting references, activation conditions, expected outputs, relevant evidence and stopping criteria. Load only guidance needed for the task. Strengthen existing guides with the agreed accessibility, performance, database/API, source verification, memory maintenance, discovery/measurement and extraction/OCR responsibilities rather than creating a top-level guide for every subtopic.

The retained reference catalog informs original authored guidance; exact source mapping and reuse review still apply. This acceptance selects guide organization and scope, not wholesale upstream instruction packs, new dependencies, runtime services or a separate agent framework. Planning and coordination uses host-owned agents. Complete-plan agreement still precedes implementation.

### 27. One Rust crate, one TypeScript adapter package and one coordinated release

The user chose option 1. Keep the Rust CLI/MCP and core behavior in one crate with focused modules, and specialist TypeScript adapters in one package. Share packaging definitions, skills, tests/fixtures and documentation within the same repository and ship one coordinated release. The directory structure in proposal section 7 is the agreed organizational direction; exact module filenames may follow implementation responsibilities.

Use module and adapter contracts to keep ownership clear without creating multiple Rust crates in advance. Split a crate only if a concrete engineering need justifies it later; do not introduce service or compatibility layers to simulate separation. Keep downloaded binaries in assembled release/build artifacts rather than committed source. Node.js LTS is selected in decision 36; its concrete version, the package manager and dependency choices remain pending. No application scaffold or implementation is authorized before complete-plan agreement.

### 28. Per-user installation and user-owned state

The user chose option 1. Install the complete Mirket package and its private tool/runtime copies in user-writable locations, with writable memory, profiles, caches and operational state owned by that macOS user and separate from release contents. Ordinary installation and updates must not require administrator access. Do not add a system-wide shared installation or privileged helper.

This selects installation scope, not exact directory paths or package format. Preserve unrelated applications and settings; native OS permissions required by individual tools remain applicable. Client configuration changes must remain scoped to the intended user's clients. Verify fresh install, update and removal under an ordinary non-administrator account; removal of durable user data requires explicit intent distinct from removal of package binaries.

### 29. Explicit verified updates through mirket update

The user chose option 1. Apply updates only through an explicit `mirket update` request. Verify the complete release before activation; do not replace installed components opportunistically during ordinary tool calls or install updates automatically when idle. Decision 66 requires explicit requests for update discovery as well; no startup or background update checks.

Active jobs remain bound to their existing release until they finish, including its executables, adapters and assets. Do not remove release files still needed by running work. Interrupted update recovery must preserve memory, profiles, project files and unrelated settings. Coordinate shared-state migrations with active clients/jobs; versioned binaries alone do not make concurrent old/new state access safe. Exact activation, migration and recovery mechanics remain to be specified before implementation, without adding legacy compatibility layers.

### 30. Focused code navigation with host-owned editing and execution

The user chose option 1. Include symbol search, go-to-definition and reference lookup as Mirket capabilities, available through the same tool contract to Codex Desktop. Editing, command execution, Git and project toolchains remain host-owned. This does not add a coding agent, general refactoring engine or Mirket command runner.

Language/file coverage is agreed in decision 31; concrete parser/language-server dependencies remain to be selected; decision 33 settles the indexing policy. Serena remains a guidance reference, not an automatically selected runtime. Navigation must identify the project, source path and applicable revision, distinguish syntax matches from resolved symbols, and report incomplete or unavailable analysis rather than presenting text matches as exhaustive semantic results. Inspect current source before an agent acts on returned locations. Decision 32 defines the execution policy: read-only project analysis by default, with required project execution delegated to an explicitly authorized host command; navigation capability alone does not authorize it.

### 31. First-release language scope — revised by decision 81

The earlier broader inventory is superseded by decision 81. Use its exclusive named target set and the current bounded SQL profiles under decision 82. The [original language research and decisions](reviews/2026-10-25/superseded-language-tooling.md) are historical evidence, not additional support obligations. Syntax/structure, source positions and full semantic navigation require separate qualification.

### 32. Read-only analysis by default; required execution stays host-owned

The user chose option 1. Navigation inspects project sources and available metadata without modifying project files or implicitly executing build scripts, repository plugins, generators, macros or other project code. Analyzer-owned indexes and caches may be written to Mirket-owned state; read-only describes the project boundary rather than prohibiting internal caching.

If accurate navigation needs generated files or build metadata, return the missing prerequisite and explain the required host action. The host agent may execute the concrete command under explicit applicable user authorization and host permissions, then refresh navigation from the resulting files. Honor existing authorization rather than asking again for the same authorized action. Mirket does not add a command runner or silently let an analyzer perform the action itself.

Configure and verify each analyzer's execution behavior; do not assume that a nominally read-only protocol request prevents build/plugin execution. Where the required behavior cannot be disabled, disclose the limit and select an appropriate alternative rather than treating navigation permission as code-execution permission. Incomplete results remain explicit until required inputs are available. This is a behavior contract under host permissions, not a claim of OS sandboxing.

### 33. On-demand incremental indexing with reusable local caches

The user chose option 1. Build navigation indexes when needed, update affected data incrementally and reuse valid caches across sessions. Keep caches in Mirket-owned local state, scoped to the actual project/worktree and analyzer identity. No permanent indexing service is required; stop idle analyzers while preserving reusable cache data.

Validate cached results against current source and relevant configuration before presenting them as current. Changes to files, branches, project configuration, dependencies or analyzer versions invalidate affected analysis; refresh dependent symbol/reference information as required, not only the edited file. Deleted or renamed files and separate worktrees must not leak stale locations or results across contexts.

Reuse analyzer-native incremental indexing and persistence where supported. Do not build a second semantic indexing engine to imitate unavailable upstream persistence; an analyzer may need to rebuild its internal state. Invalid or corrupt derived caches are rebuildable. Report incomplete analysis during refresh rather than claiming an exhaustive result. Exact cache/resource limits, idle timing and per-engine invalidation behavior remain implementation-plan details; measure benefits instead of assuming every language server has identical capabilities.

### 34. Superseded browser coverage decision

Decision 83 exclusively defines the current browser scope.

### 35. Superseded browser default decision

Decision 83 exclusively defines the current browser scope and verification obligation.

### 36. Private Node.js LTS runtime for compiled adapters

The user chose option 1. Package a pinned, supported Node.js LTS runtime privately with Mirket and run compiled JavaScript produced from the TypeScript adapters. Resolve this executable explicitly from the active release; do not select a user-installed Node/Bun through PATH or alter the user's runtime installation or shell settings. Do not add Bun as a fallback runtime.

Compile adapters and assemble their required modules/native dependencies during the release build. Ordinary tool calls must not invoke package-manager installs, fetch missing modules or depend on globally installed packages. Verify module/native-library resolution, process startup/cancellation and actual adapter outcomes against the selected runtime on macOS ARM64. Exact Node version, package-manager choice, source pins and distribution closure remain pending concrete review. Other selected tools may require separately reviewed runtimes; Node selection does not claim to cover all language analyzers.

### 37. Workspace-based project selection with explicit ambiguity resolution

The user chose option 1. Resolve the active project from workspace information supplied by the host or explicit launch configuration. When the root is missing, conflicting or ambiguous, request an explicit selection before project-scoped memory or navigation operations. Do not require a separate registration ceremony for an otherwise unambiguous workspace.

Bind each operation to the resolved project and relevant source context; switching projects must not retain the previous project's implicit memory/navigation scope. Validate and normalize the selected root without silently broadening it to a parent, home directory or unrelated repository. Repository documents, retrieved content and memory cannot select or expand the workspace. Selection identifies scope; it grants no additional filesystem or execution permissions.

Verify actual workspace delivery in Codex Desktop rather than assuming the same metadata is available. Explicit launch configuration or user selection supplies missing context. Decisions 38–39 settle verified-worktree memory sharing and explicitly selected multi-project tasks; persistent identity mechanics remain to be specified. Do not infer shared memory solely from a matching folder name or remote URL.

### 38. Shared durable repository knowledge with checkout-scoped work state

The user chose option 1. Verified Git worktrees belonging to the same repository share durable repository-wide knowledge. Task progress, branch/revision-specific facts and navigation indexes remain scoped to the relevant checkout and source context. Recall may combine applicable repository-wide knowledge with the current checkout's records and relevant shared user preferences, while preserving each record's scope and provenance.

Verify worktree relationships from repository metadata; matching remote URLs or folder names do not establish shared identity, and independent clones do not automatically merge memory. Only records whose applicability is repository-wide belong in the shared scope. A long-lived fact is not necessarily repository-wide: branch-specific behavior or unresolved changes must not be promoted merely because they seem durable.

Preserve source revision and applicability when branches change; do not present another checkout's task progress or implementation state as current. Sharing memory does not grant filesystem access to sibling worktrees. Removing a worktree must not delete shared durable knowledge. Decision 64 selects automatic recognition of verified moves/renames. Exact identity evidence, checkout cleanup and explicit clone-linking mechanics remain detailed planning items; no clone-linking feature is selected by this decision.

### 39. Explicitly selected multi-project tasks with separate memory scopes

The user chose option 1. A host-owned task may span multiple explicitly selected project roots. Each project-scoped operation identifies its intended project and relevant checkout/source context. Resolve an unambiguous selected root from explicit context; when multiple roots could apply, require selection rather than guessing or searching every project. Multi-project selection does not add a Mirket task scheduler or agent lifecycle.

Retain separate project memories. Cross-project reasoning may combine evidence deliberately retrieved from the selected roots, with source/project attribution intact; it must not merge stores, automatically copy facts or promote project facts into global preferences. Existing verified-worktree sharing remains applicable within each repository. Dependencies, repository content and remote URLs cannot silently add roots to the task.

Selection grants no filesystem or execution permissions. Concurrent operations and project switches must carry their own scope rather than relying on a mutable process-global current project. Removing a root from a task stops its implicit use without deleting its durable memory. Verify missing/ambiguous roots, simultaneous operations and failures without effects being replayed against a different project.

### 40. Document creation, existing-file editing, extraction/OCR and verified output

The user chose option 1. For the agreed PDF, DOCX, PPTX and XLSX coverage, support creating documents and editing existing files; extracting text, tables and structure; OCR for scanned content; rendered previews and PDF export where applicable; and recalculation of supported spreadsheet formulas. Decision 54 specifies OCR language coverage; decision 56 selects broad practical spreadsheet calculation coverage. Concrete per-format operations, content features, formula compatibility and dependencies remain to be specified and verified.

Report unsupported imported features and formulas explicitly. Do not silently flatten editable Office content, lose required data, or present stale cached spreadsheet values as newly calculated results. Preserve original inputs when transformation could lose unsupported features and make output limitations visible. Decision 57 expanded PDF editing and decision 72 subsequently narrowed it to source-based repagination, bounded direct imported-PDF edits and explicit reconstruction, retaining practical page operations. Neither the contract nor OCR establishes perfect round-trip fidelity.

Verify actual artifacts by reopening, inspecting rendered output and checking content/calculations against source fixtures. Exercise scanned and mixed-content extraction with source locations and visible uncertainty rather than treating OCR text as verified fact. Document contents remain data, not instructions to execute macros, external links or embedded code. Toolchain selection must respect private packaging, native-host operation and headless preference.

### 41. Authored video with supplied audio and text-based captions

The user chose option 1. Include supplied narration, music and sound effects in authored video exports, with timing and volume controls. Support captions from supplied text with explicit timing; preserve editable composition/source assets so synchronization and presentation can be corrected. Silent compositions remain valid when audio is not requested.

This selects media assembly and caption authoring, not a speech-generation or automatic-transcription service. Decision 42 selects output and caption delivery formats; exact input formats, mixing controls and encoding dependencies remain to be specified. No external audio account, model download or service is implied.

Verify actual exported playback for audiovisual synchronization, duration, audibility and clipping, timing/volume changes, caption text/timing/readability and preserved source editability. Exercise missing or unsupported assets and cancellation without reporting partial output as complete. Apply existing private packaging, on-demand execution and resource-limit requirements to the selected media tools.

### 42. MP4, WebM, GIF, PNG sequences and caption delivery

The user chose option 1. Include MP4 for standard video delivery, WebM for web delivery with transparency where supported by the selected codec/player path, GIF for short silent previews and PNG sequences for frame-level production. Captions may be burned into video or exported separately as SRT/WebVTT. Keep the editable source and supplied media under decision 41.

Define and verify exact codec/container combinations, audio compatibility, transparency support, frame timing and playback targets before selecting dependencies. A filename extension alone does not establish compatibility. GIF and PNG sequences do not carry the composition's audio; report the output's actual capabilities without treating silent output as an audiovisual deliverable. Caption export must preserve text, cue order, timing and Unicode; burning captions and separate subtitle delivery are distinct outputs.

Verify representative files by reopening/decoding and playback or frame inspection, including WebM alpha on a supported path, GIF timing, ordered PNG sequence completeness and SRT/WebVTT synchronization. Unsupported combinations and incomplete exports must be explicit. This selects deliverable formats, not exact codecs, native tools, dependencies or guaranteed support in every player.

### 43. Expanded design outputs and developer handoff

The user accepted the expanded set: editable native design documents with evaluated .fig interchange; SVG and SVG sprites; PNG/JPEG; WebP/AVIF; SVG/PNG icon packages, favicon ICO and web-app icon sizes; PDF; editable PPTX; HTML/CSS with assets; React/TypeScript components; Storybook stories where the project uses or explicitly selects Storybook; design-token JSON and CSS variables; and a handoff directory/ZIP containing source, selected exports, assets, tokens, manifest and usage/font/license notes.

Produce outputs when the task needs them rather than generating every format by default. Preserve supported layers, text, components and layout in editable sources; .fig interchange requires explicit fidelity testing and does not promise complete Figma parity. Report unsupported imported features and retain original inputs. Existing video, animation and caption scope remains governed by decisions 41–42.

Native export, format conversion and host-owned code implementation are distinct paths. Generated JSX is a starting point, not proof of production readiness. Code deliverables must follow the project's existing framework/styling conventions and be checked for responsiveness, accessibility, interactions and integration. No automatic framework installation follows from this decision.

Verify editable-source reopening, rendered fidelity, fonts, alpha/color handling, token consistency, icon/export completeness and actual component behavior. Handoff contents must identify assets and relevant usage/redistribution information. OpenPencil/Sharp and other concrete tool choices, versions and native dependencies remain pending review; acceptance selects deliverables rather than claiming any exporter already meets the full contract.

### 44. Packaged redistributable fonts and project-supplied fonts

The user chose option 1. Bundle a curated font set with verified redistribution rights and support project-supplied font files. Resolve authored design/document/video typography from those controlled sources rather than depending on the user's personal font installation. Keep font assets private to Mirket or the project; do not install them globally or change the user's font settings.

Report missing fonts, missing glyphs and substitutions explicitly. Additional font downloads require an explicit choice under existing authorization; ordinary rendering must not trigger hidden font-provider downloads. A renderer that cannot honor controlled font selection needs an explicit limitation or another implementation. Packaging alone does not establish equivalent shaping, line wrapping or embedded-font behavior across tools.

Decision 55 specifies the bundled language coverage and font roles. Exact families, weights/styles, mathematical glyph inventory, variable-font support and renderer integration remain to be selected and tested. Preserve source/version/license metadata; project supply does not itself grant redistribution or embedding rights. Include font files in exported handoffs or documents only when the applicable rights permit it. Verify rendered typography, fallback reporting and applicable export/embed behavior on actual artifacts.

### 45. Prepare authorized assets before local rendering

The user chose option 1. For authored design, document and video workflows, resolve required remote assets during an explicit preparation stage under the task's existing authorization, then render from a stable local asset set. Ordinary web browsing retains its separately agreed network behavior. Do not introduce repeated approvals for downloads already covered by the task; additional font selection still follows decision 44.

Preparation records asset source/content identity and applicable usage information, discovers required nested assets where supported, and reports missing or inaccessible resources before final rendering. Do not silently fetch replacement content, refresh assets midway through a render or fall back to live URLs. Preserve original project sources; prepare a render workspace or asset mapping rather than overwriting the user's remote references incidentally.

Final rendering uses packaged/project/prepared assets and any required owned local preview endpoint, without depending on live external downloads. Verify this with external network access unavailable and with request diagnostics appropriate to the renderer. Configure selected renderers to suppress unexpected external fetching where supported, and report unsupported guarantees rather than claiming a custom OS network sandbox. Missing assets require preparation or an explicit changed requirement, not a successful-looking partial export.

### 46. 16 GB Apple Silicon performance reference

The user chose option 1. Use a 16 GB Apple Silicon Mac as the reference configuration for performance evaluation of the full agreed feature set. Bound concurrency and leave capacity for the active host client and representative normal applications. This is a test/design target, not a measured performance claim, a strict minimum-system requirement or permission to remove capabilities.

Record the exact chip, OS, host-client versions, workload sizes and concurrent application conditions with benchmark results. Measure cold/warm latency, idle/peak process-tree memory, memory pressure/swap behavior, CPU use, package/cache size and responsiveness during representative browser, indexing, memory, document and video workloads. Include combined workloads rather than measuring each tool only in isolation.

Decision 47 selects throughput-first concurrency; concrete memory budgets, parallel-job counts, queue/idle limits and acceptance thresholds remain to be specified against representative fixtures and validated after implementation. Do not infer that all engines/analyzers/renderers can run concurrently merely because each succeeds alone. Higher/lower-memory behavior and the actual minimum supported platform remain separate planning decisions.

### 47. Throughput-first bounded concurrency for heavy work

The user chose option 2. Permit multiple heavy jobs, including render/export work, to run concurrently by default rather than serializing all heavy work behind a single slot. Prioritize throughput while accepting the possibility of greater memory pressure and reduced interface responsiveness. Keep concurrency configurable and bounded; this does not authorize unlimited processes or simultaneous activation of every packaged tool.

Select actual job counts and resource limits from representative measurements on the agreed 16 GB Apple Silicon reference with host/client applications present. Respect tool-internal parallelism when measuring total load so independent job caps do not conceal a much larger process/thread count. Queue work beyond the configured capacity with visible state and cancellation; preserve per-job ownership, output integrity and active browser-profile exclusivity.

This changes tool-job scheduling defaults, not host-agent ownership or the browser scope in decision 83. It adds no permanent agent scheduler or broker. Verify simultaneous heavy workloads, cancellation of one job without affecting others, resource exhaustion reporting and complete-versus-partial output states. Do not claim a measured throughput gain before testing.

### 48. Free and open-source tool selection

The user requested a deep review of the current toolchain and selection of free, open-source licensed tools, explicitly including HyperFrames and PhotoCraft as alternatives. Use that licensing requirement when revising the component proposal. Evaluate actual capability, performance evidence, maintenance and packaging fit rather than popularity alone.

This does not mean permissive licenses only, does not select the new repository's license, and does not automatically approve exact production dependencies, model weights, brand assets or implementation. The accepted macOS and host-owned agent topology remains. The proposal must distinguish an OSS project's code from proprietary binary components, paid services and separately licensed assets. Preserve the retained reference catalog; a reference need not become a bundled runtime.

Specific recommendations, comparisons and feasibility gaps are recorded in `toolchain-proposal.md`. HyperFrames is the recommended video direction; PhotoCraft is evaluated but not recommended for the baseline. Decision 49 accepts the component direction; complete-plan agreement and concrete dependency review remain outstanding.

### 49. Accept the consolidated open-source toolchain direction

The user accepted the revised toolchain plan and its comparisons. Use the component ownership table in `toolchain-proposal.md` as the agreed direction: Rust/official MCP SDK, private Node.js LTS/TypeScript adapters, Playwright and the three agreed browsers, Readability, OpenPencil, Sharp/libvips, resvg/SVGO, Style Dictionary, D2/ELK, CSS/WAAPI with Motion/Anime.js as appropriate, HyperFrames/FFmpeg, LibreOffice/UNO, Typst, pypdf/PDFium, narrow Docling/Tesseract, embedded SQLite hybrid memory/local multilingual embeddings, curated fonts and selected upstream language servers.

This includes the justified private Python document-runtime exception to the Rust/TypeScript application envelope. Decision 73 explicitly separates the general extraction interpreter from Office’s matched Python/UNO. Verify each runtime’s own compatibility; do not introduce further interpreters silently. The baseline remains Docling. Evaluate docling.rs as the leading replacement candidate and Xberg as a secondary challenger; replace the displaced stack only if required output quality and measured resource behavior justify the change. Do not ship duplicate fallback engines.

HyperFrames replaces Remotion as the selected video direction, using an OSS animation path. PhotoCraft is excluded from the baseline. Prefer the upstream native TypeScript LSP, subject to actual lifecycle/framework compatibility. Retained reference selections remain unchanged.

Acceptance covers the proposed component direction and verification approach, including a two-heavy-job experiment; it does not establish measured performance, production concurrency limits, exact pins/build/model/license closure or complete-plan acceptance. Do not repeatedly seek approval for these same component choices. Resolve concrete additional dependencies or materially different tradeoffs when the inventory is available. Implementation, installation, commits and publication remain outside the current planning step.

Decision 81 supersedes the earlier language inventory while retaining decision 51's bounded offline SQL contract. Qualify only the selected targets and report unimplemented syntax or navigation operations explicitly.

### 50. Historical structural-language decision — superseded by decision 81

This former language-specific scope is outside the first-release target list. Its original decision is retained in [the historical record](reviews/2026-10-25/superseded-language-tooling.md). Structural parsing must still be distinguished from resolved semantic navigation for every currently selected target.

### 51. Offline schema-aware SQL navigation

The user selected offline schema-aware navigation for SQLite and PostgreSQL. Resolve table/column references using project SQL and explicitly supplied schemas where the available information permits. This clarifies the SQL portion of decision 31 and preserves its navigation intent.

Do not connect to databases, introspect live systems or execute migrations to obtain schema state. Preserve dialect, source identity and revision-bound locations. Missing or ambiguous schema information, unsupported syntax and dynamic SQL must produce clearly incomplete results; syntax/search results must remain distinguishable from resolved references. Do not imply complete runtime query resolution.

The exact OSS parser/navigation engine remains to be qualified; this decision accepts the behavior, not a particular dependency or a custom SQL analyzer. Prefer an established engine with a narrow adapter and demonstrate its offline behavior before final selection. Verify local table/column definitions and references, aliases and scope collisions, missing schemas, changed files and dialect differences. A fixture with ambiguous names must not receive fabricated definitive resolution.

### 52. Historical analyzer prerequisite decision — revised by decision 81

The original analyzer-specific commitment is outside the first-release language scope. The [historical record](reviews/2026-10-25/superseded-language-tooling.md) preserves its rationale. Explicit platform SDK prerequisites remain allowed only for selected C/C++ targets under the current decision 74 boundary; no automatic SDK installation or global analyzer fallback is introduced.

### 53. Local diagnostics without automatic reporting

The user selected local diagnostics only. Keep bounded local logs with secret redaction and provide a user-requested diagnostic bundle that can be inspected before sharing. Mirket does not automatically upload crash reports, performance data or diagnostic bundles and does not require a reporting service.

Disable bundled tools' telemetry where supported and verify the assembled tools' behavior. A configuration flag is not proof of silence. If a tool cannot meet this policy, resolve that admission issue explicitly rather than silently accepting reporting. This is an application behavior contract, not an OS network sandbox or a restriction on separately authorized browsing and asset preparation. Host clients retain their own independent reporting policies.

Prefer operational metadata such as component versions, error categories, timings and resource use. Exclude credentials, saved browser authentication, raw project/document contents and memory records from ordinary logs and default diagnostic exports. Redact sensitive values before persistence where feasible; minimize collected fields rather than relying on pattern redaction alone. Capture child-process output with bounded handling so upstream logs cannot bypass these rules. A diagnostic export remains local until the user explicitly chooses to share it.

Verify log bounds, rotation, redaction and export contents with planted secrets and representative child-tool failures. Check for unexpected reporting in exercised startup, success and failure paths, while distinguishing authorized task networking. Exact retention/size limits and bundle fields remain implementation specifications to settle. No claim of perfect redaction or complete network isolation follows from this decision.

### 54. Seven-language OCR coverage

The user selected the broader curated OCR set: English, Turkish, German, French, Spanish, Italian and Portuguese. Bundle the required language assets in the complete release; ordinary extraction must not download missing language models. This settles the OCR language scope from decision 40 and is separate from multilingual memory and authored-font coverage.

Verify each language and representative mixed-language scanned documents, including accented characters, Turkish dotted/dotless I, numbers, tables and reading order. Preserve source locations and report uncertainty or unsupported language coverage explicitly. Do not claim that bundling language assets establishes recognition quality, or that every possible language combination has been tested.

The exact asset versions, licenses, recognition settings, quality thresholds and package/resource costs remain to be qualified. Use these same coverage requirements when comparing the accepted document baseline with replacement candidates. Additional OCR languages require an explicit scope decision; do not infer all-language support from a multilingual engine.

### 55. Focused bundled fonts for authored artifacts

The user selected focused bundled font coverage for English, Turkish, German, French, Spanish, Italian and Portuguese. Provide sans-serif, serif and monospace families, a small display selection, and common mathematical symbols. Continue supporting project-supplied fonts under decision 44. This defines authored document/design/video typography independently of OCR and multilingual memory.

Select exact families, styles and a concrete mathematical-symbol inventory during component qualification, with source/version/license records and verified redistribution and embedding rights. Do not infer complete mathematical typesetting, all Unicode symbols or broader script coverage from this decision. Do not bundle a broad world-script collection by default or use personal host fonts as an implicit substitute.

Verify each supported language's characters, including Turkish dotted/dotless I and European diacritics, across actual selected renderers. Check shaping, wrapping, font selection, mathematical-symbol fallback and permitted embedded exports. Keep fonts private, report missing glyphs/substitutions and preserve explicit project font choices. Coverage acceptance is not evidence that any particular family or renderer already passes these checks.

### 56. Broad practical spreadsheet calculation coverage

The user selected broad practical calculation coverage: arithmetic, conditional aggregation, logic, text, dates, lookups, statistics and financial formulas. Include modern lookup and dynamic-array operations where verified. Use the selected LibreOffice engine rather than implementing a separate formula engine. This refines the spreadsheet portion of decision 40.

Produce an explicit compatibility list tied to the packaged engine version and tested workbook behavior. Report unsupported formulas and incomplete calculations clearly; do not imply full Excel compatibility or label pre-existing cached values as freshly calculated. The category list is a coverage target, not proof that every function, argument combination or imported workbook is supported.

Verify actual recalculation against independently established expected results, including cross-sheet dependencies, changed inputs, blanks/errors, date conventions, representative financial/statistical cases and the selected modern lookup/dynamic-array functions. Reopen saved XLSX files to check formulas and results survived export; verify array dimensions and representative dependent cells as well as top-level outputs. Preserve original workbooks when transformations could lose unsupported features.

Exact function inventory, numeric tolerances, supported date/locale behavior and workbook resource limits remain to be specified. Macros, embedded code and external data refresh are not authorized by calculation support. Retain the agreed document execution boundary and report unmet calculation dependencies instead of silently retrieving or executing them.

### 57. Existing PDF text and layout editing

Historical scope, narrowed by decision 72 for imported PDFs without editable source. Read decision 72 as the current contract.

The user selected option 2: require editing existing PDF text and layout in addition to practical document operations. Scope includes replacing existing paragraphs, reflowing text and rearranging existing page content, plus merge/split, page reorder/delete/rotate/crop, metadata/bookmarks, supported form filling, annotations and text/image overlays. Preserve editable source for newly authored documents. Do not silently reduce this requirement to overlays or page manipulation.

This expands decision 40 and reopens the sufficiency of the PDF backend portion of decision 49. The accepted pypdf/PDFium/LibreOffice direction is not evidence that arbitrary existing text/layout editing is solved. Qualify an OSS editing or reconstruction path before final implementation-plan acceptance, including dependency/license review for any replacement. Prefer reuse of a suitable existing engine; do not implicitly authorize a custom PDF editor or several redundant backends. No new production dependency is selected by this scope decision.

Preserve original inputs and verify saved edits by rendering and inspecting text/content as well as appearance. Test longer and shorter replacement paragraphs, wrapped/multicolumn text, font availability, image/vector repositioning and preservation of unaffected content. Distinguish image-only scans and outlined glyphs from editable text; identify reconstruction requirements and unsupported cases rather than declaring success from OCR or a painted overlay. Secure redaction, signature preservation and perfect round trips are not established by these operations.

The required outcome is broad existing-document editing; decision 58 selects controlled overflow/pagination behavior; actual supported cases, fidelity criteria and engine feasibility remain to be qualified. A failure to meet a case must be visible and must not silently remove the requirement. No implementation, installation or publication is authorized by this planning choice.

### 58. Controlled PDF reflow and pagination

Historical scope, superseded in part by decision 72: automatic source-based pagination remains; general automatic reflow of imported PDFs is no longer required.

The user selected controlled reflow when replacement PDF text no longer fits. Expand the affected layout and add pages when necessary, preserve unrelated content, and report pagination changes. Do not silently truncate text or shrink it to unreadable sizes. This specifies overflow behavior for decision 57 rather than reducing the requirement to fixed-size overlays.

Determine the affected text flow and dependent layout before applying the edit. Preserve content, reading order and intentional relationships as that flow moves; avoid unrelated layout changes merely to simplify reconstruction. Report affected pages and resulting page-count changes. If the engine cannot establish or preserve the required flow, expose that limitation rather than emitting a successful-looking clipped, overlapping or incomplete document.

Verify long replacements that cross page boundaries, shorter replacements, adjacent columns/images/tables and unaffected pages. Inspect saved text for omissions/duplication and render outputs for clipping, overlap and legibility. Check page-dependent structures such as bookmarks and links where the operation affects them; report unsupported preservation rather than claiming it. Exact overflow detection, supported layout cases and fidelity criteria remain backend qualification work, not proven capabilities.

### 59. Save edited artifacts to a new output copy by default

The user selected a new output copy as the default for edited documents and designs. Preserve the original unless the user's instructions request an in-place edit. Honor explicit overwrite/in-place instructions within their scope without repeated confirmation; the default is not a mandatory approval flow.

Use an unambiguous output path and return the actual saved location. An existing output destination is not permission to overwrite it: choose a distinct destination when the naming is flexible, or report a conflict when the requested destination is fixed. Check source/destination changes before committing results so another process's edits are not silently lost. Never replace a file with partial output after failure or cancellation; stage output and use a verified finalization mechanism appropriate to the file or multi-file artifact.

Verify original preservation, explicit in-place success, conflicting edits/destinations and failure/cancellation before finalization. Ensure output references and linked assets remain valid when an artifact moves to a new location. Exact naming, conflict detection and multi-file finalization mechanics remain to be specified; no new version-history service or automatic prior-version archive is implied by this decision.

### 60. Balanced local memory-backup retention

The user selected up to seven daily and four weekly recovery points, plus a snapshot before state migrations. Apply a configurable storage cap. These are Mirket memory backups, not backups of project files or edited artifacts; manual export/import remains available under decision 23.

Create consistent snapshots through the existing local runtime without a permanent backup daemon. Capture actual available state; do not fabricate snapshots for missed days or claim uninterrupted daily coverage when Mirket was inactive. Daily/weekly selection and shared snapshot accounting must keep retention bounded without introducing redundant backup services. Decision 61 specifies cap-pressure rotation and recovery-point protection. Decision 62 specifies the default cap and scope; exact timing remains to be specified.

Pre-migration snapshots must be valid recovery material for the affected state and remain available for the required rollback window. They are not an unbounded exception to storage accounting; qualification must define safe behavior when a required snapshot cannot be created or retained. Explicit forgetting continues to remove or invalidate affected managed snapshots under decision 23 and takes precedence over retaining their contents.

Verify consistent snapshots during concurrent writes, retention selection across inactive periods, interrupted backup creation, restore validation and forgetting across daily, weekly and migration snapshots. Acceptance selects a retention policy, not evidence that recovery or migration rollback has been implemented or tested.

### 61. Rotate eligible older backups at the storage cap

The user selected automatic rotation: remove the oldest eligible managed snapshots to make room when backup storage reaches its configured cap. Protect the latest verified recovery point for each backed-up memory scope and snapshots required for an active migration. A newer snapshot becomes a protected recovery point only after successful creation and verification; do not discard the last valid recovery point in anticipation of success.

If protected snapshots leave insufficient room, report the failed/deferred backup rather than exceeding the configured policy silently or deleting protected state. A migration that requires the unavailable backup must wait. Ordinary memory use remains available, with degraded backup coverage visible; do not imply successful backup protection after a failed attempt. Cap pressure may reduce the daily/weekly history retained under decision 60.

Protection applies to automatic storage rotation, not to explicit forgetting: decision 23 still requires removing or invalidating affected managed snapshots so forgotten content cannot be restored. Preserve scope identity and recovery validity during concurrent creation, pruning, forgetting and migration operations. Count in-progress backup storage in the space-admission design; decision 62 specifies the cap value/scope, while temporary-space accounting mechanics remain to be specified.

Verify oldest-eligible selection, per-scope last-good protection, active-migration protection, failed snapshot creation, insufficient space and recovery after space becomes available. Do not build a permanent backup service or a separate general storage scheduler for this policy.

### 62. Five-GiB aggregate memory-backup budget

The user selected a configurable 5 GiB total default budget (5,368,709,120 bytes) across managed project-memory and shared-preference backups. This is one installation-wide budget, not 5 GiB per project. It includes daily, weekly and pre-migration snapshots; it does not cap live memory, models, caches or project files.

Apply decision 61 oldest-eligible rotation and protected-recovery-point behavior within this aggregate budget. Do not silently raise the limit, multiply it as projects are added or treat migration backups as exempt. Account for in-progress backup creation so temporary output cannot become an unbounded loophole; exact reservation/finalization mechanics remain to be specified. Manual exports outside the managed backup store remain user-owned and must not be pruned by automatic retention.

The cap is a configurable product default, not a measured claim that all projects will retain every daily/weekly point. Verify aggregate accounting across multiple projects and shared preferences, the binary-unit boundary, concurrent backup attempts, interrupted temporary files and reduced-cap settings with protected snapshots. Report insufficient capacity and degraded coverage rather than destroying protected recovery material.

### 63. Restore a selected memory scope from its recovery point

The user selected replacement from the chosen recovery point as the default restore behavior, rather than merging recovered records into current memory. Replace only the selected project-memory or shared-preference scope, preserving recoverable current state first. Other projects and unselected scopes remain unchanged. This is a user-requested restore operation, not automatic rollback on a transient error.

Validate the snapshot, source/scope identity and schema compatibility before changing live state. Retain provenance, revisions, supersession and unresolved conflicts from the selected point, while applying known forgetting records so deleted content cannot be revived. Thus restoration reproduces the eligible snapshot state subject to the existing forgetting policy; it does not restore forbidden historical content for the sake of byte-for-byte equivalence. Rebuild derived indexes from restored canonical records.

Protect the current-state recovery material until restoration is verified or rolled back, within the managed backup budget. If required recovery material cannot be created safely, leave live state unchanged and report the failed prerequisite. Coordinate active clients and revision changes so writes cannot be silently lost or mixed with partially restored state. On interruption or failure, preserve a recoverable consistent state and report what actually completed.

Verify selected-scope replacement, isolation of unrelated scopes, preservation/recovery of current state, corrupt or incompatible snapshots, concurrent writes, interruption, derived-index rebuild and forgetting precedence. This decision does not add a general merge engine or select import collision semantics; those remain distinct from restore. Exact transaction/finalization mechanics remain implementation qualification work.

### 64. Recognize verified project moves and renames

The user selected automatic recognition of verified project moves/renames. Retain the existing project-memory identity when evidence establishes continuity between the old and newly selected location. Ask only when that relationship is ambiguous; do not require a registration or relinking ceremony for a verified move.

Matching folder names, remote URLs or source contents alone do not establish continuity and must not merge independent clones. Preserve old memory while an identity question remains unresolved; do not attach it silently to a different project. A copied directory is not automatically a moved directory. Verified linked worktrees continue to share repository-wide facts under decision 38, while checkout-specific task and analysis state remains separate.

Resolve identity at explicitly selected or otherwise authorized workspace locations; automatic recognition does not authorize broad filesystem discovery. Revalidate path-bound source references and derived analysis state after relocation so stale paths are not reported as current locations. Preserve original provenance rather than rewriting historical facts as if they were recorded at the new path. Identity continuity grants no additional filesystem permissions.

Qualify concrete identity evidence and persistence for Git and non-Git projects, same-volume renames, cross-volume moves, copied checkouts, linked worktrees and path reuse by a different project. If available evidence cannot reliably distinguish a move from a copy, ask rather than guessing. Decision 65 specifies the unavailable-project cache grace period. Exact identity metadata/storage and cleanup mechanics remain to be specified.

### 65. Thirty-day grace period for unavailable-project caches

The user selected a 30-day grace period for checkout-specific, rebuildable indexes/caches when a project becomes unavailable. Retain eligible derived state for possible reconnection, then reclaim it during normal runtime maintenance. Do not add a permanent watcher or cleanup daemon.

Preserve durable memory, project identity, backups under their own retention policy and any state required by active work or recovery. A missing folder is never an instruction to forget a project. Reclaim only Mirket-owned rebuildable state after rechecking availability, ownership and active use; do not delete project files, saved browser profiles or another checkout's state.

Track the observed unavailability interval explicitly. A verified reconnection or move under decision 64 ends that interval; revalidate retained caches before reuse. Permission failures or uncertain identity must not be silently equated with confirmed deletion. If Mirket is inactive when the grace period expires, cleanup happens at a later eligible maintenance opportunity, not through an added background service.

Verify temporary disconnection, reconnection before/after the grace period, verified relocation, path reuse, active-job protection and interrupted cleanup. Rebuild on demand when a returning project no longer has caches. Exact timestamp/clock-change handling and maintenance scheduling remain implementation details; the grace period is a product default rather than a measured optimal value.

### 66. Explicit-only update discovery

The user selected explicit checks only. Contact the release service for update discovery only when the user requests an update check or an update. Do not check at startup, on a timer, during idle cleanup or incidentally during ordinary tool calls. This completes the discovery policy alongside decision 29's explicit verified installation policy.

A check reports availability and compatibility without downloading or activating the release payload. An explicit update request may perform the required discovery, download, verification and activation within its authorized scope without repeated permission questions. Preserve active jobs and state according to decision 29. Individual bundled tools must not independently check for or install component updates during normal use.

If a requested check cannot reach or validate the release source, report that availability is unknown rather than claiming the installation is current. Cached metadata must retain its retrieval time and must not be presented as a fresh check. No automatic reporting service or update daemon is introduced; permitted browsing and asset preparation retain their separate network contracts.

Verify that startup and normal workflows produce no update-discovery requests, explicit checks do not fetch release payloads, and explicit updates follow the verified activation path. Exercise offline and invalid-metadata failures. Decision 67 selects one stable release channel with exact-version prerelease selection. Exact release source, manifest/trust mechanism and CLI spelling for check-only behavior remain to be specified. This planning decision does not authorize publishing a release.

### 67. One stable release channel with explicit prerelease selection

The user selected one stable channel. Ordinary explicit update requests resolve to eligible verified stable releases. Testing a prerelease requires explicitly selecting that exact version; it does not subscribe the installation to a preview channel or authorize subsequent prerelease updates.

Use the same release verification, compatibility, active-job and state-migration requirements for stable and explicitly selected prerelease artifacts. A stable label is not proof that a release is compatible, authentic or safe to activate. Do not silently downgrade an installed prerelease or bypass schema compatibility when a normal update request is later made; report when no eligible stable transition exists. Exact version ordering and transition mechanics remain to be specified.

Verify stable candidate filtering, explicit exact-version prerelease selection, absence of persistent preview subscription, invalid/missing versions and incompatible stable transitions. Retain decision 66 explicit-only discovery and decision 29 explicit installation. No new preview service, separate installer path or publication authorization is implied.

### 68. Home-folder installation, scripts and a supplied installation prompt; plan for Windows

The user requires retaining the home-folder `.mirket` installation approach, distributing through installation scripts and providing a special prompt through which installation will be performed. The user also explicitly requires accounting for future Windows work. The unanswered DMG-versus-ZIP question is withdrawn; a DMG-first user workflow is not selected.

Use the current user's home `.mirket` root: `~/.mirket` on macOS and the resolved Windows user-home `.mirket` directory (normally `%USERPROFILE%\.mirket`). Keep versioned package contents, writable durable data, profiles, caches and temporary operations separate beneath that root. Preserve per-user installation without administrator privileges. Existing unrelated or unrecognized content, including an older harness installation, must not be overwritten or deleted implicitly; a clean rewrite does not authorize destructive migration or a legacy compatibility layer.

Installer scripts are distribution entry points for platform-specific, complete prebuilt release payloads. Proposed implementation: a thin macOS shell bootstrap and Windows PowerShell bootstrap identify OS/architecture, retrieve and verify the selected release, then hand off to `mirket setup`. Keep substantive installation/update/recovery and client configuration logic in the shared Rust implementation, rather than duplicating it in scripts or an LLM prompt. Release archive format and bootstrap trust/signing mechanics remain technical qualification items; script delivery does not remove native platform signing or security requirements. No global package-manager dependency or runtime first-use dependency download is introduced.

Provide a maintained installation prompt that instructs the host agent to identify the platform, inspect existing installation state, obtain the official bootstrap for the intended release, follow the verified setup workflow, and inspect `mirket doctor` plus actual client discovery/invocation. The prompt guides deterministic commands; it does not grant permissions, bypass host/OS safeguards or improvise installation code. Honor existing user authorization without redundant confirmations, and surface genuine prerequisites/conflicts. Exact prompt text, download URLs and runnable commands are finalized only after the release/install contract exists; do not provide fictional live installation endpoints.

Design the Rust core, adapter protocol, package manifest and state format for future Windows without introducing a generic platform/plugin framework. Isolate actual platform boundaries: home/path handling, process trees and cancellation, file locks and replacement, executable/native-library resolution, signing and client configuration. Do not assume Unix signals, symlinks or replacement of running executables work identically on Windows. Produce separate OS/architecture payloads with explicit capability inventories and platform tests; don't silently substitute global tools or claim Windows parity from Rust/TypeScript portability. The Apple SDK exception remains Apple-specific.

Decision 69 supersedes the provisional macOS-first delivery order: Windows joins the first-release scope. Decision 70 selects Windows 11 x64 and ARM64. Exact supported Windows builds and host-client/tool qualification remain open. No implementation, installation, publication or modification of the user's existing `.mirket` is authorized by this planning step.

### 69. Native Windows joins the first-release plan

The user explicitly requested adding Windows support to the plan now. Treat native Windows and macOS Apple Silicon as supported first-release targets, not a macOS release with a hypothetical Windows follow-up. This supersedes Windows deferral in decision 3 and the provisional delivery order in decision 68. Decision 70 selects Windows 11 x64 and ARM64; exact supported Windows builds and client floors remain pending.

Retain one Rust core, one TypeScript adapter package, shared tool/memory contracts and the same home-folder `.mirket` layout. Build separate complete, verified platform payloads with macOS shell and Windows PowerShell entry points, using the same maintained installation prompt with platform-specific branches. Native Windows must not require WSL, a VM, Docker, administrator access or global installations of Mirket-managed tools. Do not add separate Windows product logic or a second installer implementation.

Both platforms are release gates: verify installation, actual client discovery/invocation, private executable/state resolution, all applicable selected capability journeys, cancellation of owned process trees, concurrent clients, backup/restore, updates/migrations and uninstall preserving data. Test Windows paths with spaces/non-ASCII characters, drive/UNC handling as supported, file locking, executable-in-use updates and ordinary-user permissions. Introduce Windows CI and real packaged runtime tests early, alongside macOS; cross-compilation or passing Rust unit tests alone cannot establish support. Choose a documented Windows performance reference and measure it separately from the accepted Mac reference.

Qualify the tool inventory on each OS/architecture. The common product contract remains the target; a Windows packaging gap is unresolved work, not permission to silently drop a capability or substitute global tools. Platform-specific prerequisites must be explicit. Apple SDK-dependent C/C++ analysis remains an Apple-specific boundary; qualify each selected target separately instead of claiming Windows can supply Apple SDK functionality. Confirm Codex Desktop's native Windows integration independently.

Current upstream evidence establishes leads, not an assembled working release: Playwright documents Windows browser support, OpenPencil advertises Windows binaries but its desktop path depends on auto-updating WebView2, LibreOffice's standard Windows installer requires administrator access while a portable distribution is listed, and Docling documents Windows support. Reconcile the OpenPencil UI/runtime route with private packaging, OSS policy and explicit-only updates; verify portable LibreOffice/UNO isolation and the full native/model/runtime closure. Do not silently admit a proprietary WebView2 payload, execute an upstream installer or relax the no-administrator requirement. Detailed findings are in `toolchain-proposal.md`.

No Windows binaries were installed or executed for this planning change. Exact dependency approval, platform runtime verification, complete-plan agreement and publication authorization remain outstanding.

### 70. Windows 11 x64 and ARM64 in the first release

The user selected Windows 11 x64 and ARM64, both required alongside macOS Apple Silicon for the first release. Maintain three explicit release targets: macOS ARM64, Windows x64 and Windows ARM64. Architecture selection is settled; exact supported Windows builds, minimum macOS version and host-client versions remain to be qualified.

Build and package the Rust core natively for each target. Provide complete target-specific runtime/tool inventories with architecture-correct native modules, libraries and executables. Detect the actual host architecture during installation even if the invoking shell or host client is running under emulation; do not select an x64 payload for ARM64 accidentally. Preserve the single `.mirket` layout and shared application/tool contracts, with no parallel product or installer implementation.

Both Windows architectures require their own packaged installation, host-client integration, capability journeys, update/migration/restore and cancellation verification on representative Windows systems. Cross-compilation alone is insufficient. Measure ARM64 and x64 resource behavior independently and qualify every native tool/runtime/model dependency, including browser engines, Office/UNO, extraction/OCR and language servers. No unavailable component may silently disappear from the ARM64 package or fall back to the user's global installation.

Decision 71 selects native-first ARM64 support with explicitly inventoried and tested x64 helper-process exceptions when no suitable native build exists. Keep emulated-process evidence distinct from native ARM64 evidence. A qualification gap must be resolved before claiming first-release ARM64 support; no individual exception is qualified merely by this architecture selection.

No Windows implementation or runtime verification has occurred during this planning change. It establishes release scope and acceptance obligations, not demonstrated portability or permission to install or publish.

### 71. Native-first Windows ARM64 with qualified x64 helper exceptions

The user selected native-first Windows ARM64 support with tested exceptions. Keep the Mirket Rust core native ARM64 and prefer suitable native ARM64 tools/runtimes. When an upstream tool lacks a suitable native build, permit an explicitly documented, privately packaged x64 helper process under Windows' built-in emulation only after compatibility and performance qualification. This is a bounded integration choice, not a general fallback to the x64 release or the user's installed tools.

Record each exception's reason, executable/runtime/dependency architecture, exact release identity, license closure, private paths and verified Windows build/hardware. Keep architecture-compatible dependencies within each process; communicate across process boundaries through the existing adapter contract rather than loading incompatible libraries into the native core. Ship the required helper and runtime in the complete ARM64 payload, with no incidental installs or runtime download fallback.

Verify actual output quality, startup, cancellation/process cleanup, concurrent operation, update behavior, private-state isolation and cold/warm resource use on Windows ARM64. Distinguish native and emulated components in the capability inventory and diagnostics. An x64 test result is not ARM64 emulation evidence. If an exception fails the required behavior or resource criteria, resolve the dependency choice before release rather than silently removing the capability or labeling it native.

This policy approves the integration direction, not a particular new dependency or measured performance claim. Keep one selected implementation per capability for each platform; do not ship redundant native/emulated backends as automatic fallbacks. No separate emulator, VM, WSL dependency or architecture-compatibility framework is added. Exact exception selection follows the concrete tool inventory and existing dependency approval rules.

### 72. Source-first editing and bounded direct PDF edits

The user accepted source-first editing plus bounded direct PDF edits following the PDF backend feasibility review. This narrows decisions 57–58; it does not remove PDF text editing or the existing practical PDF operations.

When a corresponding editable source exists, edit that source through the selected Office/Typst owner and regenerate the PDF, preserving the source and reporting pagination changes. Verify source/PDF correspondence, including an embedded source; do not silently select a stale sibling file. Qualified source-based pagination remains required, without promising pixel-identical re-export.

For imported PDFs without source, provide an explicit, tested subset of direct text replacement and object movement, with paragraph-local wrapping where it fits and preserves unaffected content. Keep page operations, metadata/bookmarks, supported forms, annotations and overlays. A direct text replacement must remove old text from search/extraction, not merely cover it. Refuse unsupported edits rather than clipping, shrinking to unreadability, overlapping adjacent content or silently substituting fonts.

When a change requires reconstruction, expose it as a separate explicit operation producing new editable source and a PDF, with layout differences and uncertain structure reported. The host agent may author/review the reconstruction using the existing extraction/document tools. An ordinary edit must not silently trigger reconstruction; general automatic cross-page reflow and new-page insertion for arbitrary imported PDFs are no longer a release requirement.

Decision 59's new-copy default, explicit in-place handling and conflict/failure preservation remain unchanged. Verify saved text and rendered outputs, source-based overflow, bounded imported edits, unsupported-case refusals and explicit reconstruction. Do not treat OCR, an overlay or a successful save as evidence of faithful editing.

Keep existing backend owners provisionally and qualify LibreOffice's bounded editing path first; PyMuPDF remains a candidate only if it closes a demonstrated gap. This acceptance selects behavior, not a new dependency, a custom PDF engine, installation or implementation. Exact supported cases, backend choice, license closure and all three platform checks remain outstanding. See the [feasibility review](toolchain-proposal.md#pdf-backend-feasibility-and-proposed-scope-clarification).

### 73. Separate private Office and general Python runtimes — superseded by decision 79

The user accepted two private Python runtime owners: LibreOffice's matched Python/UNO for Office automation, plus one general Python environment for extraction, PDF libraries and SQL libraries if admitted. This resolves the sharing question in decision 49. Do not install general third-party packages into Office's interpreter or load its native UNO bridge into an unrelated interpreter.

Keep Office and its matching Python native for each release target. The general worker is proposed native on macOS ARM64 and Windows x64, with an x64 worker on Windows ARM64 subject to decision 71's explicit compatibility/resource qualification. This decision accepts the ownership split, not an untested emulation result or specific interpreter distribution/version. Keep architecture-compatible dependencies within each process.

Use the existing job lifecycle and complete private `.mirket` payload. Launch only needed processes, and import only the libraries required by the operation; ordinary PDF/SQL tasks must not initialize the ML pipeline. No third general interpreter, always-running Office service, duplicate fallback interpreter or new scheduler is selected. The additional distribution space, patch maintenance and runtime inventory are accepted tradeoffs; actual size and performance remain to be measured.

Exact interpreter/library pins, redistribution closure, private Windows Office relocation, policy/font handling and all three target workflows remain qualification work. SQLGlot is still a proposed dependency. Existing limits on global installations, host configuration changes and implementation before complete-plan agreement remain unchanged. This former packaging contract is superseded by the [current Office ownership contract](toolchain-proposal.md#host-owned-office-and-general-python).

### 74. Explicit platform SDK prerequisites for selected C/C++ targets

Within decision 81's language scope, C/C++ analysis may read explicitly selected user-installed Apple SDK headers or Windows SDK/MSVC components where required by the project target. Mirket retains privately packaged analyzers and redistributable runtimes. Record selected paths, versions and architectures; invalidate affected analysis after changes. Missing or incompatible inputs report incomplete analysis without blocking unrelated capabilities.

Do not install or update SDKs, modify global PATH/developer-tool settings, enable Developer Mode, redistribute proprietary SDKs or execute project preparation implicitly. The [earlier wider prerequisite research](reviews/2026-10-25/superseded-language-tooling.md) remains historical. Exact C/C++ runtime/toolset combinations and passive-loading behavior require actual qualification.

### 75. Maintenance-window activation for every update

The user accepted one active release with a brief maintenance window for every update. An explicit update may download and verify the complete candidate while existing work continues. Activation requires all Mirket sessions, owned tool sessions and shared-state operations to stop; do not run old and new releases concurrently against mutable state.

If the installation is busy, report the verified candidate as staged and leave existing work available. Do not cancel jobs, force-close retained browser sessions, block clients indefinitely or activate later in the background. After sessions stop, another explicit update request resumes activation. Idle connected Mirket servers also count as active sessions; clients reconnect after activation through their supported mechanisms. Host conversations and unrelated applications remain open.

During activation, hold exclusive installation ownership, prevent new sessions from opening shared state, and preserve required recovery material before migration. Honor decisions 29 and 60–63: preserve active-release resources, enforce backup capacity/protection, and never silently restore older memory after the new release has accepted writes. Failure or interruption must leave a consistent recoverable release/state pair; retaining an old executable alone does not establish rollback compatibility.

This accepts the maintenance-window policy and its user-visible interruption, not exact locking, stable-launcher, migration, signing or storage-retention implementations. No daemon, parallel-version compatibility layer, installation or publication is authorized. See the [operating contract](proposal.md#installation-activation-and-recovery-contract). Next settle release signing and bootstrap trust.

### 76. Keep the release strategy simple

The user directed that the release strategy must not become overcomplicated, rather than selecting either option in the signing question. Withdraw that pending binary choice. Treat this as a constraint to simplify the design, not approval to purchase certificates, create publisher accounts, publish, or bypass OS protections.

Remove the proposed custom Mirket release-signing layer, signed channel index, sequence/expiry protocol and key-transition workflow from the active plan. Keep the already agreed complete per-target packages, home-folder scripts/prompt installation, explicit updates and maintenance-window activation. The [simplified working design](proposal.md#simple-release-and-installation-workflow) uses one versioned GitHub Release, checksums, the Rust setup/update owner and standard native platform verification.

Native signing/provider details belong to concrete release preparation and platform qualification. They are not a reason to add another architecture questionnaire now. This direction does not approve a specific service or fee, nor establish that checksums independently authenticate a publisher. Next consolidate practical resource defaults and the remaining implementation plan.

### 77. Apply independent-review fixes and repeat the review

The user authorized applying the four Astra High reviewers’ recommended fixes and making another review pass. Preserve decisions 1–76 and all accepted capabilities; amend the planning documents, then review the amended baseline independently with the same roles/model/effort. This is planning authorization, not complete-plan approval or authorization to start implementation, install dependencies, commit or publish.

Apply explicit host/guide delivery and scope contracts; finite operations and saved-artifact inspection; bounded long-job calls with effect reconciliation; connection-owned children and installation-wide admission; crash-safe finalization/forgetting; expandable evidence/constraint applicability; idle reclamation/model unloading; exact source-rights mappings and comparative guide tests. Split M0 into a small real-client/activation path and bounded risk slices. No broker, additional memory service, universal executor, agent journal or custom release-signing layer is introduced.

The earlier analyzer-specific review recommendations are historical and superseded by decision 81. Retain the corrected docling.rs renderer evidence and compare complete extraction outcomes/closure. No candidate is automatically adopted or pinned by this authorization. Add the six focused method references in references.md H; conditional/comparison-only suggestions are not installed or wholesale catalog imports.

The shared [host/tool/evidence contracts](proposal.md#host-tool-and-evidence-contracts) own detailed invariants; the implementation plan owns sequencing and proposed defaults. A backend limitation must not silently narrow required capability. A material contract conflict returns for a concrete product decision; routine schema/locking choices within the contract do not require another questionnaire.

## Superseded branches

Mirket-owned model orchestration and attached model-worker execution were superseded by host-owned agents. Selective VM placement and Podman/ARM64 container images were subsequently superseded by privately packaged host tools. The unanswered VM startup/shutdown question is withdrawn. Do not retain these branches as compatibility layers or fallback implementations. The unanswered DMG-first distribution suggestion is replaced by decision 68 script/prompt installation. Decision 69 supersedes Windows deferral and the provisional macOS-first delivery order. Decision 72 supersedes the general automatic imported-PDF reflow requirement in decisions 57–58, retaining source-based pagination, bounded direct edits and explicit reconstruction. Reference selections remain unchanged.

## Acceptance obligations

- Verify decision 75 stages updates without cancelling active work, returns staged/busy for connected servers or retained sessions, and never activates later without an explicit update request. Exercise admission races, client auto-respawn, interrupted migration/activation and reconnect on all three targets; prohibit concurrent old/new mutable-state access and rollback that silently discards post-activation writes.

- Verify decision 74 uses the explicitly selected compatible SDK/toolset and private analyzer, preserves global settings and unrelated capabilities, rejects wrong architecture, and invalidates affected analysis after SDK changes. Exercise missing/conflicting SDKs and stale metadata; trace project loading for unauthorized preparation or executable overrides.

- Verify decision 79's boundary: no packaged Office engine, UNO, Office interpreter or Office supervisor. General Python resolves only Mirket's intended libraries with compatible architecture and private paths; lightweight PDF/SQL tasks must not initialize ML models. Check actual host Office capabilities and saved outputs separately, without assuming a shared backend across Codex sessions.

- Verify decision 71 explicitly inventoried x64 helpers on Windows ARM64 for output quality, resource behavior, cancellation, updates and private-state isolation; distinguish emulation evidence from native results and prohibit implicit global/x64-release fallbacks.
- Verify decision 70 target detection and separate Windows x64/ARM64 packages on representative systems, including callers running under emulation, architecture-correct dependencies and per-target output/performance evidence.
- Gate both first-release platforms under decision 69 with packaged capability/client journeys, Windows process/file-lock/update failure cases and separately measured performance; unsupported native dependencies remain open admission work.
- Verify decision 68 prompt-to-script-to-CLI installation under a disposable user home, including existing-state conflicts, interrupted downloads/setup, release verification, private paths and real host invocation; qualify each shipped OS/architecture independently.
- Verify discovery and invocation in Codex Desktop, including relevant host-owned delegation workflows without recreating an agent runtime.
- Run the actual packaged browser/design tools on macOS ARM64 and each selected native Windows target with no WSL/VM/container manager, global JS runtime, or globally installed copies of those tools required.
- Verify selected C/C++ analysis uses only explicitly selected compatible SDK/header inputs and its private analyzer, reports missing prerequisites, and preserves global developer-tool settings.
- Verify executable and native-library resolution uses the package plus declared platform system dependencies and the explicit platform SDK exceptions in decision 74 where applicable; conflicting tools on PATH must not change selection.
- Exercise coexistence with normal browser/design apps: separate profiles, settings, caches, IPC/session endpoints and process cleanup. Updating/uninstalling Mirket must preserve unrelated applications and user work.
- Verify headless outputs for actual required operations; verify a visible workflow when a selected capability needs it. Do not infer headless support from the existence of a CLI.
- Verify cancellation, process descendants, concurrent client access, output integrity and interrupted updates. Stop only processes owned by Mirket.
- Test the Rust/TypeScript contract, clean installation, signing/distribution path and recovery; inspect third-party redistribution rights and notices before bundling.
- Verify the complete assembled release includes every selected Mirket-owned tool/runtime; a fresh installation must execute local fixture tasks without downloading missing tool dependencies or resolving global tool copies. Web operations may still require network access by their nature.
- Exercise both local preview and public-web workflows through the packaged browser; verify navigation, extraction, screenshot output and representative permitted interactions.
- Verify fresh sessions do not inherit saved authentication; explicitly selected saved profiles retain intended state across restarts without touching personal browser state. Exercise expiration, deletion and concurrent ownership.
- Verify explicit cross-project profile reuse without implicit login selection or sharing live tabs; concurrent attempts to use an owned saved profile must not corrupt it or attach to the wrong session.
- Verify a multi-call browser journey retains live page state; explicit close, idle cleanup and owner disconnection release owned processes/profile locks without interrupting another client. Check protection of active operations and intentional user interaction.
- Verify an editable-design journey: create a document, reopen it, inspect and edit a layout/component, save it, and inspect exported previews/assets. Check editability and content fidelity as well as successful file creation.
- Verify interface animation behavior and rendered-video playback for representative compositions, including frame timing, dimensions and expected content; verify cancellation and cleanup. Verify supplied audio timing/volume, audiovisual synchronization and clipping, plus supplied-text caption content/timing/readability and retained editable source.
- Verify decision 72 source/PDF correspondence, source-based editing and repagination, qualified direct imported-PDF edits and explicit reconstruction as distinct outcomes. Inspect saved text and rendered output for missing/duplicated text, clipping, overlap and unrelated changes; verify unsupported direct edits are refused without silent reconstruction. Old text must be removed for direct replacement; overlays alone do not meet that requirement. Preserve editable source and report pagination/layout changes.
- Verify decision 59 new-copy defaults preserve originals, explicit in-place requests succeed without redundant prompts, conflicts do not lose concurrent edits and failed/cancelled jobs cannot commit partial artifacts.
- Verify representative DOCX/PPTX/XLSX host-tool journeys in Codex Desktop using their actual available capabilities: create, reopen, edit, render and export; preserve editability and verify supported decision 56 spreadsheet formulas against independent expected results, including saved/reopened formulas and values. Report missing operations and unsupported imported features explicitly; this is not a uniform bundled Office guarantee. Verify Mirket-owned PDF operations separately.
- Verify Mirket tool routing does not expose general control of unrelated desktop apps or fall back to their active sessions; supported visible tool workflows remain within their declared ownership.
- Verify structured memory writes/reads and keyword/tag/filter retrieval with representative records, including stale/conflicting entries; verify semantic retrieval alongside exact search under the selected hybrid design; report its runtime and packaging costs rather than excluding it by version number.
- Exercise host-agent curation of stable facts/decisions with source attribution and inspect/correct/delete operations; verify memory does not override current instructions or introduce passive conversation capture.
- Verify project-scoped retrieval excludes other projects, shared preferences remain explicitly scoped, and project/client requirements are not automatically promoted to global memory.
- Verify durable records survive age-based cleanup, superseded records appear only in appropriate historical queries, and explicit forgetting removes targeted revisions and derived entries without resurrection during index rebuild.
- Verify explicit corrections supersede without redundant confirmation, project exceptions preserve shared preferences, ambiguous evidence stays unresolved, and concurrent stale writes cannot silently overwrite current records.
- Verify consistent backups during concurrent writes, decision 60 daily/weekly/pre-migration retention, decision 61 cap-pressure rotation/protected recovery points and decision 62 aggregate 5 GiB accounting, decision 63 selected-scope restoration/current-state recovery, corrupt/interrupted restore recovery, scope-preserving export/import, and forgetting across managed snapshots and known older imports.
- Verify English–Turkish recall in both directions, mixed-language memories and exact code/path lookup; preserve original records and provenance and measure quality/resource tradeoffs for the selected local model.
- Verify focused recall on substantial task starts/resumptions in Codex Desktop, trivial-request bypass, appropriate reuse of unchanged context, bounded source-linked results and visible incomplete coverage.
- Verify guide discovery and selective loading with representative tasks: appropriate activation, distinct ownership, source attribution and useful outputs without loading all focused guides or duplicating shared rules.
- Verify per-user installation and update without administrator access, scoped client configuration and removal of package binaries without deleting durable user data or unrelated settings.
- Verify decision 67 stable filtering, explicit prerelease version selection and compatible transitions without a persistent preview subscription or silent downgrade.
- Verify decision 66 explicit-only discovery, check-only behavior, absence of incidental upstream-tool checks and honest offline/stale-metadata reporting.
- Verify explicit update activation rejects incomplete/unverified releases, active jobs retain their full release resources, interruption preserves user state, and migrations cannot expose active clients to incompatible shared state.
- Verify symbol search, definitions and references on representative projects in Codex Desktop, including identical names in different scopes, edits/branch changes, stale locations, missing analysis dependencies and incomplete results. Measure indexing latency and resource use; do not equate text matches with resolved references.
- Verify decision 81's exclusive named language/file/framework targets plus decision 82's five bounded offline SQL profiles, including embedded web regions, original Unicode/CRLF positions, explicit syntax-only capability reports and separately demonstrated symbol/definition/reference behavior. No database access, migrations, implicit builds or unsupported semantic claims.
- Verify analyzers do not execute project hooks/scripts or modify project sources by default, using a discriminating fixture that exposes unintended execution. Verify missing metadata is reported and navigation refreshes after an authorized host action supplies it.
- Verify decision 65 grace-period eligibility, reconnection/relocation, active-state protection and on-demand rebuild without deleting durable records or project files.
- Verify cache reuse and invalidation across file/dependency/configuration changes, branch switches, renames/deletions and separate worktrees; exercise corrupt-cache rebuild, incomplete refresh reporting and idle analyzer cleanup without losing valid persistent caches.
- Verify packaged Chromium on macOS ARM64 for required navigation/interaction/capture flows and supported headless/visible modes, saved-profile isolation, cancellation and accurate result reporting. Measure package size and concurrent resource use.
- Verify the public browser route and release inventory expose only Chromium, with no alternate-engine launch or fallback.
- Verify packaged Node.js LTS and compiled adapters work with conflicting/missing global runtimes and no dependency downloads; check native module resolution and cancellation against the pinned release.
- Verify host/configuration workspace selection in Codex Desktop, missing/conflicting/multi-root context, scope changes, normalized roots and no cross-project memory/navigation leakage. Selection must not expand permissions.
- Verify decision 64 project continuity across verified moves/renames, ambiguous copy/move cases and reused paths; preserve old memory and never merge clones from matching names/remotes alone.
- Verify shared durable knowledge across genuine linked worktrees without leaking checkout-specific progress, branch facts or indexes. Verify same-remote independent clones stay separate, branch switches preserve applicability, and worktree removal preserves shared knowledge.
- Verify multi-project operations retain explicit per-call scope during concurrency and switching, ambiguity does not trigger broad searches, cross-project evidence keeps attribution, and deselection preserves durable memory without implicit further use.
- Verify host-owned Office create/edit/preview/export and Mirket-owned PDF/extraction/OCR workflows with explicit ownership for representative document fixtures, including scanned tables, mixed text/images, all seven decision 54 OCR languages and representative mixed-language documents, unsupported imported features and stale spreadsheet cached values. Check source attribution, editability and real calculations as well as rendered output.
- Verify MP4/WebM decoding and playback, alpha where supported, GIF timing, PNG frame order/completeness and burned/sidecar caption text/timing. Report silent formats and unsupported codec/player combinations accurately.
- Verify decision 43 outputs through representative source/export/handoff fixtures, including editable native/PPTX content, raster/vector/icon correctness, token/theme consistency and responsive accessible code in its target project. Do not equate generated JSX or file creation with working UI.
- Verify decision 55 language/font-role coverage and controlled packaged/project font resolution across selected renderers, with conflicting/missing host fonts, missing glyphs and blocked unrequested font downloads. Inspect shaping/wrapping and permitted export/embedding with real artifacts.
- Verify prepared design/document/video rendering with external networking unavailable, including nested/missing assets, unrequested font-provider access and remote assets changing after preparation. Preserve original sources and report incomplete preparation instead of silently fetching or substituting during final rendering.
- Verify concurrent heavy jobs under throughput-first defaults, including tool-internal parallelism, bounded admission, visible queues/cancellation, per-job output integrity and resource exhaustion without silently changing browser targets.
- Evaluate the full feature set on a documented 16 GB Apple Silicon reference with host client/normal applications present, measuring cold/warm latency, process-tree memory, pressure/swap, CPU, idle behavior, package/cache size and combined-workload responsiveness. No quantified speed/memory improvement is claimed from topology alone.

- Verify decision 53 diagnostic bounds, secret redaction before persistence, inspectable local exports and absence of automatic reporting on exercised tool paths; preserve legitimate task networking.

## Decision 78 — Complete-plan approval and implementation

On 2026-10-24 the user approved the complete plan and instructed implementation to begin with sufficiently reasoned subagents, continuing unless user input is necessary. The approved scope includes the proposed defaults, MIT licensing for authored code, M0 qualification and the full capability implementation sequence. Exact new production dependencies still require concrete admission under the project rule. Keep the old harness, installation and client settings intact. No commit, push or publication is authorized.

## Decision 79 — Host-owned Office workflows

On 2026-10-25 the user approved removing Mirket's duplicate Office stack and using the host's available document tools. DOCX/XLSX/PPTX creation, editing, spreadsheet recalculation, rendering and export belong to host tools and their format-specific skills. Mirket supplies concise workflow guidance, source preservation and evidence requirements. Check task-specific capability and host permissions before use; inspect saved/reopened content, rendered output and independently expected formula results. Report concrete unsupported operations or imported features instead of assuming that a capable model guarantees a document engine.

Remove bundled LibreOffice, UNO, its matched Python, private Office profile management and the unfinished Office process supervisor. Do not replace them with another Office engine, custom OOXML/formula machinery, optional pack manager or compatibility layer. Host tooling may internally use LibreOffice or another engine; Mirket does not install or manage it. There is no guarantee of identical self-contained Office functionality across Codex sessions. Host document processes and their admission/lifecycle remain host-owned.

Retain Mirket's implemented basic-PDF tools, SQL navigation and their general Python runtime, plus the separate extraction/OCR, design and PDF roadmap. One general Python remains the intended Mirket runtime; this decision does not add or replace its dependencies. Source-first PDF regeneration from Office sources uses available host export tools. Bounded direct imported-PDF editing remains separately qualified; LibreOffice is no longer its selected backend, and no replacement is admitted by this decision.

This supersedes decision 73 and the Office packaging/backend portions of decisions 40, 49, 56, 69, 70 and 72, and any earlier complete-package or uniform Office support obligations. Desired document outcomes and preservation/verification rules remain; their availability is checked against actual host capabilities. Retain past qualification evidence outside shipped source and historical review snapshots unchanged. Preserve existing PDF/SQL implementation, the old harness and user installation. No commit, push or publication is authorized.

## Decision 80 — Codex Desktop is the sole first-release host

On 2026-10-25 the user selected Codex as the only first-release target and directed removal of the other host integration. Codex Desktop owns model selection, agents, coordination, permissions and its execution lifecycle. Keep only its native registration, skill delivery, configuration and qualification routes. No alternate-host wrapper, configuration, test or release gate belongs to 0.0.1. General source research and attribution may remain without implying host support.

This supersedes earlier dual-host target and acceptance wording. It does not reduce capability, platform, artifact-preservation or evidence obligations. Concurrent Codex connections still require shared admission, exclusive live ownership, saved-profile protection and release activation checks. macOS ARM64, Windows x64 and Windows ARM64 remain the platform targets, subject to their actual qualification. No commit, push or publication is authorized.

## Decision 81 — Exclusive first-release language tooling scope

On 2026-10-25 the user selected only Python, Rust, TypeScript, JavaScript, C, C++, Bash, JSON, HTML, JSX, TSX, CSS, TOML, XML, PowerShell, Vue, Angular, Astro, SVG and YAML for the first-release language tooling, and additionally retained SQL navigation for SQLite and PostgreSQL. This supersedes the wider language/analyzer inventory and removes excluded analyzer/runtime/package/qualification obligations. Retain generic data-analysis references without treating them as language-tool support.

Treat syntax and structure, original source positions, symbols, definitions and references as distinct capabilities. A grammar/tree parse or initialized language server is not complete navigation. Qualify each applicable operation and platform with representative pass/refusal cases, including embedded languages, Unicode/CRLF spans, scope collisions and source changes. Unimplemented or incomplete support stays explicit. No project build, plugin, generator, macro, script or dependency download is authorized implicitly.

Preserve dated research and prior qualification evidence as clearly historical where it covers excluded languages. The retained offline SQL contract and its existing evidence remain current. This changes language-tool scope, not the independent natural-language OCR, typography or multilingual memory requirements.

## Decision 82 — Five bounded offline SQL profiles

The user requested additional SQL dialects beyond the original SQLite/PostgreSQL choice. On 2026-10-25, the existing pinned SQLGlot 30.24.0 route qualified explicit IDs `postgres`, `sqlite`, `mysql`, `tsql` and `duckdb` for bounded declared-schema navigation. This supersedes the two-dialect limit in decisions 51 and 81; their original choices remain recorded above. It adds no dependency, live database access, universal SQL contract or implicit alias.

The additional MySQL, T-SQL and DuckDB fixtures verify original quoted/escaped Unicode and CRLF byte positions, namespace-bound declarations, aliases, joins and CTE/derived outputs, plus refusal cases. MySQL case mismatches and executable comments refuse; T-SQL server collation modes are not inferred; unproven dynamic/synthesized/external relations refuse. MariaDB and other dialects remain unqualified and are not exposed. Full dialect syntax, native Windows, oldest macOS, installed final payloads and Codex GUI/model behavior require their own evidence. The [SQL qualification](../docs/retired-evidence.md#retired-docs-m0-sql-qualification-md) owns exact cases and limits.

## Decision 83 — Chromium-only browser operations

On 2026-10-25 the user removed Firefox and WebKit from the first-release harness. Chromium is the sole privately packaged browser and the sole engine exposed by browsing, interaction, capture and browser-based verification. Remove alternate-engine selectors, runtime branches, profiles, payloads, notices and release requirements rather than preserving compatibility. Headless remains the preferred mode, with visible Chromium operation only where verified. This supersedes decisions 34 and 35 and their multi-engine acceptance wording. It does not imply that Chromium has passed redistribution rights, native Windows or actual desktop activation gates. No publication is authorized.

## Decision 84 — Simplification review, first revision

On 2026-10-25 the user directed three changes while the rewrite is paused for plan review. **Keep Mirket-owned design machinery** because Codex needs more support for design workflows. Retain the implemented design-token, SVG rendering/optimization and D2 diagram direction, with editable OpenPencil and raster preparation still candidate capabilities; agree the exact output/tool boundary before resuming implementation. Codex owns creative judgment, project code and live browser inspection, while Mirket owns only selected deterministic design operations and their artifact integrity.

**Move general document work to Codex and available host tools.** This extends decision 79 beyond Office to PDF reading, extraction, authoring and editing. Mirket may add concise format-specific guidance or a narrowly justified helper, but no private general document engine is presumed. Seven-language offline OCR and its Docling/Tesseract/model package are no longer release requirements. For scanned or complex documents, host extraction must be verified against page evidence and uncertainty reported; Codex's ability to read a document is not a guarantee of complete OCR, table coordinates or faithful imported-PDF edits. This supersedes the Mirket-owned PDF/extraction/OCR portions of decisions 40, 54, 57–58, 72 and 79 while preserving the document-quality intent where host capabilities can meet it.

**Abandon Chromium packaging.** Use the Codex Desktop browser for web interaction and design inspection, and project-specific browser test dependencies only when a project needs them. Remove Mirket's browser engine, browser tools, profiles and capture lifecycle from the proposed release path. This supersedes decision 83's privately packaged Chromium requirement; its historical engine-selection evidence remains recorded, not an implementation obligation.

The [working review](codex-native-simplification-review.md) incorporates these choices and lists remaining tradeoffs. Navigation, the video rendering route, memory-model cost and exact design deliverables remain to be agreed. No production source is removed while this review is underway. No installation change, commit, push or publication is authorized by this decision.

## Decision 85 — Keep Mirket-owned video export without packaged Chromium

On 2026-10-25 the user explicitly kept Mirket-owned authored-video export and directed qualification of a separate renderer after abandoning Chromium packaging. Retain the selected standalone-video outcomes: editable composition/source assets, supplied audio and timed text captions, and MP4, WebM, GIF and PNG-sequence exports where the exact selected codec/player path supports them. Preserve the distinction between live project interface motion and rendered standalone video.

The [HyperFrames/Puppeteer qualification](../docs/m3-motion-video-qualification.md) remains historical evidence, but is no longer the active renderer direction because it requires browser provisioning. Do not restore packaged Chromium by inference or claim that Codex Desktop's browser is a private offline rendering API. Qualify a browser-free frame renderer and controlled encoding path. The editable source format, supported visual effects, exact renderer, codecs, build closure, package cost and macOS/Windows results remain open; no candidate is selected or admitted by this decision. In particular, a vector/Lottie source would change the prior HTML/DOM composition model and needs a separate product choice.

This supersedes the HyperFrames runtime selection in decisions 48–49 and the active-candidate conclusion in the motion/video qualification, while retaining decisions 13, 41–42's outcome intent. It authorizes plan revision and bounded research, not a production dependency, installation change, commit, push or publication. Implementation remains paused during scope agreement.

## Decision 86 — Editable vector/timeline source for standalone video

On 2026-10-25 the user chose an editable vector/timeline source with a bounded Lottie feature set for Mirket-owned, browser-free video export. Project-owned Lottie composition and local visual assets are the qualification target; supplied audio and timed captions remain editable separate sources. CSS/WAAPI/DOM animation stays with the live project and Codex browser inspection, without an automatic standalone-video parity guarantee. Do not introduce a custom animation language or Mirket editor merely to reproduce the old HTML/DOM source model.

Qualify one representative source-to-export workflow before selecting a production renderer. ThorVG is the first native Lottie frame-rendering candidate; Skottie is a comparator if ThorVG fails the same visual fixture. A controlled FFmpeg build remains a candidate for MP4/WebM/GIF/PNG encoding and supplied audio/captions. The exact versions, build/license closure, supported Lottie features, codecs, output fidelity, resource cost and native macOS/Windows behavior remain unproven and unadmitted. This choice supersedes the HTML/DOM-editability expectation in the old video direction while preserving decisions 13, 41–42's video, audio, captions and format intent. Implementation remains paused for the scope review; no dependency, installation, commit, push or publication is authorized by this decision.

## Decision 87 — Retain bounded JS/TS-family and SQL navigation only

On 2026-10-25 the user chose the narrow navigation option for 0.0.1: retain Mirket's existing selected-source JavaScript, TypeScript, JSX and TSX inspection/definition/reference path, and the five bounded offline SQL dialect profiles `postgres`, `sqlite`, `mysql`, `tsql` and `duckdb`. The TypeScript path uses its pinned private 5.9.3 language service and Node runtime; the SQL route retains its selected private CPython and SQLGlot. No second TypeScript language server, live database connector, project loader or global host-tool fallback is implied. Each path must report its explicit-source coverage and refusals, not claim whole-project semantics.

Remove the general Python/Jedi, C/C++/clangd+SDK, Rust, Bash, PowerShell, Vue, Angular, Astro and other proposed Mirket language analyzers from release scope; Codex and project toolchains own navigation for those languages. Generic file editing/search remains host-owned. The newly implemented JS/TS graph still needs installed public MCP, Codex-client and Windows checks; SQL's five profiles need native Windows checks and a clean release component separate from document/OCR packaging. The previous language-list selection in decision 81 remains historical intent; this decision supersedes its Mirket-owned release qualification obligations outside JS/TS-family and SQL. The TypeScript graph and Apple SDK/C/C++ work at the pause point remain preserved as source/evidence until final scope approval and reviewed cleanup. No source removal, dependency change, installation, commit, push or publication is authorized by this decision.

## Decision 88 — Keep scoped lazy hybrid memory

On 2026-10-25 the user kept Mirket's scoped SQLite/FTS facts plus the already selected local multilingual MiniLM/ONNX semantic retrieval, loaded on demand for semantic indexing and recall. Retain explicit source identity, record scope, correction, supersession, forgetting and recovery rather than replacing canonical facts with ambient Codex memory. The frozen bilingual retrieval evidence is the reason to keep the model despite its measured package and peak-memory cost.

Do not introduce another vector database, model router, memory service or speculative replacement model. Lazy initialization does not prove low idle memory after the first call; measure two-client post-use idle and combined navigation/design/video load before claiming a performance target. If those acceptance workloads fail, compare a concrete repair against the same frozen retrieval cases rather than silently reducing recall quality. This is a scope choice, not new dependency admission or permission to change installation, commit, push or publish while planning remains paused.

## Decision 89 — Focused Mirket design package

On 2026-10-25 the user selected the focused design option for 0.0.1. Mirket owns repeatable token compilation, vector cleanup/rendering, D2 diagrams and editable OpenPencil design-file workflows, with controlled fonts/assets and only the raster operations needed by those outputs. Codex owns creative judgment, project UI code, live responsive browser inspection and paged document work. Authored standalone video remains its separate vector/timeline export boundary under decisions 85–86.

The implemented Mac token, SVGO, resvg and D2 routes remain the core; OpenPencil file create/reopen/edit/export and any selected Sharp operations require concrete source/output, rights and native-platform qualification before release claims. Do not add the full earlier Typst/Kroki/Satori/remote/editor stack merely to mirror the reference catalog. Preserve meaningful design quality checks: actual editable source change, rendered pixels, typography/assets and responsive application behavior. This supersedes broad design-tool inventory obligations in decisions 49 and 78 while keeping design outcomes. It does not authorize a new dependency, installation change, source removal, commit, push or publication during the paused review.

## Decision 90 — Deliver all thirteen guides as Codex-native skills

On 2026-10-25 the user chose all thirteen specialist guides for the first release, delivered through Codex's native skill mechanism. Preserve the thirteen subjects accepted in the earlier plan: Engineering; Verification; Research/context; Interface/visual design; Motion; Product/marketing; Documents/artifacts; Memory; Security/privacy; Reliability/delivery; Data/analysis; AI evaluation; and Planning/coordination. Each must have a relevant activation trigger, concise method, correct tool/permission ownership and useful output tests, including held-out nonactivation cases. The selected reference catalog remains authoring/provenance evidence, not a requirement to install every referenced runtime or copy its text.

Do not preserve the custom Mirket MCP guide-discovery/read API, release-bound guide IDs or a parallel selection protocol. Codex owns loading and lifecycle; Mirket's setup may install only its managed native skill files with drift/rollback protection and must preserve unrelated user skills. This replaces the guide-delivery machinery in decisions 77–78 without reducing the thirteen authored skill outcomes. It grants no account, provider, model, dependency, commit, push, installation or publication authority during the paused plan review.

## Decision 91 — Codex owns agents; Mirket sets the coordination strategy

On 2026-10-25 the user reaffirmed **Codex Desktop as the only client** and macOS/Windows 11 as the only release OS families. The existing platform targets remain macOS ARM64, Windows 11 x64 and Windows 11 ARM64 unless separately changed; Linux, VM, WSL and a Claude client are outside this release. Codex retains model selection, native subagent spawning/lifecycle, shell/Git and permission authority. Mirket must strongly guide how Codex coordinates work instead of building a second spawner or model router.

The global guidance and project Planning/coordination skill should require useful independent deliverables, disjoint ownership, task-appropriate expertise/reasoning and lead-owned integration. In particular, a creative or design task must not be assigned to a low-capability or low-reasoning subagent merely to increase speed; use a capable lead and comparable critique when independent review helps. Mechanical work may use narrower workers, with actual output verification. Preserve explicit user model/effort/provider choices; never silently downgrade or switch. A skill or MCP server cannot intercept every native Codex spawn or expand host authority, so test observed behavior and report that limit rather than claiming a hard technical prohibition. This supersedes any interpretation of Mirket as an agent spawner.

## Decision 92 — Lean global skill and opt-in project catalog

On 2026-10-25 the user chose a lean global skill/guidance surface plus an extensive Mirket-native Codex skill catalog that can be automatically installed into opted-in project source. One global `mirket-core` skill and concise managed global `AGENTS.md` guidance are the proposed always-available surface. All thirteen previously selected specialist subjects remain native skills, now in the project catalog with focused task skills. Project opt-in is recorded in `.mirket/project.json`; after opt-in, activation/setup/update may synchronize only selected skills into the repository's `.agents/skills` with exact hashes, drift refusal and rollback. Passive inspection cannot mutate a project. [Skill-surface design](skill-catalog-design.md) specifies the initial catalog and authoring/rights/evaluation contract.

The user wants Codex to use Mirket primarily for Mirket-owned agentic workflows. For qualified Mirket operations, the global guidance must direct Codex to use Mirket first and diagnose/report unavailability rather than silently substitute host behavior. This is instruction-based behavioral enforcement, not a permission bypass or a guarantee that Codex cannot choose another tool. Propose a required MCP registration only after testing its actual Codex Desktop startup/failure/recovery behavior. This narrows decision 90's delivery mechanism: all thirteen remain authored, but are not all global.

## Decision 93 — Restore private Chromium and full design/video route

On 2026-10-25 the user changed the browser choice again: Mirket should privately package a browser for **all browser-related work**, not only video. Restore Chromium-only general browser operations, Playwright integration, private profiles, local/public navigation, interaction and screenshots; Codex's browser is no longer the intended default for Mirket workflows. Firefox and WebKit remain excluded. This supersedes decision 84's no-packaged-Chromium instruction and the narrower browser-free video assumptions in decisions 85–86.

Restore strong design tooling and the expanded design outcomes: editable OpenPencil, Style Dictionary tokens, SVG/SVGO/resvg vector work, Sharp raster operations, D2 diagrams, controlled fonts/assets and checked web/component/handoff exports, with Codex authoring project code. Keep HyperFrames for editable HTML/CSS/JS video compositions, private Chromium/Puppeteer capture and a controlled FFmpeg/ffprobe encoder for the selected formats/audio/captions. Lottie/vector content may be an editable local asset within the HyperFrames composition; a separate ThorVG/Skottie renderer is not selected. Reuse one private browser package across browsing and video only after exact compatibility testing; do not silently add a second engine. The currently inspected Chromium snapshot remains **rejected** for free-license/source-notice reasons, and HyperFrames/FFmpeg/OpenPencil native release gates remain open. This is a product-scope decision, not admission of a particular new binary or permission to alter the installed client, commit, push or publish during review.

## Decision 94 — Rebuild the skill catalog around global methods and project specialists

On 2026-10-25 the user requested a fresh, extensive skill catalog grounded in the retained original references: roughly eight to ten general-purpose global skills and sophisticated project-local choices. The [proposed catalog](skill-catalog-design.md) specifies nine global methods and 58 opt-in project specialists, grouped by decision and deliverable rather than copied upstream repositories. It preserves the thirteen subject outcomes in decision 90 while superseding decision 92's one-global/27-project draft. Exact IDs, source mappings, selection profiles and acceptance remain proposed for review; they are not installed or release-qualified by this planning change.

Project opt-in and approved-selection sync remain as in decision 92. Mirket-first guidance and Codex-owned agents remain as in decision 91. New references are not silently added, and reference selection does not admit dependencies or third-party text/assets. Implementation stays paused until the revised plan is agreed.

## Decision 95 — Use descriptive, unbranded skill names

On 2026-10-25 the user rejected the `mirket-` prefix for skill names. The [proposed catalog](skill-catalog-design.md) now uses task-based IDs for all nine global and 58 project skills. Native directory names match those IDs. Setup and project sync identify managed copies from the release manifest and exact hashes, not a naming prefix; they refuse collisions with unmanaged native skills and never silently rename an ID. This supersedes the example `mirket-core` name in decision 92 and the prefixed names first proposed under decision 94. No installed skill is renamed by this planning change.

## Decision 96 — Remove Mirket C/C++ language tooling

On 2026-10-26 the user explicitly dropped C and C++ language tooling from
Mirket. Codex and each project's own compiler, language server and toolchain
handle those languages. Mirket keeps selected-source JS/TS/JSX/TSX navigation
and the five named SQL profiles. The release code, native skill surface and
tests already had no active clangd route; the unused C/C++ admission record and
standalone qualification documents were removed after a recoverable source
snapshot. This does not remove C/C++ code from upstream tools that Mirket itself
uses, such as Chromium, FFmpeg or native libraries.

## Decision 97 — Acquire bulky tools during setup; stop the Chromium source build

On 2026-10-26 the user stopped the exact Chromium source-build route and chose
an existing browser build acquired by the installation script through Mirket's
setup flow. The general packaging rule is to move a troublesome bulky tool to
a separate first-party setup acquisition when this reduces the Mirket download
package, while preserving fixed versions, verified archive and extracted-file
hashes, private `.mirket` paths, explicit vendor terms and native behavior
checks. Do not use the user's existing browser or choose a floating latest
version at install time. Complete the existing release manifest in private
staging before invoking the one setup/update transaction; no runtime download
occurs on first use.

The first candidate is official Chrome for Testing 156.0.8078.47. It passed
Mac ARM64 development browser and MP4 journeys, but normal installation,
visible mode, all video formats, native Windows and Codex Desktop GUI activation
are still open. Chrome's executable is subject to Google's terms and is not an
all-open-source binary; this supersedes the source-build and all-open-source
browser selection in decision 93. The official download catalog has no native
Windows ARM64 Chrome for Testing archive; using its `win64` build under Windows
ARM64 emulation requires a native test. [Candidate evidence](../docs/m3-browser-setup-acquisition.md)
records exact URLs, hashes and the Mac signing limitation. The old source-build
evidence is retained privately and is no longer a release gate.

## Decision 98 — Acquire semantic assets at setup; qualify prebuilt raster

On 2026-10-26 the user accepted applying the browser acquisition pattern to
other costly third-party payloads where it simplifies delivery. The selected
Mac MiniLM model files and ONNX Runtime archive move to first-party setup
acquisition with exact revision, sizes, hashes, private release paths and no
first-use download. The canonical SQLite memory owner and offline retrieval
contract stay unchanged. [Disposable Mac setup evidence](../docs/m2-semantic-setup-acquisition.md)
records both direct HTTPS and supplied-file paths; Windows remains gated.

Investigate the exact upstream Sharp/libvips prebuilt packages as the route out
of a custom native source build. The [Mac pilot and Windows static inventory](../docs/m4-raster-prebuilt-qualification.md)
show technical promise but do not admit Raster to normal setup. Its private
resource paths, source/notice/recipient terms, installed operations and native
Windows behavior still require qualification. Node and D2 remain in their
already working private release selections because moving them now saves little
engineering work; no floating npm/pip install or host PATH fallback is adopted.

## Decision 99 — Permit explicit external native tools when private packaging is costly

On 2026-10-26 the user clarified that Mirket need not privately install every
dependency. Keep an exact private installation where it is manageable; an
external tool is allowed when it materially reduces packaging and maintenance
cost. External selection must be explicit at setup, validated for the needed
features and recorded for doctor/runtime drift checks. A job must not silently
switch to another executable on PATH. This broadens decision 97's private-only
acquisition rule for native tools; it does not change the private browser or
semantic selections already implemented.

The upstream Sharp prebuilt passed [Mac Rust/MCP raster jobs](../docs/m4-raster-prebuilt-qualification.md)
and remains a small, manageable private package candidate. An explicitly
selected host FFmpeg 9.0.2 pair passed [four Mac development video formats](../docs/m3-host-ffmpeg-development-qualification.md),
but that Homebrew executable links to mutable libraries outside the pair.
The already qualified, self-contained private FFmpeg successor is the simpler
Mac release candidate. External binding remains available for a future target
where private packaging is genuinely worse. Source/terms handling,
minimum-OS and Windows qualification remain open. These tests do not admit
Raster or Video to a normal release.

## Decision 100 — Retire the custom Raster recipient path

On 2026-10-26 the user confirmed that troublesome native tools may be
installed separately while manageable tools should remain private. The six
official, locked Sharp archives reconstruct the tested Mac module exactly,
so the closed Raster candidate now pins that upstream prebuilt tree. The
custom source-built libvips candidate, 1,563-line recipient owner and its
import/rollback CLI were removed from active code. Their records remain
historical evidence, separate from the current admission selection. No normal
Raster release was admitted, and no installed user state was changed.

First-party setup still needs to acquire and verify the selected archives
before a complete release manifest can be activated. Source/notice handling,
installed Mac jobs and native Windows x64/ARM64 remain gates. This choice
removes a dormant alternate native stack; it does not treat an npm package
receipt or local render as release acceptance.

## Decision 101 — Codex owns general browsing and SQL

On 2026-10-26 the user approved both dependency simplifications. Remove Mirket's public browser sessions, article extraction, interaction and capture tools, their Playwright/Readability adapter path, and its SQL navigator with private Python/SQLGlot payload. Codex Desktop and established project tools own ordinary browsing, live UI inspection, SQL source work and database queries. Retain the authored browser and SQL skills as scoped guidance for those host routes; neither skill promises a Mirket MCP tool.

Keep the exact setup-acquired Chrome for Testing selection solely for HyperFrames video rendering. It remains a Video prerequisite and has no separate Browser release component or general browsing API. Preserve selected JS/TS navigation, hybrid memory, design machinery and the video candidate. Historical browser/SQL qualifications remain evidence of the superseded design, not current product readiness. Installed Video and native Windows verification remain open; this decision does not admit an unqualified release or authorize publication.

## Decision 102 — Keep only distinctive Mirket ownership in the first release

On 2026-10-27 the user accepted the concrete dependency, memory and topology simplifications. Codex owns general file reading/editing, documents, shell/Git, browsing and SQL. Mirket removes its generic file/range/artifact reads, text-replace-copy operation, duplicate job-result endpoint and core text adapter. Capability-specific design, diagram, video, navigation, memory and job-status tools remain. The semantic index keeps canonical SQLite/FTS and MiniLM but computes bounded cosine scores in Rust rather than loading sqlite-vec. Its warm model expires after 60 seconds of inactivity, subject to measured idle and cold/warm checks.

Finite effectful jobs keep prepare/start and `job_status` because a one-call effect with only a server-issued operation ID cannot be reconciled safely after the response is lost. The native skill catalog remains nine global and 58 project skills; all receive integrity, activation and nontrigger screening, while matched held-out comparisons focus first on effect-bearing, security, memory, coordination and output-quality skills. Mirket guidance is mandatory for Mirket-owned operations and does not displace Codex's host-owned tools. Mac ARM64 is the 0.0.1 release target. Windows 11 x64 and ARM64 are planned targets with native release gates, not an implied simultaneous release. The HyperFrames CLI is a comparator to examine, not a second renderer or permission to remove the currently qualified candidate before replacement evidence. The old harness installation and repository remain untouched; no commit, push or publication was authorized.

## Decision 103 — Waive the Desktop GUI test

On 2026-10-27 the user instructed us to ignore the Codex Desktop GUI test and
finish the remaining rewrite work. The host computer-use tool explicitly
refuses control of `com.openai.codex`; a macOS Accessibility permission does not
remove that tool-level restriction. The already completed installed MCP,
Codex app-server and noninteractive Codex CLI model turns remain the available
client evidence. Continue technical, skill and package verification through
those routes. Report actual Desktop GUI loading and interaction as untested,
without claiming that backend success proves it. This waives the GUI test as a
release gate; it does not authorize publication or modification of the user's
existing installation.

## Decision 104 — Focus motion work exclusively on HyperFrames

On 2026-10-27 the user dropped Remotion from the plan and selected HyperFrames
as the only motion/video option. Cancel Remotion comparator trials, production
admission, separate host-runtime integration and migration proposals. Retain
prior research and licensing findings as historical evidence only. No Remotion
runtime dependency was added to the production closure.

Continue HyperFrames feedback-loop work: author-visible preview, saved-pixel
review, targeted repair and decoded-export verification with editable sources.
Codex owns model choice and agents; one shared review/repair budget applies.
The redistribution and permitted direct-vendor/host-acquisition requirements
remain applicable to the selected dependencies. This decision does not change
the live installation or qualify untested output/platform paths.

## Current step

The user approved the [consolidated implementation plan](implementation-plan.md) on 2026-10-25 and authorized execution. Decisions 102–103 narrow the core, set first-release platform scope and waive GUI acceptance. The [working simplification review](codex-native-simplification-review.md) records earlier alternatives; the revised plan owns current scope. Local and CLI passing tests do not establish Windows support, Desktop GUI activation or a complete distribution. No commit, push or publication is authorized by plan approval.

Subsequent state on 2026-10-27: the former installation was retired and the
rewrite installed under the user's separate authorization; this repository is
now the current `harness` checkout. The deep-review task permits focused local
commits after verification and requires preserving the live installation. It
does not authorize pushes or publication. Earlier decisions above retain their
historical authorization boundaries rather than describing the current
installation as still coexisting with its predecessor.

Historical reviews and their baseline manifests remain unchanged. Implementation progress and temporary coordination records live outside shipped source under the task-state directory.
