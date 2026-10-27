# Mirket rewrite — independent review synthesis

2026-10-24. All four requested reviewers completed using `gpt-6-astra` with High reasoning. Each received a fresh context, the same six-file planning baseline and its own remit; reviewers were instructed not to read peer reports or delegate recursively. Architecture completed before Integration started because only three child-agent slots were available. This document integrates their findings; it is **a recommendation, not an amendment to the accepted decisions or authorization to implement**.

## Verdict

**Keep the topology. Tighten the contracts and change the order of qualification.** The reviewers support host-owned agents, one Rust core, one compiled TypeScript adapter package, embedded memory, private tools and the two distinct Python owners. None establishes a need for a VM, broker, agent framework, cloud memory service or general plugin system.

The main complexity is the breadth of the promised tool distribution across three targets. Removing runtime components is a simplification only if their required behavior survives. The useful reductions are fewer overlapping interfaces, less upstream machinery in the package, earlier real-client tests and explicit ownership. The plan needs these bounded amendments before final approval; it does not need another open-ended architecture exercise.

These are planning findings and source-observed compatibility risks, **not a count of security vulnerabilities in an implementation**. No rewrite runtime was built or exercised. P1/high labels in the reports describe potential consequences if a gap survives implementation.

## Independent reports

| Reviewer | Report | Main contribution |
| --- | --- | --- |
| Architecture | [Architecture review](architecture.md) | Six findings: ownership/recovery, forgetting, uncertain effects, multi-file output, resource admission and milestone order. |
| Capability | [Capability review](capability.md) | Eight findings plus eight ranked resource groups: discovery, evidence expansion, browser interaction, guide evaluation, source rights and specialist methods. |
| Toolchain | [Toolchain review](toolchain.md) | Six findings: native PHP feasibility, passive Ruby loading, extraction topology, actual package dependencies, feature selection and embedding lifetime. |
| Integration | [Integration review](integration.md) | Nine findings and seven workflow walkthroughs: protocol/client compatibility, guide delivery, explicit scope, long jobs, abrupt disconnect, tool operations and inspection. |

The 29 findings overlap substantially. They should become the eight work items below, not 29 new subsystems or approval questions.

## Recommended amendments, in priority order

### 1. Prove the client-to-tool workflow before expanding the tool inventory

Record exact client/SDK versions and the supported MCP facilities tested together. Keep one canonical guide corpus, stable guide IDs and release-matched tool examples. Choose one qualified delivery route per host; a thin managed native entry can load focused guidance without copying a second independently maintained body. A `.mirket/skills` directory alone does not establish host discovery.

Every operation should carry explicit project/checkout context or an unambiguous configured default. Return the resolved context. Workspace roots and client names cannot identify a particular host conversation. Application handles should retain their owner, scope and release identity; changing projects must not silently retarget them.

The proposal's MCP reference is older than the current published revision, whose negotiation and optional task facilities have materially changed. Select a demonstrated common host/SDK contract rather than requiring the newest specification or building custom protocol compatibility. [Current MCP changes](https://modelcontextprotocol.io/specification/2026-07-28/changelog).

**Evidence to require:** both intended desktop Code surfaces discover a guide, invoke its real tool, inspect a result and continue correctly across two calls. Test two tasks in one checkout and two selected roots. This consolidates C1/C8 and I1–I3/I6.

### 2. Make jobs and process ownership survive abrupt failure honestly

Keep live children connection-owned. Share durable memory and qualified disk caches; do not accidentally introduce cross-client live analyzer sharing. Use one small installation-wide source of ownership/admission state for concurrency, profile exclusion, cleanup and update admission. A dead parent does not prove its children stopped using a resource.

Long work needs a qualified completion route. Prefer ordinary bounded start/status/cancel/result operations over the existing job owner unless a shared supported MCP facility demonstrably removes work. No durable agent resumption is implied. Separate execution status from effect knowledge: an error or accepted cancellation does not establish that nothing happened. Retain small local receipts for reconciling a saved path/revision after response loss; never automatically repeat an uncertain external submission.

The documented OpenAI MCP default tool timeout is 60 seconds. Claude Code documents approximately 500 ms before forced termination on macOS and 300 ms after stdin closure on recent Windows versions. These are documentation findings, not measured behavior of the target desktop installations. They establish why ten/thirty-minute server deadlines and a five-second internal cancellation grace cannot alone define client integration. [OpenAI MCP configuration](https://developers.openai.com/codex/mcp/), [Claude Code shutdown](https://code.claude.com/docs/en/mcp#how-claude-code-stops-a-stdio-server).

**Evidence to require:** drop a response after save; terminate the owner during finalization; reconnect while another client and updater contend. No duplicate mutation, premature profile reuse or false rollback claim. This consolidates A1/A3 and I4/I5.

### 3. Close the edit → save → inspect loop

Define a finite set of useful inspection and edit operations for each capability. An Office engine must expose enough bounded operations to edit a representative imported document; a generic UNO/Python executor is not a substitute for the selected host-owned execution boundary. Keep effectful operations distinguishable to the host without adding another permission service.

An artifact result should identify the saved revision, type, extent and available text/page/frame/range inspection. Verification reports should identify what was checked and what remains unseen, rather than returning a generic `verified` flag. Browser observations need scoped targets, stale-state handling and post-action evidence; article extraction and interaction state serve different purposes.

Specify the finalization unit: one file, one new owned artifact directory/archive, or host-owned project edits. Source/PDF pairs and HTML/assets must not become mixed-version successful outputs. Any limit on arbitrary in-place multi-file editing must be explicit if it narrows the expected product contract.

**Evidence to require:** stale browser target; timeout after a successful submission; clipped later report page; stale preview; interrupted multi-file export. This consolidates A4, C3/C8 and I6/I7. It requires focused operations and evidence reads, not a universal document model or viewer service.

### 4. Preserve the memory promises through retrieval and recovery

Make forgetting authority survive restoration of older scope data. Known deleted record lineages must remain excluded from reads, imports and snapshots, including backups completing concurrently. Content-free exclusion metadata needs a rollback policy distinct from ordinary memory. Do not expand this into semantic detection of arbitrary rewritten external copies.

Keep search results expandable to their source/revision. Distinguish no evidence, incomplete index, changed/unavailable source and budget truncation. Define applicability for critical constraints explicitly; if those constraints exceed the recall budget, report incomplete coverage. A similarity query cannot guarantee that every instruction relevant to the task was retrieved.

Use a compact host-curated continuation record with objective, current constraints, scope, evidence pointers, unfinished work and uncertain effects. Recovered memory is evidence, not permission or a replacement for the host's conversation lifecycle.

**Evidence to require:** forget → old restore → old import with interruption; decisive evidence just beyond an excerpt; changed source on expansion; constraint overflow; simultaneous tasks in one checkout. This consolidates A2, C2/C6 and I9 within the existing memory system.

### 5. Resolve the concrete toolchain conflicts through bounded qualification

**PHP:** Phpactor explicitly documents Linux/macOS and WSL for Windows. An x64 PHP worker on Windows ARM64 does not resolve that OS requirement. PHPantom 0.11.1 publishes native archives for all three selected targets and is a credible candidate to qualify, not an approved replacement. Its passive-loading behavior, configuration precedence, completeness and reuse rights still need the exact dependency review and runtime evidence. [Phpactor requirements](https://github.com/phpactor/phpactor/blob/master/README.md), [PHPantom release](https://github.com/PHPantom-dev/phpantom_lsp/releases/tag/0.11.1).

**Ruby:** upstream Ruby LSP assumes trusted workspace code and documents normal project-code execution. Test a preassembled passive route early, including useful navigation, rather than assuming flags or a private interpreter eliminate project loading. The same preparation contract should cover other risky analyzers: host action, accepted metadata, source/target identity, freshness and a passive return path. [Ruby LSP security](https://shopify.github.io/ruby-lsp/security.html), [composed bundle](https://shopify.github.io/ruby-lsp/composed-bundle.html).

**Extraction:** correct the stale claim that current docling.rs defaults to PDFium. The inspected tag uses a Rust renderer by default; its published conformance figures use a particular renderer/model/OCR configuration. Those figures do not prove Mirket's multilingual quality or resource advantage. Run one bounded comparison of the selected Python pipeline and challenger on the same required fixtures, selecting one extraction owner if a replacement earns its place. [Tagged conformance and renderer documentation](https://github.com/docling-project/docling.rs/blob/v2.51.1/docs/PDF_CONFORMANCE.md).

**Packaging:** inspect the actual resolved OpenPencil/HyperFrames imports, native modules, model features and browser paths. Account separately for assembly tools and shipped files. Disable unnecessary dependency features where supported; measure complete installed and staging bytes on each target. Sharing a Chromium executable is allowed only when compatible and does not imply profile sharing. This consolidates T1–T5 and I8. No new production dependency is approved by these reports.

### 6. Fix resource admission before tuning speculative targets

The proposed two analyzers can remain idle for five minutes while an ordinary navigation call has a one-minute deadline. Treat five minutes as maximum idle retention and reclaim idle slots on demand. Separate bounded queue waiting from execution deadlines. Include embedding rebuilds and analyzer startup in admission for memory-intensive work, not only renders/extraction.

Specify lazy embedding load and idle release, then measure both connected clients after they have actually used semantic recall. Retain the selected concurrency and soft budgets until measurements justify tuning. Do not add an embedding server to meet an unmeasured idle target. This consolidates A5/T6.

### 7. Author useful guides with exact reuse boundaries and comparative tests

Keep the twelve-guide structure and the 52 selected references. Map actual source sections to retained techniques, license/use basis, discarded host assumptions and a Mirket acceptance case. Whole source families do not have uniform reuse terms. In particular, the selected Anthropic DOCX material restricts derivative/distribution use; keep it link-only absent separate permission and independently author the Office method from reusable sources. [License at the retained pin](https://github.com/anthropics/skills/blob/683bc88e56f3e09ba94f7055977f3d3aa499f202/skills/docx/LICENSE.txt).

Evaluate activation separately from execution. Compare the shared workflow alone with the same workflow plus a guide, using held-out failures, trivial bypass, missing tools and mixed tasks. Inspect artifacts and decisions, including unnecessary calls/questions, instead of scoring whether the guide was loaded.

The highest-value reference additions proposed by the Capability Reviewer are Agent Skills specification/evaluation, targeted Trail of Bits methods, QMD's search-to-source interface, Sentry's skill review methods, statistical-analysis methodology and W3C accessibility criteria. Vercel/Pi/Hermes/agent-browser are narrower conditional or comparison references. **No addition is made to the approved catalog here.** Reuse restrictions and exact coverage are recorded in the capability report; these are sources of methods, not a proposed runtime bundle. This consolidates C4/C5/C7 and I9.

### 8. Change the milestone order without shrinking the release

Split M0 internally into a minimal client integration path and focused risk experiments. First establish real client discovery, explicit scope, one owned child, cancellation/abrupt death and synthetic two-release activation/reconnect. Include two thin guide-driven journeys rather than postponing all guide usability to M4.

Then qualify the risks that could force product choices—passive analyzers and native Office/Windows ARM64 extraction—followed by the remaining accepted capabilities. Successful slices can feed M1/M2 while unrelated experiments continue. The same complete three-target capability matrix remains required before release.

Keep M5's full package/update/recovery checks and the user's simplified GitHub/checksum/native-verification release strategy. The architecture report's mention of a “signed distribution” must not be read as reinstating the rejected custom signing infrastructure or making an unapproved publisher purchase mandatory. This is A6 integrated with the other reviewers' early workflow recommendations.

## Disposition and limits

All report findings are represented above. The synthesis agrees with their bounded contract corrections, but exact field names, lock APIs and tool schemas remain implementation work. PHPantom and docling.rs are qualification candidates, not adopted replacements. New references remain suggestions. Multi-file in-place editing could require a product choice if the supported boundary narrows an intended capability; other changes largely make existing promises executable.

Do not respond by adding a model/router, always-running broker, second memory system, interchangeable fallback fleet, generalized filesystem transactions or a universal execution tool. Keep these corrections in the existing owners and ordinary release inventory.

Reviewers used plan inspection and primary-source research. The Capability Reviewer inspected selected portions from nine approved source families and additional public skill resources; it did **not** audit every file in all 52 selected repositories. The Toolchain Reviewer documented several fetch limitations and alternative primary sources. No reviewer measured performance, package size, navigation correctness, document fidelity or actual desktop activation.

The integrating reviewer rechecked the source claims about Phpactor/Windows, PHPantom release assets, Ruby's trust/loading model, docling.rs renderer/conformance, Anthropic DOCX terms, current MCP changes and documented host timeouts/shutdown. Baseline integrity and report/link structure are checked locally. CLI documentation is evidence of a risk to test, not proof of identical behavior in either desktop Code surface.

The six planning documents remain byte-for-byte unchanged, recorded in [baseline.json](baseline.json). Only this review bundle was produced. The old harness, installation and client configuration were not changed; no implementation, dependency installation, commit, push or publication occurred. The next planning action is to fold the agreed bounded amendments into the existing plan, then obtain complete-plan agreement before beginning the rewrite.
