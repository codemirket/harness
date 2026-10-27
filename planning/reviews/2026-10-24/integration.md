# Independent integration review

2026-10-24 · Integration Reviewer · requested Astra / High

## Executive verdict

**Retain the architecture; revise the host-facing integration contract before treating the plan as implementation-ready.** The private tool inventory is much more developed than the path by which a host discovers a guide, invokes its required operations, inspects the result and records useful context. The missing pieces can be resolved with a small contract and a few early vertical slices. They do not justify a broker, provider orchestration, another agent runtime or a general plugin framework.

The most concrete compatibility risks are an unspecified MCP protocol baseline, long operations exceeding Codex's documented default timeout, and disconnect cleanup assuming more time than Claude Code documents. The largest product risk is implementing broad engines first and discovering in M4 that the guides cannot reliably operate or verify them through the exposed interface.

I verified all six assigned files against `baseline.json`; every SHA-256 matched before this review. I read `implementation-plan.md`, `proposal.md`, `decisions.md`, `references.md`, `toolchain-proposal.md` and `platform-qualification.md`. I did not read any peer review. Evidence is plan inspection and fetched primary documentation only: **no implementation, host activation, analyzer, renderer, installer or runtime behavior was tested**. Severity below describes the consequence if the gap survives implementation, not a demonstrated exploit or existing implementation defect.

## Prioritized findings

### 1. P1 — Select a real host/SDK protocol baseline before designing connection-dependent behavior

**Locations:** `proposal.md:46`; `implementation-plan.md:84`, `implementation-plan.md:109`, `implementation-plan.md:124`.

The proposal cites MCP 2025-06-18 and delegates compatibility to an unspecified official SDK. That is not enough to design sessions, roots, guide delivery and jobs. The current published revision fetched for this review is 2026-07-28. Its changes include per-request negotiation instead of initialization, `server/discover`, changed roots exchange, and tasks moved to an extension. This is material interface evolution, not a patch-level library choice. [MCP changes](https://modelcontextprotocol.io/specification/2026-07-28/changelog).

**Broken workflow:** the foundation is written around initialization instructions and connection-scoped project state, while a qualified host/SDK combination negotiates another revision or does not support the chosen extension. A successful trivial tool call does not expose the mismatch; guide loading, project discovery or deferred results subsequently fail. Conversely, adopting the newest specification does not prove either intended desktop client supports every facility.

**Smallest correction:** M0 must record exact host versions, SDK version, negotiated protocol revision and actually exercised facilities. Select the smallest common supported contract. Use the SDK's supported negotiation; do not write a second protocol stack. Distinguish an owned stdio process lifetime from a protocol session and from an application browser/job handle. Keep application state explicit in tool arguments, independent of protocol generation. Test discovery, errors, one multi-call handle, cancellation and result delivery in both real Code surfaces before M1 freezes interfaces.

**Tradeoff / confidence:** a small compatibility fixture now avoids an adapter rewrite later. Supporting an older qualified revision is acceptable; neither “latest” nor custom compatibility machinery is required. High confidence in the documentation mismatch; actual client/SDK compatibility is untested.

### 2. P1 — The twelve guides have no selected delivery path, and the first guide-to-tool test is too late

**Locations:** `proposal.md:88`, `proposal.md:241`, `proposal.md:301–324`; `implementation-plan.md:124`, `implementation-plan.md:140–159`.

A directory under `.mirket/releases/.../skills`, a searchable index and an explicit read API do not by themselves place activation descriptions or the shared workflow in a host's initial context. The plan also has no decision about native skill installation versus MCP-served instructions, name collisions, relative reference resolution, or matching a guide's revision to the connected tool release. M4 authors the actual guides after M3's entire tool surface.

Codex documents native skill locations and metadata-first loading; those locations do not automatically include `.mirket`. Claude Code also distinguishes skill discovery from loading a skill body. The newer MCP Skills extension explicitly separates reading a resource from host skill activation, and says host/SDK support is still being implemented. [OpenAI skill documentation](https://developers.openai.com/codex/skills/), [Claude skills](https://code.claude.com/docs/en/skills), [MCP Skills extension](https://modelcontextprotocol.io/extensions/skills/overview).

**Broken workflow:** “Edit this spreadsheet and check the totals” never loads the document guide, or loads a cached guide pointing to operations from a different release. The model instead uses an unrelated installed skill, invents a private binary command, or receives all twelve guide bodies to compensate. Package tests still pass.

**Smallest correction:** select one canonical guide corpus and one qualified delivery route per host. A thin managed native entry/index that reads selected release-bound guidance is a sufficient baseline; use native skill delivery or the Skills extension only where qualified. Do not maintain independent native and MCP-authored copies. Specify a short bootstrap, exact discovery/read route, bounded supporting-file reads, missing-tool behavior and guide/tool release identity. Add one engineering and one document guide slice to M0/M1; M4 can finish the remaining content. Keep the host's discovery mechanism authoritative rather than introducing a second general tool search/routing system.

**Tradeoff / confidence:** a little authoring moves earlier; the twelve-guide structure remains unchanged. High confidence in the delivery omission; the best route is a qualification result, not established here.

### 3. P1 — “Host workspace” must become an explicit per-operation scope contract, not inferred task identity

**Locations:** `proposal.md:213–231`, `proposal.md:378–380`; `implementation-plan.md:42`, `implementation-plan.md:126`.

The plan correctly rejects a mutable global project and provides an explicit fallback, but it never specifies the information actually crossing the host boundary. Workspace roots do not identify the active task, distinguish several tasks in one checkout, or prove which selected root a particular call intends. Current MCP roots are informational and deprecated in favor of parameters, resource URIs or configuration. [Current roots contract](https://modelcontextprotocol.io/specification/2026-07-28/client/roots).

**Broken workflow:** two host agents work in the same repository, or one conversation selects two roots. An implicit memory save is attached to the wrong checkout/task, or a session handle is reused after a scope change. Checkout-scoped task state alone cannot distinguish two unfinished tasks in that checkout. A root list or client name cannot supply the missing correlation.

**Smallest correction:** every project operation accepts an explicit selected root or resolved project/checkout handle; only an unambiguous configured root may be defaulted. Return resolved scope in results. Browser/artifact handles bind their actual project, owner and release, and scope-changing calls cannot silently retarget them. Let host-curated task records use an explicit caller-supplied correlation key when needed, with no claim that it authenticates a host thread. Treat host thread IDs, visible-app state, transcript access and automatic task lifecycle notifications as unavailable unless the selected integration demonstrates them. Test two tasks in one checkout, two roots, a subagent, deselection and reconnect.

**Tradeoff / confidence:** a few explicit arguments replace fragile implicit discovery; no registration ceremony or task scheduler is added. High confidence in the missing correlation contract. The existing refusal-on-ambiguity policy is good and should be retained.

### 4. P1 — Long jobs need a host-visible completion and uncertain-outcome contract

**Locations:** `implementation-plan.md:61`, `implementation-plan.md:65–66`, `implementation-plan.md:124`; `proposal.md:204–206`.

Ten-minute extraction and thirty-minute video deadlines do not fit Codex's documented default 60-second MCP tool timeout. A server-side deadline does not configure the host. [Official OpenAI MCP configuration](https://developers.openai.com/codex/mcp/). The planned envelope enumerates complete/partial/unsupported/error, but not accepted, queued, cancelling, cancelled or outcome-unknown. The rule against replaying uncertain effects is therefore not yet operable by a caller.

**Broken workflow:** a render or in-place save commits just as the host abandons the call. The model receives a timeout, cannot determine whether an artifact exists, and starts another attempt. A queued job can also outlive the caller's request without any usable status handle. Browser submission is more consequential: cancellation cannot undo a remote acceptance.

**Smallest correction:** choose one qualified mechanism: bounded start/status/cancel/result operations over the existing job owner, or a supported MCP Tasks route. Do not assume an extension or silently add durable execution; the current Tasks extension has explicit negotiation and durable-handle semantics. [MCP Tasks](https://modelcontextprotocol.io/extensions/tasks/overview). A synchronous route is also possible if setup configures and verifies the needed host timeouts, but it still needs reconciliation after response loss. Define commit point, operation identity, effect status, retained result/staging location, retry safety and result retention. A new RPC request ID must not automatically mean a new logical mutation. Record minimal receipts sufficient to inspect an uncertain effect without resuming the original process.

**Failure probe / tradeoff / confidence:** commit a new-copy and an in-place artifact, then drop the response; reconciling must identify the actual saved revision and must not repeat the mutation. Separately test a submitted browser action with no response. This adds a small lifecycle surface already justified by queued jobs, not an agent journal. High confidence in the timeout mismatch and state gap; no timeout was executed.

### 5. P1 — Five-second graceful cleanup cannot be the disconnect safety mechanism

**Locations:** `implementation-plan.md:65`, `implementation-plan.md:70`, `implementation-plan.md:126`; `proposal.md:203–206`.

Claude Code currently documents only about 500 ms before forced termination on macOS and 300 ms after stdin closure on recent Windows versions; older Windows versions terminate immediately. [Claude stdio shutdown](https://code.claude.com/docs/en/mcp#how-claude-code-stops-a-stdio-server). The plan's five-second grace can be a local cancellation budget, but cannot be assumed available on host disconnect.

**Broken workflow:** the host exits during an Office save or artifact finalization. Mirket never finishes its graceful cleanup, leaving an uncertain final path, a stale profile/queue reservation or an owned descendant. A test that calls Mirket's own cancel method and waits five seconds will miss this.

**Smallest correction:** distinguish explicit cancellation from abrupt owner death. Make artifact commit records and resource ownership crash-safe before effects occur; correctness cannot depend on a shutdown handler finishing. Use qualified OS process-tree containment for owned children and OS-backed lock release; recover stale reservations/staged output at the next ordinary launch. Reconcile committed output before clearing recovery state. The host controls teardown timing, so shutdown handling should be best effort within the actual host window.

**Tradeoff / confidence:** modest crash recovery work replaces an invalid timing assumption; no daemon is required. Test real client close/reconnect and forced process death during each commit phase, with unrelated tools open. High confidence in the documented conflict; actual desktop timing remains to be measured.

### 6. P1 — Engine selection is not yet a useful, permission-reviewable tool surface

**Locations:** `proposal.md:237–250`, `proposal.md:433–437`; `toolchain-proposal.md:75`, `toolchain-proposal.md:242–245`; `implementation-plan.md:124`, `implementation-plan.md:136`.

The plan names strong engines but leaves the callable operations and supported document content largely unspecified. A “narrow UNO controller” could be too narrow to author tables, layouts and formulas, or become a generic script/property-dispatch endpoint exposing much more than document editing. Similar ambiguity exists for browser evaluation, OpenPencil operations and prepared rendering. One TypeScript package is a packaging choice, not a reason for one opaque `run` tool.

**Broken workflow:** the document guide must inspect a DOCX's blocks/styles, replace one table cell and preserve headers. The adapter only offers convert/export, so the agent rebuilds the document; alternatively it receives arbitrary UNO/Python execution and bypasses the intended host-owned shell boundary. A generic browser executor can also mix read inspection, downloads and submissions under one approval rule.

**Smallest correction:** define a finite capability-to-operation manifest as each M0 slice lands: stable names, input/output schemas, addressable source objects/ranges, supported edits/features, effect class, required inputs, limitations and verification operation. Provide enough inspection and bounded edit primitives for representative imported files. Keep raw Node/Python/shell/UNO dispatch out of the public surface; if specialized scripting is genuinely required, expose and review it as execution, not as a harmless renderer. Keep web browsing, asset acquisition, local authored rendering and external submissions distinguishable to the host. Existing authorization remains valid; do not invent a second consent service or an agent-supplied `approved: true` security boundary.

MCP exposes per-tool schemas and result errors; Codex can apply per-tool allow/deny and approval policy. Tool naming and effect separation therefore affect actual host control. [MCP tools](https://modelcontextprotocol.io/specification/2026-07-28/server/tools), [OpenAI MCP tool policy](https://developers.openai.com/codex/mcp/).

**Tradeoff / confidence:** this may reveal missing upstream primitives early. Reuse engine models and typed patches instead of designing a universal document or workflow language. High confidence in the specification gap; no claim that a generic executor has already been implemented.

### 7. P1 — Returned paths do not close the render-and-inspect loop

**Locations:** `implementation-plan.md:124`, `implementation-plan.md:136`; `proposal.md:405`, `proposal.md:427–437`; `toolchain-proposal.md:198`.

The guides require inspecting saved artifacts, but the common result contract only explicitly promises artifact paths/source locations. The plan does not choose how either host obtains bounded pixels, pages, sheet ranges or audio/video evidence from that saved revision. Nor does it distinguish engine validation from host visual/semantic review. A path and renderer success are insufficient for the promised verification workflow.

**Broken workflow:** a thirty-page report exports successfully, but a table is clipped on page 17. The agent sees a path or first-page thumbnail and marks the artifact verified. An MP4 may have moving frames but an inaudible or mistimed supplied track. A preview produced before the last edit can also look correct while the final file is wrong.

**Smallest correction:** provide an artifact descriptor with saved path, media type, content identity, page/sheet/frame extent and available inspection routes. Return bounded MCP image content or provide an explicit page/frame/crop read operation, with text/table/formula reads tied to the same saved revision. Define the host's media playback/inspection route separately; if it is unavailable, report which checks remain unverified. List coverage rather than a generic `verified` boolean. Keep full-resolution assets on disk without putting entire documents into context. MCP already has image/resource content primitives; no custom viewer service is necessary. [MCP content/results](https://modelcontextprotocol.io/specification/2026-07-28/server/tools).

**Failure probe / tradeoff / confidence:** deliberately clip a later page, corrupt one sheet formula cache, and mismatch the preview revision. The guide must discover the relevant evidence or explicitly withhold the verification claim. This adds focused read/preview primitives and bounded calls, not another renderer. High confidence in the missing delivery contract; actual client image/media behavior is untested.

### 8. P2 — Passive analyzers need a concrete preparation handoff, not only a refusal message

**Locations:** `proposal.md:285–297`; `toolchain-proposal.md:311–332`; `implementation-plan.md:117`.

The source review already identifies hidden project execution correctly. The remaining integration gap is how the Engineering guide transfers a missing prerequisite to the host, then binds prepared output back to the analyzer. There is no common contract for the preparation request, accepted metadata, freshness, source span encoding or supported project forms. A private analyzer runtime can also differ from the project's host toolchain.

**Broken workflow:** a Rust/Swift/C# lookup asks the host to prepare metadata. The host does so, but the analyzer cannot consume that exact output without importing the project again, or consumes stale metadata produced for another target/toolchain. The workflow either loops or presents incomplete references as resolved. SQL is notably better specified here: its schema input, source origins and strict-binding refusals are explicit at `toolchain-proposal.md:354–361`.

**Smallest correction:** for each high-risk M0 analyzer, document supported project forms and one round trip: prerequisite reason → concrete host action as data → accepted files/configuration and their source/target identities → passive refresh → revision-bound result. Include the negotiated position encoding and conversion to the public navigation contract. A prepared artifact does not authorize the analyzer to rerun its producer. Upstream rust-analyzer explicitly warns about configuration/toolchain execution routes beyond macros. [rust-analyzer security](https://rust-analyzer.github.io/book/security.html).

**Tradeoff / confidence:** this is an adapter recipe, not a new loader, build runner or SDK manager. If no upstream passive route consumes the required input, report that exact unsupported project form for a product decision before expanding the language surface. High confidence that the handoff needs specification; feasibility remains open and is already appropriately assigned to M0.

### 9. P2 — Critical-constraint recall and guide evaluation need observable selection criteria

**Locations:** `implementation-plan.md:67`, `implementation-plan.md:132`, `implementation-plan.md:159`; `proposal.md:483–485`, `proposal.md:516–524`.

The design promises direct retrieval of applicable critical constraints so similarity ranking cannot omit them. The canonical schema described so far does not define constraint identity, applicability selectors or how a caller declares the current task context. A general relevance query cannot guarantee those properties. Separately, M4 asks for representative guide exercises without a minimum discriminating corpus or a baseline that isolates whether the guide improves behavior.

**Broken workflow:** a short Turkish request paraphrases a release task without matching the saved constraint's keywords. Recall finds similar technical notes but omits the current “do not publish” constraint, or uses a different checkout's unfinished task. The host produces a convincing answer and stores its own conclusion as a verified lesson; repeated retrieval amplifies the error. The plan correctly says stored content grants no permission, but that alone does not catch the omission or unsupported verification label.

**Smallest correction:** define a small explicit constraint record/applicability contract and retrieval request context, keeping caller-supplied interpretations distinguishable from source observations. If applicable constraints exceed the budget, return incompleteness and an expansion route rather than claiming complete coverage. Preserve evidence identities for verification claims; do not infer human acceptance from a save. Add reviewed task-level fixtures alongside the 100 retrieval queries: correct/incorrect guide triggering, trivial bypass, overlapping guides, unavailable tools, same-checkout concurrent tasks, source contradictions, interruption and a misleading near-match. Compare the shared workflow alone with shared workflow plus the selected guide on the same tasks; record host/model/version and outcome, not just whether a guide loaded. Evaluations remain host-owned and do not imply provider access or paid calls.

**Tradeoff / confidence:** a compact schema and fixture set make the promised feedback loop measurable without passive transcript collection or another evaluator service. Medium-high confidence: details may be resolved naturally in M2, but the current completion criteria would allow a weak implementation to look successful. This finding is based on the plan's own stronger recall promise, not a claim that an embedding model can establish authority.

## Representative skill → tool → artifact walkthroughs

These are proposed qualification journeys, not executed tests. Operation labels describe required primitives; they are not invented existing API names.

| User outcome and lead guide | Host discovery and selected context | Mirket/host handoff | Artifact and feedback evidence | Discriminating failure |
| --- | --- | --- | --- | --- |
| Fix an API regression — Engineering + Verification | Load engineering entry and relevant test reference; focused project/checkout recall | Mirket symbol/definition/reference queries; missing metadata returned to host; host owns build/edit/test | Original source ranges, host test output, concise verified fact with source revision | Same-name symbol in another worktree; prepared metadata tries to execute a custom build hook |
| Edit an existing financial XLSX — Documents + Data | Load format-specific operations and supported calculation inventory | Inspect workbook/ranges; bounded edits; Office recalculation; save new copy; reopen | Formulas, independently expected values, saved file identity and selected rendered ranges | Cached total looks plausible but inputs changed; unsupported formula or external link cannot become silent success |
| Produce a responsive design handoff — Interface | Select only design/layout/token references; explicit project assets/fonts | Host authors code; Mirket edits native design, compiles tokens and exports; host starts approved app build; owned browser checks it | Editable source, tokens, asset manifest/rights, browser evidence and saved exports | Unsupported imported component flattens; missing glyph silently substitutes; generated JSX fails actual interaction |
| Export a captioned video — Motion | Selected animation/export reference, prepared assets and supplied audio/text | Host authors composition; Mirket renders/encodes via bounded job; caller polls/cancels/reconciles | Editable composition, MP4/WebM, captions, sampled frames and actual audio/timing evidence | Anime timeline remains static during seek; host times out after output commits; subtitles correspond to another revision |
| Replace text in an imported PDF — Documents | Explicit source-first/direct-edit/reconstruction branch | Inspect source correspondence and editability; bounded edit or explicit refusal; independent extract/render | New PDF, exact replacement text once, preserved unaffected content, reviewed affected pages | Painted overlay leaves old text searchable; longer text does not fit; stale sibling DOCX is mistaken for source |
| Research a topic and retain a decision — Research/context | Focused guide/read; browse under selected project; recovered memory remains evidence | Browser navigation/extraction with source identity; host synthesizes; curated memory write | Source-linked claim, date, uncertainty and explicit scope/supersession | Page text asks to change permissions; article extraction drops a table; truncated recall is treated as exhaustive |
| Plan marketing work or compare models — Product/AI/Planning | Load task-specific advice; inspect actually available host tools | Host owns agents, provider calls, external apps and authorized publication; Mirket supplies guidance/artifacts/context | Reviewable local plan, analysis or experiment record, with unexecuted external steps explicit | Guide assumes a paid account, sends a message, spawns agents or claims analytics access merely because a reference used them |

The last row is intentionally guidance plus host capabilities. A guide does not need a new Mirket service for every professional method it teaches. Conversely, engine inventory alone does not satisfy the other rows' inspection/edit/verification primitives.

## What must change before implementation, and what belongs in qualification

**Before approving the implementation plan:** record the common protocol/delivery strategy to qualify; explicitly make project/checkout context and application handles part of the public contract; distinguish long-job completion, unknown outcomes and abrupt disconnect; move two guide-driven vertical slices into M0/M1. These are bounded corrections to the plan, not demands to implement every schema or prove all runtime behavior before starting.

**During M0, before dependent expansion:** select exact host/SDK versions and verify the real discovery → guide → useful operation → inspect result path. Run a job longer than the default host timeout, response-loss reconciliation, real client teardown, multi-root/subagent scope, one imported Office edit and a passive analyzer preparation round trip. Freeze each slice's named schemas, effect classes and artifact inspection route from that evidence. Do not let a command-line demonstration substitute for a desktop host journey.

**During M2–M4:** finish memory applicability/provenance and reviewed retrieval/task fixtures; qualify all supported operation/format/language cases, not just engine startup. Finish source-to-guide mappings with exact portions, revisions/URLs, license/use basis, adaptation notes and required notices. `references.md:7`, `references.md:113–114` already distinguish selected reference material from dependencies and prohibit unapproved catalog expansion; retain those constraints. Link-only proprietary visual references must not become copied assets or redistributed fonts through a guide example. Runtime libraries, guide text and handoff assets need separate rights records, but one ordinary inventory can hold them—no new licensing service.

**During M5:** verify the installed guide and connected tool release agree after update/reconnect; detect managed-copy drift and host name collisions; exercise real files and bounded visual/media evidence on all three targets. Keep account/signing/publication authorization separate. Report exact versions and unavailable paths rather than a generic “both hosts supported” claim.

## What to retain

- Host-owned models, agents, shell, editing, Git and project builds; one Rust core and one compiled TypeScript package; no VM, broker or provider abstraction.
- One engine per actual capability, with narrow upstream adapters. Private browser parity has a real cross-host purpose; Puppeteer should remain internal to video, not become a second public browsing interface.
- The excellent separation of engine availability, packaging, invocation and output correctness. The source reviews generally avoid pretending that documentation is a runtime pass.
- Explicit partial/unsupported navigation, structural Kotlin, strict offline SQL and preparation assigned to the host. The SQL integration section is a good model for other capability contracts.
- Source-first PDFs, qualified direct edits, explicit reconstruction, real Office recalculation, originals preserved and no success inferred from a ZIP/XML file or renderer exit code.
- Prepared local assets, controlled fonts, no first-use downloads, and rights checked independently for tools, models, fonts and reference material.
- Agent-curated memory, canonical revisions/scopes, exact retrieval during semantic rebuild, forgetting precedence and no automatic permission claims from memory.
- Maintenance-window activation and one release owner. Improve the client-facing stop/reconnect and crash paths without reinstating a supervisor or simultaneous-version state layer.

The needed simplification is a smaller, explicit set of callable workflows with shared evidence contracts. It is not a smaller promised product or another infrastructure layer.
