# Recovery and evaluation follow-up — 20 October 2026

Subsequent work is recorded in [Harness completion evidence](harness-completion-2026-10.md):
live native recovery, additional vision defects, candidate acceptance, local
activation, the quality pilot and remaining evidence limits. This report retains
the status and observations from the earlier batch.

This follows the [first implementation batch](harness-remediation-2026-10.md).
It addresses F08, F11 and F12, and adds a startup ordering fix and diagnostics for
F13. The original intermittent timeout's cause remains unproven. Changes are
local and uncommitted; nothing here establishes release acceptance or activation
of an existing desktop connection.

## Resume compatibility (F08)

[Execution](../src/router/execution.rs) now binds execution-relevant model
capabilities, tier, availability, provider configuration and native executable
bytes. Descriptions, display labels, classification explanations and pricing are
observations, so their changes alone do not reject recovery. Changes that alter
the discovered tier still reject it.

The selected model must remain compatible. Missing or incompatible unused
candidates are excluded from live failover eligibility with a trace receipt.
Saved indices stay unchanged, so repeated checkpoints cannot silently select a
different model. Compatible later candidates retain their original indices and
floors. Native executable hashing is shared across candidates for each provider.

When refreshed metadata omits previously established native vision support, a
fresh conformance probe is scoped to the selected model and its bound executable.
Another candidate cannot satisfy this check, and this temporary scope is not
written over the discovery cache. Source, project, tools, guidance, artifacts,
explicit model selection and uncertain-effect restrictions remain enforced.

Regressions cover a removed unused fallback, metadata/price churn, incompatible
selected capabilities, unavailable selected models, invalid indices, a later
fallback retaining index 2 and changed native executable bytes. Unknown vision
cannot pass the compatibility check until re-established. A paid/native-provider
resume was not exercised; these tests establish the deterministic contracts,
not live provider availability or perception quality.

## Startup ordering and diagnosis (F13)

[MCP startup](../src/mcp.rs) now finishes its handshake before launching ambient
model discovery. Native application hashing/probing no longer competes with
identity capture before initialization. Opt-in `MIRKET_STARTUP_TIMING=1` emits
payload-free JSON phase timings on stderr. The installed motion regression
captures child diagnostics, prints them on timeout and asserts phase ordering.
Its 15-second initialization and 45-second operation deadlines are unchanged.

The instrumented original concurrent scenario passed four tests in 36.95 seconds;
initialization reached the handshake at 3,912 ms, including 3,805 ms between
catalog completion and executable identity capture. That observation does not
establish why the historical run exceeded 15 seconds. [Baseline log](harness-recovery/startup-before.log).

Three optimized macOS ARM64 probes through the official MCP SDK measured
62.4–63.2 ms startup. Each exercised tool discovery, resource delivery and refusal
paths; each phase trace puts discovery after initialization. These are local
samples, not an SLA or a comparison under controlled machine load.
[Optimized samples](harness-recovery/optimized-startup.json).

## Evaluation pipeline and retained failures (F11)

The new [packet preflight](../evals/design-quality/packet_preflight.py) executes the
production collector, score-lock serializer, post-score merge, packet identity
verification and unchanged frozen primary analyzer. It carries 24 planned cells,
48 captured attempts and 32 locked ratings. Missing cells, overwritten failed
reports and replaced score locks are rejected. Contradictory judges and a failed
functional guard remain visible and prevent advancement.

`prepare.py freeze` runs this preflight before dispatch preparation and binds its
report and helper sources into the freeze. CI runs it too. `capture-attempt`
retains each original report verbatim in an exclusively created host record;
sealing fixes the complete capture inventory. Collection verifies original source
and capture identities. A failed report cannot silently become a successful one
at the same path. Every repair must use a new source path. The host must keep
captures outside worker write roots: these helpers do not provide a sandbox or
recover reports overwritten before capture.

The [preflight evidence](harness-recovery/evaluation-preflight.json) is synthetic,
with zero model executions. Existing packet/tile tests also passed (20 tests).
This closes the tooling round-trip gap; a fresh blinded campaign, representative
cases and independently inspected outputs are still required before claiming a
quality gain or expanding guidance based on one.

## Bounded known pauses and rejected-transition recovery (F12)

[LangGraph](https://github.com/codemirket/harness/blob/0293dce091771398520e6151a86906d0bb48f663/runtime/agent/workflow.py) can persist a known inactive pause only
at a worker boundary with no pending effect. Resumption credits that interval
once, up to 86,400 cumulative seconds across the run. Original active seconds,
turn limits and design repair counts remain intact. Resume activates the clock
before discovery or conformance inference. Crashes without a recorded pause,
pending effects and already expired active budgets receive no credit. Backward
clocks and exhausted pause allowance reject resumption. Older snapshots receive
no retrospective credit; compiled-generation checks still apply.

The actual contained test exposed another defect: validation inside a LangGraph
node occurs after its interrupt resume has been journaled. A rejected turn reset
could therefore poison a later valid retry. Validation now runs before
`graph.invoke`, with the node retaining its own checks. The regression rejects a
turn reset, checks the checkpoint identity and then successfully resumes the same
run. SQLite connections are explicitly closed after each operation.

The real SQLite regression also resumes after the original deadline using only
known pause credit, preserves idempotent retries, rejects cumulative pause
exhaustion, expires an unrecorded interruption, rejects pausing a pending effect
and ignores a forged snapshot deadline. Rust status checks and dependency-free
Python checks cover the same budget boundary. [Initial runtime failure](harness-recovery/initial-runtime-failure.log)
is preserved alongside the final contained result.

## Verification and limits

The final Rust gate passed **404 tests, zero failures, seven opt-in tests ignored**,
with formatting and Clippy passing. All six installed debug startup repetitions
passed (three concurrent motion targets and three isolated runs); initialization
was 4.04–4.54 seconds, with discovery following the handshake in every sample.
The corrected real offline agent suite passed. Five focused Python regressions,
20 existing packet/tile tests and the complete synthetic packet preflight passed.

The [verification manifest](harness-recovery/checks.json) binds final executable,
source and component evidence, including the full Rust gate, six repeated debug
startup scenarios, real offline agent runtime, Python checks and disposable
installation checks. See [component identities](harness-recovery/identity.json).

No required timeout, threshold or release gate was lowered. The first Clippy run
rejected a large-error diagnostic closure; it was replaced with a match, without
suppressing the lint. A later [documentation-link gate failure](harness-recovery/pre-manifest-gate-failure.log)
caught this report referring to its verification manifest before that file was
saved; the manifest now exists and records verification status explicitly. The
failed runtime test led to the prevalidation fix above.
An initial local installation evidence script omitted `--json`; setup succeeded,
but the script could not parse its text output. The corrected script repeated the
checks in a fresh disposable home.

Local native validation covers macOS ARM64; contained runtime validation covers
Linux ARM64. Native Windows/Linux CI, actual desktop reconnection, paid native
vision resumption and a complete candidate release-acceptance receipt remain
unverified. Existing JS dependency/runtime sources were unchanged by this batch;
the prior 81-test evidence is retained without claiming a new run. The remaining
historical startup uncertainty and lack of measured quality benefit remain open.
