# Initial release: installation and task verification

Harness **1.0.0** is installed for Codex Desktop and Claude Desktop Code on the
user's Mac. This verification was performed against an uncommitted local release
candidate before publication was authorized. The [source review](../harness-2026-10-10/README.md) remains the pinned
review of the 40 repositories and two documentation services requested earlier.

## Installation and current contract

- One schema 1 project contract and the target names `codex`, `claude`, `all`.
  Removed the old installer module/command, target aliases, schema upgrade paths,
  embedded adapter fallback, obsolete Claude instruction wrapper and scheduler
  migration recognition. Role aliases such as CFO remain useful capability names.
- Setup and maintenance use the same installation implementation. Installation
  owns guidance, 14 shared skills and enabled MCP definitions. It preserves host
  model, approval, sandbox, theme and other unrelated settings.
- Current receipts require ID, repository, commit, source path, source hash,
  installed hash and executable-file contract. Missing provenance cannot be
  accepted as an unchanged installation. Managed-copy conflicts remain visible.
- Configuration publication rechecks bytes and guarded parent paths before
  replacement. Regression cases cover concurrent edits and parent substitution;
  these checks do not constitute an operating-system sandbox.
- Runtime reporting distinguishes Codex Desktop, Claude Desktop, Codex CLI and
  Claude CLI. A CLI login does not prove desktop presence, quota or task activation.
  Evaluation fingerprints now include target, MCP and explicit preference inputs.

Codex Desktop 26.1007.21159 was present. Its bundled CLI is
0.162.0-alpha.17.2. Claude Desktop 2.31226.1 was installed from Anthropic's official
download, with SHA-256, Apple signature and Gatekeeper checks. Claude CLI 2.1.296
was already authenticated. Model and permission defaults were retained.

The final live installation check reports both targets ready with no pending
changes. OpenAI Docs MCP is enabled and was actually invoked by both native
engines. Microsoft Learn remains defined and disabled. Optional maintenance
remains disabled; no schedule was created or changed.

See [installation](installation-final.json), [runtime probes](runtime-final.json),
[isolated installation exercises](isolated-installation.json) and
[preference preservation](preference-preservation.json). App metadata readiness
and CLI authentication in those reports are separate from the task evidence below.

## What the task trials establish

Substantive prompts did not name any skill. They requested executable financial
analysis or Turkish resource files from fictional inputs. Traces record real
tool calls and reads; merely mentioning a skill was insufficient. Source and
artifact hashes, native outcomes and independent checks are retained per trial.

| Trial | Observed activation | Artifact evidence |
| --- | --- | --- |
| Codex finance, app-bundled CLI | Read `skill-catalog`, `financial-analysis`, its cash reference and engineering guidance | Eight independent checks, including changed inputs, sparse periods, year rollover, conflicting IDs and saved-value reconciliation |
| Codex localization, app-bundled CLI | Read catalog, `localization` and message-integrity reference | Eight independent contract checks and semantic agent review |
| Initial Claude finance CLI and Desktop localization | No catalog/specialist invocation or reads | Translation passed; finance failed the newly added sparse-period probe. These runs fail activation acceptance |
| Fresh Claude finance CLI after routing repair | Invoked catalog; read finance body and reference | Selection repaired, but sparse-period rejection and timing qualifications still needed correction |
| Fresh Claude Desktop Code localization | Invoked catalog; read localization body and reference | Eight independent checks, qualitative review, actual FormatJS parsing/rendering and corrected failure probes |
| Claude finance repair after review | Invoked catalog; read current finance and testing guidance | Eight independent checks and 21 candidate tests pass; integration review clarified two timing sentences and labeled synthetic inputs, preserving the native original |
| One-sentence translation, both engines | No tools or skills invoked | Correct concise Turkish response; the router does not add overhead to this trivial request |
| OpenAI Docs MCP, both engines | Native search and fetch calls completed | Returned the documented personal-skill directory and official source link |

The Claude routing repair made the catalog description cover finance and
translation, and placed invocation/reading before substantive execution in shared
guidance. Discovery diagnostics confirmed that the original global instruction
file already loaded. This is an observed before/after repair, not proof that a
single wording change caused a measured quality improvement.

Finance review exposed a second, independent gap: a model can load guidance and
still add an invalid input restriction or overstate what monthly cash proves.
The verifier now exercises a sparse reporting calendar. Finance guidance explicitly
respects the declared calendar and qualifies funding amounts/deadlines by timing
resolution. Earlier failing artifacts remain retained instead of being rewritten
as successful runs.

The interface trial selected the design guidance and produced a working local
capability explorer. Its CLI process timed out after a sandboxed browser launch
failed. A separate Codex Desktop host-agent continuation completed real Chrome
inspection at 1440, 390 and 320 pixels, including keyboard selection, search,
empty-state recovery, long labels and reduced motion. The original CLI run remains
incomplete. See [the browser QA](trials/codex-interface/QA.md).

## Verification and scope

All **593 tests passed in 55.337 seconds**, with no skips. See the [verification record](verification.json). Context doctor, capability
integrity, generated documentation, POSIX shell syntax and whitespace checks passed.
Isolated copy and linked installations passed repeated installation/checks, and
both-target specialist registration passed sync/doctor/repeat sync. The core CLI
uses Python 3.9.6 with no added production dependency. The optional skill-creator
validator could not run because PyYAML is absent; repository frontmatter,
description, reference and catalog integrity checks passed in the full suite.

These are public development cases and review-driven repairs, not a blind benchmark
or proof of all 24 roles. The Desktop rerun used a fresh session and a new child
directory; the earlier translation remained accessible in its parent. Codex native
generation used the executable bundled with Desktop; a separate new Codex sidebar
chat was not created. The host-agent browser continuation is identified separately.
No native Windows/Linux client, physical mobile device, screen reader, native-human
translation review or production application release was tested. Browser captures
were inspected inline; the documented browser API did not save durable pixel files.

The final release fingerprint is in [source identity](source-identity.json), with
[phase identities](source-phases.json) and a supplemental [source inventory](release-tree.json).
[Trial summaries](trial-summary.json), per-trial tool traces and retained artifacts
show both failures and repairs. Tool traces omit hidden reasoning, account/session
metadata and full retrieved-document responses; paths are normalized and truncated
inputs are labeled. Earlier review records apply only to their original bytes. The [independent review](review-routed.md) records final artifact acceptance and the integration edits separately.

## Integration and cleanup

All source and retained evidence were integrated into the original checkout and
compared byte for byte, including file modes. The final installed check remained
ready for both clients with no pending changes.

Task scratch, installer downloads, disposable CLI project state, temporary browser
servers and the duplicate checkout were removed. Twenty-two obsolete generated
export/install directories were also removed after checking client references;
their 20 build receipts were retained as historical provenance. Historical review
inputs and unrelated visual work remain. Native client session history remains
under client ownership. See [cleanup details](cleanup.json).
