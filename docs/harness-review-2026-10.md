# Harness review: memory and durable execution

The highest-value gaps were reusable project memory and worker recovery. A trace
file preserved a narrative but could not restore the worker's verification state.
The installer also assumed every optional runtime was the design MCP server.

The implementation adds a contained `agent-runtime` package through the existing
setup/update owner. Mem0 owns explicit memory and LangGraph owns durable worker
transitions. Rust keeps policy and effects. The worker's many mutable arguments
are consolidated into serializable execution state. Shared OCI identity checks
serve both packages. The desktop keeps its existing two connections.

There was no existing semantic memory peer to remove. The existing task database
and evidence traces serve different purposes and should remain. LangGraph's
long-term store, Agent Server and LangSmith hosting are unnecessary here; adding
them would duplicate Mem0 or introduce a second deployment/control plane.

| Priority | Remaining issue | Next improvement and reason |
| --- | --- | --- |
| 1 | Evidence traces and unresolved checkpoints can still accumulate | Explicit policy preview/apply, paginated logical export and SQLite compaction now exist, including interrupted-apply tests. Restorable durable-state backups now cover history, checkpoints and traces; separately authorized trace archival remains; never prune unresolved effects automatically. |
| 1 | Generic external effects cannot be safely resumed merely from intent | Exact file writes now reconcile saved before/after digests without replay, retaining readback and host checks. Add other tool-specific recovery only where authoritative receipts establish the current state. |
| 2 | Retrieval quality is not yet established on real project distributions | A frozen public synthetic baseline now includes four languages, contradictions, stale/expired facts, distractors and instruction-like data. Next use consented representative projects and answer-generation evaluations before tuning ranking. |
| 2 | Mem0's dependency surface is large for explicit-fact use | An offline installed Python SBOM/size audit and fixture RSS measurements now exist. Track vulnerability changes explicitly and seek upstream optional dependency splits; never remove dependencies from the lock without testing the real path. |
| 2 | Desktop activation and server health are distinct | Retain main/package identity calls and report each client's observed state separately. Do not interpret a new installed binary as a restarted client. |
| 2 | Desktop plugins outside Mirket can add duplicate connections and unpinned launchers | Audit enabled client plugins separately for duplicate capabilities, stale launchers and distinct useful behavior. Pin or disable them through their owning plugin; do not modify synchronized cache files or infer that an unrelated connector is redundant. |
| 3 | Installer/UI/worker guidance still have some unavoidable overlapping descriptions | Keep mechanics in existing references and metadata in registries; use behavior tests to find actual contradictions before adding more profiles or an instruction generator. |
| 3 | The production package is Linux ARM64 only | Add another architecture only with native image, wheel/model and lifecycle verification. Do not imply support from Python portability alone. |

Mem0 is used as [the OSS library](https://github.com/mem0ai/mem0), with explicit
facts and local adapters. Hosted-product performance claims are not assumed to
apply to this deployment. LangGraph's [persistence model](https://docs.langchain.com/oss/python/langgraph/persistence)
provides durable checkpoints; it does not establish that an external effect or a
human acceptance decision occurred. See [the runtime contract](agent-runtime.md)
for commands, containment and recovery limits.

## Measured follow-up (2026-10-19)

The contained retrieval baseline used 22 synthetic facts and seven queries. The
expected fact ranked first in all six answerable cases across English, Turkish,
German, Spanish, current/older facts and instruction-like data. The unanswerable
query returned no facts. Several queries also retrieved irrelevant lower-ranked
facts, and both old/current color facts remained visible. This is evidence for
these fixtures, not general relevance, contradiction resolution or injection safety.

The pinned runtime audit found 98 installed Python distributions (including pip),
with 453,755,471 bytes recorded in their file manifests. One contained fixture run
peaked at 837,728 KiB in a child process; this excludes the whole VM and is not a
steady-state workload benchmark. The explicit host OSV query for the 97 locked
Python package versions returned no reported advisories on this date. Base OS,
models, unpublished vulnerabilities and runtime exploitability were not covered.
No dependency versions or retrieval thresholds were changed to improve these figures.

Concurrent desktop calls exposed two runtime issues: overlapping rootless OCI
starts could fail at the `devpts` mount before Python ran, and eight-call bursts
could exhaust the project store's ten-second lock wait. Version 1.0.8 queues
startup across processes until identity is verified and permits a bounded
60-second store wait within the existing 120-second response deadline. Running
sessions remain independent; operations are not automatically replayed.

On macOS ARM64 with the contained Linux ARM64 runtime, 88 candidate and 32
installed-build calls passed across shared connections, four client processes,
two project scopes and bursts of eight. Checks asserted retrieved fact identity,
complete export contents and retention previews. The contained suite also covers
a held lock beyond ten seconds and deadline exhaustion. These are bounded
synthetic regression results, not unlimited-concurrency or performance claims.
The [runtime contract](agent-runtime.md) describes staging a selected candidate
package for verification before desktop update.


## Reliability follow-up (1.0.9)

Verification now binds declared source/configuration/test inputs and produced
artifacts before host checks, retains immutable attempts and reconciles stale
status. Source and UI completion share the host-check contract. Project-scoped
design processes and browser routes replace worker access to the shared root;
Penpot object operations stay with the host until object-level scope is available.
Separate control/runtime/execution admission and a two-worker/one-operation
cross-process limit bound agent sessions. Lifecycle traversal uses cursor pages
rather than full rescans. Explicit fact supersession preserves history while
hiding replaced decisions, and optional recall no longer blocks graph startup.

Remaining priorities, in order:

1. **Recovery coverage:** consistent durable-state archives and explicit same-filesystem
   directory relocation now exist. Next consider cross-filesystem migration and
   project/design asset backup coordination; neither is covered by the current
   durable-state archive. Restore deliberately refuses live-store replacement.
2. **Budget accounting:** separate actively consumed execution budget from time
   awaiting the host, with conservative charging for uncertain in-flight work.
   Current deadlines intentionally still expire across downtime; reset-on-resume
   would allow an unbounded budget. Job-size estimates need measured workloads.
3. **Retrieval/routing evidence:** representative multilingual and adversarial
   project evaluations, active filtering before search candidate limits, and
   observed token/latency/cost data before changing model defaults. Synthetic
   recall results do not justify quality or cost claims for real projects.
4. **Guidance discoverability:** group the granular category vocabulary under a
   small set of navigable domains, preserving search aliases. Consolidate only
   guidance shown to conflict; portable foundation delivery remains intentional.

No new production dependency, storage engine, desktop connection or inference
provider was introduced for this follow-up. Existing task state, Mem0 facts,
LangGraph checkpoints and evidence traces retain their distinct owners.

## Desktop contract follow-up (1.0.11)

Lifecycle cursors and retention plans now have explicit object schemas. Direct
design tool identity is readable in ordinary output as well as MCP metadata.
Recall expands past inactive candidates within a fixed ceiling and discloses
incomplete results. The explicit `mirket dev acceptance` release gate combines
contained runtime regression tests, fresh desktop protocol contracts, pagination,
parallel calls, a scoped design artifact and restorable backup fixtures. See
CONTRIBUTING for prerequisites and actual-client activation limits. Portability,
active-time budget accounting and representative quality evaluation remain next.
