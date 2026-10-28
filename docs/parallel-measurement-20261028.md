# R9 measured parallelism — 2026-10-28

Later evidence: [supplemental evaluation](supplemental-model-evaluation-20261028.md).
This record preserves its original cohort; the later report supplies current
results and remaining qualification limits.

This is a new, bounded measurement cohort. It does not reopen the closed D5 qualification or the existing follow-up cases.

## Invocation ledger

Two shell invocations are retained in the task-state record, but only one reached native model execution:

1. `/Users/nazmiyilmaz/.codex/task-state/harness-parallel-measurement-20261028/` was an invalid preflight attempt. The outer shell created that directory before invoking the runner; the runner intentionally uses exclusive directory creation and stopped with `EEXIST` before writing `frozen.json`, starting app-server, initializing a thread or making any model turn. Its directory is empty. No model budget, worker, retry or repair was consumed.
2. `/Users/nazmiyilmaz/.codex/task-state/harness-parallel-measurement-20261028-run2/` is the one valid executed cohort. It ran the two frozen cases once each, sequentially, with zero retries, repairs, replacement workers or new specimens. The `run2` suffix records the setup correction; it is not a second native trial or a hidden repeat.

The measured comparator is **one native worker versus two native workers on the same task**, not a no-delegate baseline. A separate no-delegate case already exists in the closed D5 cohort and is not mixed into these timings.

## Frozen cohort

The runner added a separate measure suite to tests/codex-coordination-trials.mjs. The task is identical in both cases:

- Disposable project contains only input.json with [3,3,8,-2] and the coordination skill copied as AGENTS.md.
- Required final artifacts are sum.txt (12), count.txt (4) and lead-owned result.json ({"sum":12,"count":4}).
- Serial arm: exactly one native worker owns both sum.txt and count.txt; the lead verifies actual files and writes result.json.
- Parallel arm: exactly two native workers own disjoint output files; the lead verifies both and writes result.json.
- Same lead model and effort: requested gpt-6-astra / high; same 240-second per-case deadline; one 600-second suite wall budget; zero repairs, retries, replacement workers or new specimens.
- The lead owns integration in both arms. A successful turn is not accepted without output, scope and integration checks.
- Native host, agent lifecycle and permissions remain Codex-owned. The measurement does not claim provider identity from model metadata.

Output root:

/Users/nazmiyilmaz/.codex/task-state/harness-parallel-measurement-20261028-run2/

Host:

- macOS Darwin ARM64
- Codex CLI 0.164.0-alpha.11.1
- Binary: /Applications/ChatGPT.app/Contents/Resources/codex-cli/CodexCLI.app/Contents/MacOS/codex
- Binary SHA-256: a04752e4d46a4d89e53e6f61c59c5349a638c8ce172ffbf391fd482b4d902a4e
- Normal native turns used the previously qualified disposable-auth setup and automatic review path; no live Mirket home or project was used.
- Cohort runner SHA-256: ebc25d411cdd5154472c8069c5e5522cc3fa5be72e16dccbc6c957db32b7c275
- Frozen coordination skill SHA-256: 6f412decd1642afc72edcd460e9f3d9131f6e347497e9a2bdbe56d82886508ea

## Results

| Arm | Native children | Elapsed | Aggregate tokens | Input | Cached input | Output | Reasoning output | Lead commands | Lead output commands | Artifact oracle | Scope |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- | --- |
| Serial | 1 | 57.305 s | 156,713 | 155,708 | 146,048 | 1,005 | 43 | 2 | 2 | pass | pass |
| Parallel | 2 | 79.042 s | 291,698 | 289,809 | 267,904 | 1,889 | 70 | 3 | 1 | pass | pass |

Suite wall time was 136.359 seconds. Both turns completed normally, both produced the exact expected artifacts, and both preserved all supplied files. The parallel arm produced two resolved native children with command-observed child turns; the serial arm produced one.

The token totals are the latest cumulative per-thread thread/tokenUsage/updated totals summed across the lead and resolved children; intermediate usage events are not added repeatedly. Cached input is reported separately as a subset of input tokens, not added again to totalTokens. They are host-reported usage evidence, not a billing statement. The runner retains per-thread totals in each case's result.json.

The lead-command counters are trace-derived integration effort indicators:

- Serial lead: two commands touching output paths, including the final result.json write.
- Parallel lead: three commands total and one output-path command, including the final result.json write. The extra lead command is a trace-level coordination/inspection cost; command count is not a quality score.

The retained traces establish actual ownership for this cohort:

- Serial child `01a182ec-e1f9-74a1-88d5-38eabc7dff99` wrote both sum.txt and count.txt; the lead `01a182ec-8b8f-7ea3-b87c-24b2951b63b1` verified them and wrote result.json.
- Parallel child `01a182ed-e568-71e3-878f-a1dc4d4737a2` wrote only sum.txt; child `01a182ed-fedc-7a51-87b1-1cd0197806ef` wrote only count.txt; the lead `01a182ed-6b4f-7790-ba6a-3561c411a882` verified both and wrote result.json. Neither child wrote the other's file or result.json.

These are command-trace ownership observations plus final artifact checks. They do not authenticate provider identity.

## Measured conclusion

This cohort does not demonstrate a speedup from two native workers on this small, mechanical task:

- Parallel elapsed time was 79.042 seconds versus 57.305 seconds serial: 21.737 seconds slower, or approximately 37.9% slower.
- Parallel aggregate tokens were 291,698 versus 156,713 serial: 134,985 more, or approximately 86.1% higher.
- Both arms passed the deterministic artifact and scope oracle. Therefore the observed quality outcome is tied, while parallelism is worse on elapsed time and token cost in this cohort.
- The two-worker arm had one additional lead command overall, while the serial arm had one more lead output-path command. This is descriptive trace evidence, not a claim that command count alone explains the slower run.
- No repair or retry was allowed, so this is not a best-of-k comparison.

The correct product conclusion is to retain the existing gate: parallel delegation needs a demonstrated independence/context benefit, sufficient verifier capacity, and an explicit measurement. Do not advertise generic parallel speedups from this result.

## Verification and limits

The deterministic checks were:

- expected native child count (one versus two);
- completed turn status;
- no runner error or timeout;
- exact sum.txt and count.txt contents;
- parsed integer result.json with sum=12 and count=4;
- no out-of-scope project changes;
- lead-owned integration observed in the native trace.

The runner labels both arms requires-independent-review because native traces and model self-report do not, by themselves, establish every ownership/provenance judgment. The artifact oracle and trace review establish that this cohort completed the measured mechanical outcome; they do not prove provider identity or generalize to creative work, larger tasks, different model settings, different hosts, or a different release.

Not measured here:

- visual/motion quality;
- Windows, Desktop GUI, or other OS floors;
- worktree setup contention;
- process cancellation;
- long-running or I/O-heavy tasks;
- model-provider identity beyond host-resolved metadata;
- human review time outside the lead command trace;
- cost in currency.

The exact raw protocol, frozen runner, skill copy, per-case project inventories and per-thread usage remain under the output root. The runner's new tests cover same-task equivalence, child-count discrimination and token aggregation.
