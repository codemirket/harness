# Independent integration review — second pass

2026-10-24 · Integration Reviewer · requested `gpt-6-astra`, High reasoning · planning only

## Verdict

**Ready for complete-plan approval and then authorized M0 qualification. No remaining integration planning blocker established.** The revised plan now connects host discovery, selected guidance, explicit scope, useful operations, saved artifacts and bounded inspection. It also makes lost responses and abrupt disconnects observable rather than treating a timeout as proof that nothing happened.

The nine first-pass integration findings are closed **as planning omissions**, with executable proof assigned to the appropriate milestones. This is not runtime closure: neither desktop client, a release package, an analyzer, a renderer nor an installer was exercised. Exact SDK versions, tool schemas and platform mechanisms should be established by approved M0/M1 work, not demanded before that work can begin.

One release-binding detail below deserves an explicit M0a fixture. It is a refinement of an existing invariant, not a reason to reopen the architecture or withhold approval.

## Independence, scope and integrity

I inspected the six frozen files and formed the above assessment before reading the preceding `integration.md` and `summary.md` for closure. I did not read any second-pass peer report. Relevant guidance was `skill-catalog`, `engineering-judgment` and its boundary/service-integration references; official OpenAI documentation was used for the changed host-interface claims.

All six SHA-256 values matched `pass-2/baseline.json` on entry:

| Frozen file | Verified SHA-256 |
| --- | --- |
| `planning/implementation-plan.md` | `fef82f450f3cc10d46e898d7ed8c2791c6a470a1804b75a15b6553a2255ea7a5` |
| `planning/proposal.md` | `38dbe27795237d9aa6fde0ba271532b417157556214d3ed634965df050bcb252` |
| `planning/decisions.md` | `fa3817ddb0e06b43c5f2b3749e62f0c57b1180f9e334f1108acf388fddc1323a` |
| `planning/toolchain-proposal.md` | `0d1ba5752d266e2fc24f6c005107b7daa18fcc7db3618c6c98ba101e4a192c8a` |
| `planning/platform-qualification.md` | `63ef46dcb12ca0de8f23bab1475683f44898d99c59902be29e8d5887f709ad30` |
| `planning/references.md` | `45b78012c9075c46c74da796a7a46ca79ab070b20e63c2fc0139290083709ad3` |

Only this report was written. The fresh rewrite directory has no application implementation or `.mirket/project.json`; no installation, upstream helper, runtime probe, dependency change, commit, publication or settings change was performed.

## Prior-issue closure

Line references below identify the revised frozen baseline, not the old report's line numbers.

| First-pass issue | Revised contract and verification owner | Disposition |
| --- | --- | --- |
| I1: unspecified host/SDK protocol baseline | `proposal.md:46` separates official SDK negotiation from explicit application handles and recognizes current protocol changes. `implementation-plan.md:112` requires actual host/SDK versions and exercised facilities in M0a; `proposal.md:243` defers schema freezing until useful host journeys. | Closed in plan. No claim that newest MCP, a particular SDK or both desktop clients already interoperate. |
| I2: missing guide delivery; guide/tool test too late | `proposal.md:215` selects one canonical corpus, thin managed native entries, release-bound reads, collision/drift checks and a qualified same-corpus native alternative. `implementation-plan.md:112` starts Engineering and Documents journeys in M0a; `implementation-plan.md:146` completes them in M4. | Closed in plan. The alternative is delivery of the same source, not a second authored corpus. See the cached-context fixture below. |
| I3: roots confused with task identity | `proposal.md:217` specifies per-operation root/checkout selection, returned scope, caller correlation keys and owner/scope/release-bound live handles. `proposal.md:265` excludes mutable global scope; `implementation-plan.md:136` covers concurrent task correlation. | Closed in plan. No assumed host thread identity, transcript feed or new registration ceremony. |
| I4: long calls and uncertain effects | `proposal.md:221` chooses bounded start/status/cancel/result calls and separates RPC IDs, logical operation IDs, lifecycle and effect knowledge. `proposal.md:223` defines reconnect inspection, commit receipts, retention, unknown/expired outcomes and no replay. `implementation-plan.md:112` tests loss after finalization and work exceeding the default timeout. | Closed in plan. Seven-day/50 MiB receipt retention has an explicit protected-record pressure behavior; saved artifacts are not receipt-cache entries. |
| I5: disconnect assumed five-second grace | `proposal.md:225` assigns connection ownership, combined admission and descendant evidence; `proposal.md:227` makes shutdown best effort and requires next-launch reconciliation. `implementation-plan.md:67` distinguishes explicit cancellation from abrupt death. | Closed in plan. A dead parent cannot alone free a profile or permit activation. No permanent reaper is introduced. |
| I6: engine inventory instead of callable, permission-reviewable operations | `proposal.md:219` requires finite schemas, object/range addressing, supported edits, effect class and inspection. It excludes disguised arbitrary dispatch and separates browsing, submission, acquisition and rendering. `implementation-plan.md:124` requires each slice to exit with callable operations and failure fixtures. | Closed in plan. Exact operation fields are appropriately slice-level work; package organization does not force one opaque execution tool. |
| I7: paths do not complete artifact inspection | `proposal.md:229` declares publication units and recoverable in-place boundaries. `proposal.md:231` requires saved revision, media type, extent and bounded pixel/text/table/formula/frame routes, with coverage and unavailable checks explicit. `proposal.md:243` and `implementation-plan.md:140` cover stale previews, later-page clipping and saved-file journeys. | Closed in plan. MCP/file-media delivery must be qualified in both clients; a path or renderer exit cannot satisfy the verification contract. |
| I8: passive analyzer preparation has no return path | `proposal.md:235` defines prerequisite → host action as data → identified metadata → passive refresh, including freshness and position encoding. `implementation-plan.md:118` moves risky analyzers early. `toolchain-proposal.md:336` requires a product decision if an upstream-supported route cannot meet the policy. | Closed in plan. Host preparation does not authorize its producer to run again; no replacement project loader or semantic engine is implied. |
| I9: constraint applicability and guide quality not observable | `proposal.md:237` adds explicit applicability selectors, caller task context, independent constraint retrieval and overflow reporting. `proposal.md:239` distinguishes observations/inferences/proposals. `implementation-plan.md:136` and `implementation-plan.md:163` require task-level correctness, negative triggers and comparative guide tests. | Closed in plan. Applicability is curated and scoped, not promised as universal semantic inference. |

The source-reuse corrections also preserve the integration boundary: `references.md:121` requires section-level provenance and terms; `references.md:123` makes restricted DOCX content link-only. Reference selection does not supply a runtime, execution permission or evidence that a guide's examples work.

## One nonblocking qualification clarification

**Exercise stale guide text already present in a continuing conversation, not only stale files.** Locations: `proposal.md:215`, `proposal.md:217`, `proposal.md:219`, `implementation-plan.md:112`.

The plan correctly requires release-matched guide delivery and invalidates old live handles. However, reconnecting a server does not itself establish that a continuing host conversation has discarded previously loaded instructions. Read/result release labels prove which version answered; they should not be mistaken for a pre-effect compatibility check.

**Concrete failure workflow to test:** a conversation loads release A's Documents guide, stops its Mirket connection, activates release B through the permitted maintenance window, then reconnects in the same conversation. The agent still has A's text and initiates a fresh root-addressed edit rather than reusing an old handle. If the same operation name has changed interpretation, a B-labelled result arrives only after the edit. The existing stale-handle check alone would not catch this path.

**Smallest correction/refinement:** make this a named M0a synthetic-release fixture. At schema freeze, choose an explicit pre-effect binding, such as an expected release/operation-contract revision carried with requests and returned by the guide read, or a qualified connection bootstrap that refuses operational calls until current contract context is established. A mismatch should request current guidance before effects. Resolve supporting-reference reads against the same guide revision. This can live in the existing read/operation schemas; it needs no host-context controller, custom MCP negotiation or second guide store.

**Tradeoff and severity:** a stricter full-release match may require rereading guidance after an otherwise compatible patch; a contract revision avoids that only if compatibility is deliberately maintained. This is **nonblocking M0a acceptance detail**, because `proposal.md:215` already forbids silent guide/tool mismatch and `proposal.md:243` explicitly leaves the mechanism to qualified schemas. I am not asserting that an implementation currently accepts stale operations.

## Integration paths tested by the plan

These are reasoning walkthroughs of the revised contracts, not executed tests.

| Workflow | Why the revised path is coherent | Discriminating execution evidence still required |
| --- | --- | --- |
| Edit an imported spreadsheet | Native entry → release-bound Documents body → scoped workbook inspection/typed edits → Office-owned recalculation → staged save/reopen → saved-revision formula and visual reads. `proposal.md:219`, `proposal.md:231`, `toolchain-proposal.md:248`. | Changed inputs with plausible old cached totals; unsupported formula; lost save response; actual saved/reopened values and inspected output in both clients. |
| Long video export interrupted by host death | A logical operation survives as a bounded receipt, while the live job remains connection-owned. Execution state and effect knowledge stay separate. `proposal.md:221`, `proposal.md:223`, `proposal.md:227`. | Death before/after commit, targeted descendant cleanup, reconnect receipt lookup without resuming the job, and actual audio/frame/caption checks. Unknown remote effects remain unknown. |
| Two tasks in one checkout plus a second client | Explicit correlation distinguishes task records; project/checkout handles select scope; live handles are not shared; one locked admission record coordinates capacity. `proposal.md:217`, `proposal.md:225`. | Same-checkout concurrent saves, cross-root handle misuse, queue cancellation, PID reuse and profile exclusivity while another owner remains active. |
| Navigation needing host preparation | The analyzer returns a concrete missing prerequisite; the host runs any authorized project command; identified output is passed back as data for passive refresh. `proposal.md:235`, `toolchain-proposal.md:319`. | A useful definition/reference after preparation, stale/wrong-target metadata refusal, exact Unicode spans and sentinels proving no producer/plugin execution on refresh. |
| Saved source/PDF or HTML/assets publication | One file or new directory/archive is a declared publication unit; explicit in-place sets require a recoverable boundary, not fictional filesystem atomicity. `proposal.md:229`. | Interruption between member writes; no mixed-version success; original/current outputs preserved; inspection tied to the finalized set. |
| Recall after interruption or an older restore | Task context, revision-bound expansion and explicit constraint applicability guide recall; forgetting authority survives ordinary scope rollback. `proposal.md:237`, `proposal.md:239`, `proposal.md:241`. | Decisive evidence beyond the excerpt, changed source during expansion, constraint overflow, and forget → old restore/import without resurrection. |

For permission qualification, ensure the finite operation inventory remains distinguishable at the actual host policy boundary after adding asynchronous starts. A single generic start endpoint must not erase the effect distinctions required by `proposal.md:219`. The existing contract already calls for this; test an inspection-allowed/submission-denied host policy rather than merely checking tool descriptions. No second permission system is needed.

For receipt qualification, include reconnect **after release activation**, not only restart on the same version. Retained receipts must remain inspectable through the current release or explicitly report their unavailable status without replay; they must not require reviving an old connection or executable. The existing operation identity, retention and state-migration owners are sufficient. There is no reason to add durable job execution or an agent journal.

## Focused primary-source checks

Checked on 2026-10-24. Unchanged toolchain evidence was reused; I did not repeat broad engine research or infer artifact quality from upstream documentation.

| Source | Checked claim and effect on this review |
| --- | --- |
| [MCP 2026-07-28 changes](https://modelcontextprotocol.io/specification/2026-07-28/changelog) | Confirms changed discovery/version exchange, explicit cross-call handles and Tasks as an extension. The plan's common qualified SDK/host contract is appropriate; protocol publication does not prove desktop support. |
| [Official OpenAI MCP configuration](https://learn.chatgpt.com/docs/extend/mcp?surface=cli) | The cited legacy URL redirects here. It documents a 60-second default tool timeout and per-tool approval controls. This supports bounded job RPCs and distinguishable operations; it is documentation evidence, not a measurement of the desktop app. |
| [Claude Code stdio shutdown](https://code.claude.com/docs/en/mcp#how-claude-code-stops-a-stdio-server) | Documents approximately 500 ms to forced termination on macOS/Linux, 300 ms after stdin closure on current Windows, and immediate termination on older Windows versions. Confirms that five-second local cancellation cannot be the abrupt-disconnect correctness mechanism. |
| [Agent Skills specification](https://agentskills.io/specification) | Confirms `SKILL.md`, name/description metadata and skill-root-relative supporting files. It does not guarantee that a host will load a Mirket MCP body from a thin entry. That route properly remains an actual M0a host test. |

No checked source invalidates the revised integration direction. Optional newer host facilities do not create a requirement to adopt them or justify another protocol layer.

## Readiness boundaries

Retain host-owned agents, models, shell, editing and builds; one Rust crate and one TypeScript adapter package; the two private Python owners; one complete release for macOS ARM64, Windows x64 and Windows ARM64; and the full agreed capability scope. The contract changes do not require a VM, WSL, broker, universal executor, custom protocol, model runtime or agent journal.

M0a now has a small enough useful host path to begin after approval. M0b can expose a real backend/policy conflict before dependent expansion, while successful slices proceed independently. M3/M5 still require full capability and three-target evidence. Unavailable hardware or an unqualified client remains a gate for the relevant claim; it is not a reason to demand proof before approved qualification starts or to label partial coverage complete.

**Planning verdict: ready. Runtime/release verdict: unverified.** Complete-plan agreement, exact dependency admission and authorized target access remain required. Publication and changes to the old installation remain outside scope.

## Closure after the bounded second-pass amendments

2026-10-24. I read the changed passages, without repeating the full audit or external research. **The nonblocking release-binding refinement is resolved in the plan.** `proposal.md:215` now requires expected full release identity on operational requests and rejects missing/mismatched identity before effects. Current discovery/guidance and scoped receipt inspection remain available, supporting reads bind the same guide revision, and silent identity refresh/replay is prohibited. The explicit same-conversation A-guide → B-release → new root-addressed edit fixture appears there and in `implementation-plan.md:112`. This directly tests the path that an old-live-handle test would miss.

The two accompanying integration checks are also explicit: `proposal.md:219` preserves host effect-policy distinctions across asynchronous starts and names the inspection-allowed/submission-denied test; `proposal.md:223` permits receipt inspection after activation without old executables or resumed handles and requires migration to preserve readable evidence or report its unavailability without replay. The receipt-expiry clarification retains read-only recovery access under pressure. The matched guide-evaluation controls at `implementation-plan.md:163` do not change these integration boundaries.

All six current files match `final_files` in `pass-2/baseline.json`. The changed final hashes are `implementation-plan.md` = `dcbff94e14321febedf0fd433344e1499f447075ea9a774f8dbd71b47bd7994f` and `proposal.md` = `33975fd1d7c275619b7b7bed1b4b3db946b85fc5be958a3addd167fc183b1dcb`; the other four remain unchanged. All six preserved inputs under `/Users/nazmiyilmaz/.codex/task-state/harness-rewrite-review-2/reviewed` match the original `files` hashes, preserving the basis of the review above.

**Final integration disposition: no outstanding planning correction; ready for complete-plan approval and authorized M0.** Pre-effect enforcement, real client discovery/cache behavior, receipt migration and host policy behavior remain unexecuted qualification gates. No desktop/runtime readiness claim is added.
