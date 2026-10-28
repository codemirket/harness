# Native coordination follow-up — 2026-10-28

Following the user's authorization to continue after cleanup `b109867`, three
new frozen diagnostic cases ran with the existing Astra/high configuration.
The earlier D5 cohort, its failures and its exhausted budget remain unchanged.
This pass adds no Mirket agent controller or production runtime dependency.

## What the old failure establishes

The original sum child had a persisted turn and an actual `task_started` event
at `00:19:13.350Z`. Its turn was interrupted at `00:22:53.117Z`, with no visible
items or command execution in its own retained turn. Its sole token-count
event had `info: null`. The working count child did execute and save its file.
This proves an asymmetric lack of observed child progress, not a particular
host scheduling or model-stream cause. Inherited history includes earlier
parent events, so their presence cannot establish child execution.

The runner now distinguishes unobserved child history, a recorded turn with no
items, a turn with non-command items, and an observed command. Terminal parent
status still cannot substitute for the required output oracle. Neither current
successful probe diagnoses or erases the earlier stall.

## Actual results

| Frozen case | Observed outcome |
| --- | --- |
| One mechanical child | Passed in 47.988 seconds. Child wrote `sum.txt = 12`; lead independently read it and wrote `result.json` with integer sum 12. |
| Two disjoint mechanical children | Passed in 54.964 seconds. Children separately wrote sum 12 and count 4; lead verified and integrated both. No output was reassigned or fabricated after a missing child. |
| Interruption plus owner-command termination | Passed the independent effect review in 49.322 seconds. Turn interruption left the delayed Python command listed. The test client selected its exact item/process/cwd, terminated it through Codex, observed exit `-1` and an empty terminal inventory, then checked beyond the original 30-second write deadline. `started.txt` remained; `late.txt` never appeared. |

The interruption was requested at `09:08:45.677Z`; the turn became interrupted
at `09:08:45.681Z`; the command ended at `09:08:45.836Z`. The host terminal
inventory exposed process `90674`, item
`exec-11a655e8-d432-4644-bbd9-8bff98fd4a9e`, under the disposable project.
No guessed PID, broad process-name termination or different thread was used.
An independent reviewer checked native child commands, saved integration,
scope preservation and the cancellation trace.

Post-trial review also tightened the automatic result classification: correct
files and child creation now require independent ownership/integration review
for both mechanical cases. They cannot alone earn an output-oracle pass. The
retained raw trial remains unchanged; its independent trace review supplies
the ownership evidence stated here. No new model turn was run for this change.

The [official Codex app-server documentation](https://learn.chatgpt.com/docs/app-server)
documents separate thread interruption and background-terminal list/terminate
operations. This test used the protocol generated from the actual selected
binary. The demonstrated recovery remains a Codex host operation, not an
automatic behavior guaranteed by Mirket guidance or every Desktop surface.

## Budget and verification

One 900-second wall window, three root turns, at most 240 seconds per case
including children, zero repairs and zero retries. Actual total wall time was
**152.288 seconds**, with three native children across the two mechanical cases.
Wall time includes overlapping work; it is not aggregate billable model time.

Host: macOS 26.5.2 ARM64, private Node 24.22.1 and Desktop-bundled
`codex-cli 0.164.0-alpha.11.1`. The CLI hash remains
`a04752e4d46a4d89e53e6f61c59c5349a638c8ce172ffbf391fd482b4d902a4e`.
Disposable homes used automatic approval review. Authentication copies were
removed after each trial. All source inputs and output scopes were preserved.

The revised [runner](../tests/codex-coordination-trials.mjs) has
[12 passing regressions](../tests/test_codex_coordination_trials.mjs), including
wrong command identity, surviving commands, successful exit despite a stop
receipt, incomplete inventory, delayed effects and missing lead integration.
No native skill, catalog or Rust/adapter runtime input changed in this follow-up.
The cleanup's runtime gates remain preceding evidence, not newly claimed runs.

Frozen runner/guidance, generated protocol, native histories, result artifacts
and review evidence are retained in
`/Users/nazmiyilmaz/.codex/task-state/harness-coordination-followup-20261028/`.
The tests do not establish reliable scheduling under all loads, provider-side
model identity, native spawn enforcement, Windows or Desktop GUI support.
The original mechanical stall's cause remains unresolved.
