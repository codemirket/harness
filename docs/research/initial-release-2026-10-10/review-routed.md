# Independent review of routed and repaired native task artifacts

Reviewed 2026-10-10 by a separate agent. This is agent acceptance of bounded
synthetic development exercises, not human approval, financial authority,
native-speaker sign-off, a production acceptance test or a security attestation.
No model calls, delegation, dependency installation, settings changes or artifact
repairs were performed by this reviewer. The only newly retained review artifact
is this file. Earlier first-run findings remain in `review.md`.

The raw transcript locations below identify original inspection sources. Exported raw streams are task scratch; only redacted tool traces are retained in the repository. Native client history remains under client ownership. The per-trial `source_line` fields preserve the cited line numbers; retained directories are mapped below. Repository
verifiers were executed from
`<verification-checkout>`.

## Outcomes, kept separate by run

| Run | Independent current finance/localization checks | Agent assessment | Observed harness use |
| --- | --- | --- | --- |
| Codex initial finance | 8/8, exit 0 | Earlier accepted monthly decision remains accepted | Successful reads documented in `review.md` |
| Claude initial finance | 7/8, exit 1; sparse horizon fails | Not accepted for the full input contract; earlier decision defects retained | No relevant skill/body reads in initial trace |
| Claude routed finance | 7/8, exit 1; sparse horizon fails | Not accepted: activation alone did not fix contract or funding wording | Catalog invoked; financial body and reference read |
| Claude Desktop localization recheck | 8/8, exit 0 | Accepted for this six-string source/target contract and agent semantic review | Catalog invoked; localization body received; reference read on retry |
| Claude finance repair | 8/8, exit 0 | Code and bounded decision accepted after the three integration prose corrections documented below; no remaining artifact defect found | Catalog invoked; current financial body and full cash reference received |

The finance repair's TASK includes the specific independent-review defects. It
is a review-driven repair, not evidence of first-attempt success on an unseen
case. Rechecking earlier artifacts with the enhanced verifier is a replay, not
a new model trial. The original failed artifacts remain intact.

The two finance Claude CLI traces identify `claude-opus-5-5`, Claude Code
`2.1.296`, permission mode `auto`. The Desktop localization messages also identify
`claude-opus-5-5`. Actual model calls and client setup were the parent agent's
work; this reviewer inspected their stored outputs. Native finance repair run
metadata records completed, exit 0, 535.315 seconds.

## The added discriminating finance check

`evaluations/cases/finance-cash-decision/verify.py` now exercises an input with
`periods = ["2026-01", "2026-03"]`, leaving the ledger records otherwise unchanged.
The contract defines the horizon by its listed periods; it never requires
consecutive periods. A valid implementation must accept the list, exclude
February recognition/cash flows, retain calendar-month delay semantics and emit
only January and March results. The check executes the candidate CLI and compares
all values and types against the unchanged independent oracle.

The focused regression in `tests/test_redesign_evidence.py` pins independent
hand controls: both scenarios have EUR 30,000 then EUR 50,000 closing cash,
EUR 30,000 minimum cash and zero funding to the reserve. This is a behavioral
probe, not a check for prose, source strings or a particular implementation.
The task contract, fixture and `expected()` oracle logic were not rewritten.

Fresh replays confirmed Codex 8/8, initial Claude 7/8 and routed Claude 7/8.
Only `sparse-horizon` fails in the two Claude pre-repair versions. The final
repair passes all eight checks. Its arithmetic/code/results were unchanged by
the final prose-only integration edit, so that edit did not warrant another
arithmetic rerun.

Focused checks: `python3 -m unittest discover -s tests -p test_redesign_evidence.py`
passed all 4 tests. `git diff --check --
evaluations/cases/finance-cash-decision/verify.py tests/test_redesign_evidence.py`
passed. Verifier subprocess outputs were isolated in auto-cleaned temporary
directories; saved candidate artifacts were not overwritten.

## Finance acceptance and integration provenance

For the original ledger, both saved scenarios independently reconcile:

- EUR 150,000 revenue less EUR 90,000 operating costs gives EUR 60,000 profit.
  The EUR 70,000 principal payment affects cash, not operating profit.
- Base closing cash is EUR 30,000 / -20,000 / 0. Delayed closing cash is
  EUR 30,000 / -70,000 / -50,000. Both first become negative in February.
- The EUR 10,000 reserve requires EUR 30,000 base or EUR 80,000 delayed
  funding at opening/month-end observations.
- Paying each month's obligations before its receipts produces base minima
  EUR 30,000 / -70,000 / -50,000 and delayed minima EUR 30,000 / -70,000 /
  -100,000. The corresponding adverse-order reserve requirements are EUR
  80,000 and EUR 110,000. These are bounds on the stated ledger, not a daily
  forecast or a guarantee that missing obligations have been covered.
- Deferring the whole principal beyond March leaves base month-ends EUR
  30,000 / 50,000 / 70,000 and delayed EUR 30,000 / 0 / 20,000. The respective
  adverse-order reserve requirements EUR 10,000 / 40,000 are correct.

The repair now accepts sparse horizons. Its recommendation sizes conditional
availability to EUR 110,000 while drawing only what dated evidence requires.
It states that funding must be usable before the payments it covers, discloses
missing exact dates and frames 31 January as a conservative bound against the
earliest possible February outflow. This is not a discovered lender due date.
No loan, early collection, deferral, capital injection or permission to act is
assumed. Weekly forecasting is explicitly proposed follow-up; the monthly
program can reconcile monthly totals but cannot verify weekly timing.

The native repair still had an ambiguity in two sensitivity-table sentences:
conditions needed to preserve the reserve throughout a month appeared attached
to month-end sufficiency. The parent made a narrow integration edit to the
EUR 30,000 and EUR 80,000 rows. I inspected the exact two-line diff. The final
rows correctly distinguish month-end sufficiency from within-month ordering.
The unchanged native report is preserved as `decision.native.md`, SHA-256
`458eddf4d2f87ec5ca9f8d9d5b73d5688b855327f8d06a3857b4df260069fe77`.
The intermediate report after those two timing edits had SHA-256
`008f2b057e50c2d354588a3211a5c86d8e277f0146340604a2675233a9728f57`.

A third integration prose edit adds this sentence under Source, basis and
reproduction: "This is a development exercise using synthetic inputs, not an
assessment of a real business." This closes the standalone report's omission
of the finance rubric's synthetic-input label. I verified the exact sentence
and its placement; removing only that added sentence and its blank line from
the final bytes reproduces the intermediate hash above.

The final accepted decision report has SHA-256
`beba5acbbab0d2ec86e90824f8b1d8f313971df62f7e161e2ade734f91159e9a`.
The code and results are unchanged. Existing arithmetic evidence remains valid,
so no additional native run or arithmetic recheck was needed. The final numbers,
authority limits, synthetic basis and recommendation satisfy this task's bounded
agent review, with no remaining artifact defect found. The original native
report, two timing-row corrections and separate synthetic-label addition remain
distinct stages; the three integration fixes are not native first-attempt output.

## Localization acceptance

Reviewed `trials/claude/localization/recheck/en.json`, `tr.json` and
`translation-qa.md` against the six-string task and localization rubric.

The translation preserves the supplier's refund confirmation as a prerequisite
for cancellation, keeps request distinct from confirmation, retains the
estimated EUR 1,250.50 amount and local-tax exclusion, and preserves the
10 October 2026 17:00 Europe/Istanbul deadline. The button remains request-only;
the preview remains pending and not confirmed. Booking/name placeholders,
`count`, the =0/one/other branches, active # substitutions and strong markup
are intact. Turkish wording and terminology are suitable for the stated
professional travel interface in this agent review.

The QA note accurately distinguishes formal body instructions from the bare
button imperative. It discloses that no native human or live UI review occurred.
The independent repository verifier passes 8/8. The native trace additionally
shows an actual formatjs parse/format exercise, using already-present
`intl-messageformat` 10.7.18 and `@formatjs/icu-messageformat-parser` 2.11.4
read-only. No dependency was installed. This is an ICU reference runtime,
not proof of the eventual application's own interpolation behavior.

The native checker initially missed an inserted BOM because TextDecoder
stripped it, then checked raw bytes and caught 27/27 deliberate corruptions.
The final successful tool output confirms source unchanged, no saved-target
failures and 27/27 probes caught. These are trace-observed candidate checks,
separate from the eight independently executed repository checks. UI wrapping,
font coverage, screen-reader output and native-speaker approval remain
unverified. General Java/TDK commentary in the QA note was not an independently
researched acceptance criterion for this fixture.

## Observed skill use: tool evidence, not reasoning text

`runs/claude/finance-routed/events.jsonl`:

- Line 10 invokes `Skill` with `skill-catalog`; line 12 returns its body.
- Line 18 returns the financial-analysis body in a successful tool result.
- Line 20 successfully returns the complete cash-and-earnings reference.

`runs/claude/finance-repair/events.jsonl`:

- Line 8 invokes `Skill` with `skill-catalog`; line 10 returns its body.
- Line 16 successfully returns the full current financial-analysis body.
- Line 18 contains the full current cash-and-earnings reference. That batched
  command is marked failed because later Git/directory inspections failed;
  the reference text itself was fully returned. This is received/read evidence,
  not a claim that the entire batch succeeded.

Claude Desktop localization recheck transcript:
`<home>/.claude/projects/-private-var-folders-rk-rdby38mj0l30jyvbk965q2lc0000gn-T-harness-initial-release-ctt4c72g-trials-claude-localization/8250db6f-f4a2-4075-8828-1484f546a4a9.jsonl`

- Line 44 invokes `Skill` with `skill-catalog`; line 47 returns its body.
- Lines 72/73 read and return the full localization body. The batch then fails
  at an unquoted separator, before the reference is read. It is not a fully
  successful batch.
- Lines 75/76 retry and successfully return the full message-integrity reference.
- Lines 119/120 execute the final ICU/meaning-marker checks and return success,
  with source unchanged, saved target failures none and 27/27 failure probes.

Full-body/reference matches were checked against actual returned text. Reasoning
blocks, skill listings and self-reported selection were not used as invocation
proof. The Desktop prompt names only recheck/TASK.md and says to complete the
task without delegation; it does not name the localization skill. The finance
repair prompt names defect outcomes, not skill names.

## Evidence limits and artifact identities

These single, public development cases do not prove performance across all role
capabilities, superiority over base models, an isolated causal effect of one
routing change, production financial fitness, human translation acceptance or
all client surfaces. First-run failures, successful routing, repair instructions
and integration corrections are separate facts.

The finance ledgers retain source SHA-256
`5be1e094f76fde050cd4c8a0c0648f2fe296009b04bfc10720c04e276a967eeb`.
The localization source retains SHA-256
`5ba07c8da6fe537ed935ead1773026047287b9b6a559480bfa0061c78da2ae82`.
Exact file hashes observed when writing this report follow.

```json
{
  "trials/codex/finance/analyze.py": "e6604ecd342030013db34a14ca4fbbbc23d3948690f6c48e57ae9db06844e1f7",
  "trials/codex/finance/results.json": "1b313f2fc27f00879c2a9cefad3d2803f7862177f988f7f1646373788956d677",
  "trials/codex/finance/decision.md": "8e6534db3c0e8fc565afa4bfeff0fa79a008a5cb60add096e96f3ded54549fdd",
  "trials/claude/finance/analyze.py": "c3e8ec5c209d3c4b65364c66dfc093d7652c2ca04a0700e3910b2fd899a2bc37",
  "trials/claude/finance/decision.md": "e4278f28dccc909ebc86d059b0d7acc197b13fe687c5974b7e8f811f0b05e8fc",
  "trials/claude/finance-routed/analyze.py": "ad0b9162a5227cb5e2f3a3a5008ce3cb3c6e35e8020631d6b8d0cc45c02c89bf",
  "trials/claude/finance-routed/decision.md": "6eafc5d5918ad87f24ea0559b7a7f7b688dbca03933beeb63deead39ded94f89",
  "trials/claude/finance-repair/TASK.md": "ad82de570a0172e3edca803e6b36bff80b47ad6dac6f19bc91217875c99a1e29",
  "trials/claude/finance-repair/analyze.py": "d4d765adbd97ccf0b0ebe4909c4e2cc3eaef7212268900795f9dca1fa6147fac",
  "trials/claude/finance-repair/results.json": "1b313f2fc27f00879c2a9cefad3d2803f7862177f988f7f1646373788956d677",
  "trials/claude/finance-repair/decision.native.md": "458eddf4d2f87ec5ca9f8d9d5b73d5688b855327f8d06a3857b4df260069fe77",
  "trials/claude/finance-repair/decision.md": "beba5acbbab0d2ec86e90824f8b1d8f313971df62f7e161e2ade734f91159e9a",
  "trials/claude/finance-repair/test_analyze.py": "80d12e59ceb7e12f5323e4bb73a4e8a5238d39df8660b3d84e917a2dd673733f",
  "trials/claude/localization/recheck/TASK.md": "44f10c0942be8ff9738237359cd08682ee50c3baa441dbd6158343975d8c279e",
  "trials/claude/localization/recheck/en.json": "5ba07c8da6fe537ed935ead1773026047287b9b6a559480bfa0061c78da2ae82",
  "trials/claude/localization/recheck/tr.json": "611c795cb951bc6edaee5b132d74462413db6be1a9e7899a842859019791e133",
  "trials/claude/localization/recheck/translation-qa.md": "bb2864e488dbf4079f374e5b83be2b7b9058eb06fae7c442a36439874d423844"
}
```

## Retained evidence map

| Original trial workspace | Retained directory |
| --- | --- |
| `trials/codex/finance` | [codex-finance](trials/codex-finance/run.json) |
| `trials/claude/finance` | [claude-finance-before](trials/claude-finance-before/run.json) |
| `trials/claude/finance-routed` | [claude-finance-routed](trials/claude-finance-routed/run.json) |
| `trials/claude/finance-repair` | [claude-finance-repaired](trials/claude-finance-repaired/run.json) |
| `trials/claude/localization/recheck` | [claude-desktop-localization-routed](trials/claude-desktop-localization-routed/run.json) |

The earlier review is retained as [review-before.md](review-before.md). Original raw-stream hashes are in each run record.
