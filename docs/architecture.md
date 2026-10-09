# Harness architecture

Harness maintains reusable agent guidance and makes installation state inspectable.
The core command line uses Python 3.9+ and the standard library. It selects reviewed
payloads, preserves their provenance, reconciles declared files and reports drift.
Projects retain their own architecture, verification commands and release authority.

Start with [adoption](adoption.md) to use it or [CONTRIBUTING](../CONTRIBUTING.md)
to change it. This map describes the current boundaries; it does not prescribe an
architecture for projects that adopt the harness.

## Command owners

[ai.py](../ai.py) dispatches commands to focused modules:

| Area | Owning source | Contract and detailed guide |
| --- | --- | --- |
| Client installation | [target_install.py](../lib/target_install.py), [targets.py](../lib/targets.py), [configuration.py](../lib/configuration.py) | Explicit target selection and selected configuration merging; [targets](targets.md) |
| Global and project reconciliation | [harness.py](../lib/harness.py) | Plans, preserved modifications, managed copies and project locks; [project targets](project-targets.md) |
| Reviewed skill catalog | [catalog.py](../lib/catalog.py) | Pinned sources, adaptations, hashes, companions and executable contracts; [source review](source-review.md) |
| Task capability contracts | [capabilities.py](../lib/capabilities.py) | Discoverable deliverables, prerequisites and failure probes; [capabilities](capabilities.md) |
| Runtime and preferences | [runtime.py](../lib/runtime.py), [settings.py](../lib/settings.py) | Readiness and explicitly managed preferences; [settings](settings.md) |
| Project instruction audit | [context.py](../lib/context.py) | Explicit root-to-cwd selection, shadowing and byte-budget diagnostics; [context](context.md) |
| Scheduling and maintenance | [schedule.py](../lib/schedule.py), [maintenance.py](../lib/maintenance.py) | Explicit schedule registration and bounded checkout maintenance; [scheduling](scheduling.md) |
| Artifact workbench | [workbench.py](../lib/workbench.py), [scripts/workbench/](../scripts/workbench/) | Separate browser, vector, document and Markdown operations; [workbench](workbench.md) |
| Development evaluation | [evaluation.py](../lib/evaluation.py), [evaluations/](../evaluations/) | Fixture preparation, executed checks and artifact-bound reviews; [delivery quality](quality-harness.md) |
| Export and handoff | [bundle.py](../lib/bundle.py), [handoff.py](../lib/handoff.py) | Local packages with provenance; [plugins](plugins.md) and [target coverage](targets.md) |
| Bounded Claude delegation | [claude_delegate.py](../lib/claude_delegate.py) | Explicit worker/tool limits; [coordination workflow](../skills/agent-coordination/SKILL.md) |

[install.py](../lib/install.py) retains the older combined installation path behind
`legacy-install`; current `install` and `check` dispatch to `target_install.py`.
Check the actual command's `--help` before modifying a compatibility path.
Exported `_harness` snapshots contain the runtime and reference documentation,
including this map, but omit the contributor test suite. Use the full source
checkout for the contribution gates below.

## Authoritative inputs and derived state

- [registry/harness.json](../registry/harness.json) selects global skills, project
  defaults and export bundles. [registry/targets.json](../registry/targets.json)
  defines client adapters; [registry/mcp.json](../registry/mcp.json) holds MCP definitions.
- [registry/catalog.json](../registry/catalog.json) owns reviewed entries, source
  pins, payload contracts and profiles. Authored payloads live in [skills/](../skills/).
  [instructions/AGENTS.md](../instructions/AGENTS.md) is shared installed guidance;
  the root [AGENTS.md](../AGENTS.md) governs contributors to this repository.
- A consuming project's `.ai/project.json` declares its selections. Its generated
  `.ai/project.lock.json` records resolved provenance and target assignments. Use
  project commands to reconcile it; do not hand-edit a lock to conceal drift.
- [scripts/render_registry.py](../scripts/render_registry.py) generates
  [catalog documentation](catalog.md) and [source review tables](source-review.md).
  Change authoritative inputs first, regenerate, then run `--check`.
- `build/` contains ignored generated artifacts and scratch evidence. Dated
  [verification records](verification.md) describe their original inputs and limits;
  they do not certify later edits.

## State changes and trust boundaries

`plan`, `check`, `doctor` and dry-run commands expose their documented state without
installing it. Installation and synchronization write selected destinations;
project `init` and `add` update declarations, while project `sync` reconciles copies
and the lock. Keep optional probes separate: authentication checks, local browser
captures, evaluation execution and workbench rendering have their own effects.

Known installer preflight conflicts block writes; recovery is scoped to individual
operations, not a transaction across every client and file. Modified or unselected
copies are preserved for deliberate resolution. POSIX executable contracts and
source/adaptation hashes are part of integrity, not permission to execute helpers.

Registration does not install runtime dependencies, connect accounts or prove that
a client loaded a skill. Likewise, worktrees separate checkouts but do not by
themselves isolate services, databases, networks or credentials. Native client
permissions remain distinct from instructions and local validation.

The evaluator runs local verifier and candidate code under host permissions. Its
input fingerprints and review records detect relevant drift; they are not a
security sandbox or proof of human approval. Keep automated results, inspected
artifacts, client activation and production outcomes as separate evidence.

## Verification and change flow

Locate the owner above, inspect the relevant existing tests, and exercise the
changed consumer behavior in an isolated fixture. Follow the concrete commands in
[CONTRIBUTING](../CONTRIBUTING.md) and the root [project map](../AGENTS.md). Extend
tests for meaningful failures and preservation guarantees, not for exact prose.
For a changed workflow, inspect a representative result as well as its registration.
Record actual checks and remaining limits without upgrading historical evidence.
