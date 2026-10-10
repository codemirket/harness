# Independent review of native-client task artifacts

Reviewed 2026-10-10. This is a separate agent review, not human approval,
financial authorization, native-speaker sign-off or a security attestation.
The tasks use synthetic financial data and a fictional localization fixture.
No model calls, external messages, dependencies, source changes or artifact
repairs were made during this review. The only retained review output is this file.

## Evidence and outcome

| Native run | Independent checks | Qualitative result | Harness selection evidence |
| --- | --- | --- | --- |
| Codex bundled CLI, finance | 7/7 passed | Accepted for the specified monthly decision; no blocking finding | Successful catalog, financial-analysis, engineering-judgment and reference reads |
| Codex bundled CLI, localization | 8/8 passed | Core meaning and tokens preserved; no blocking finding | Successful catalog, localization and message-integrity reads |
| Claude CLI, finance | 7/7 passed | Correct provided fixture, with the extra contract failure and decision wording concerns below | No Skill calls or successful harness body/reference reads in the inspected trace |
| Claude Desktop Code, localization | 8/8 passed | Core meaning and tokens preserved; minor QA overstatement below | Harness skill listing visible, but no Skill/body/reference reads |

These checks do not prove all 24 role capabilities, a real production workflow,
a quality gain over the base models, live application rendering, or every client
surface. Passing artifact checks does not establish skill activation.

## Native skill-use evidence

Codex finance `runs/codex/finance/events.jsonl`:
- Line 8: completed exit-0 command output contains the skill-catalog body.
- Line 14: completed exit-0 task-routing read and `capabilities plan CFO`.
- Line 18: completed exit-0 output contains both financial-analysis and
  engineering-judgment bodies (18,849 bytes of combined output).
- Line 24: completed exit-0 output contains cash-and-earnings and
  boundary-decisions references (6,967 bytes).

Codex localization `runs/codex/localization/events.jsonl`:
- Line 10: completed exit-0 output contains the skill-catalog body.
- Lines 18/23: task-routing lookup and catalog lookup select localization.
- Line 26: completed exit-0 output contains the authored localization body.
- Line 30: completed exit-0 output contains message-integrity reference.

Both Codex invocation records use only “Read TASK.md and complete the requested
task yourself. Do not delegate.” No skill names were supplied in the prompt.
This evidence comes from completed tool commands and their returned content;
reasoning blocks and self-reported skill names were not used as activation proof.

Claude finance `runs/claude/finance/events.jsonl` exposes the Skill tool and lists
all 14 harness globals alongside other host skills, but has no relevant Skill
calls/body/reference reads. Claude Desktop localization transcript
`<home>/.claude/projects/-private-var-folders-rk-rdby38mj0l30jyvbk965q2lc0000gn-T-harness-initial-release-ctt4c72g-trials-claude-localization/47198448-84d4-40f6-9f16-3117de6eb078.jsonl`
has a skill_listing attachment at line 9 and 8 actual tool calls. The calls
read the task/input, inspect runtime locale support, write artifacts and run
checks, without reading the catalog or localization skill. Claude activation
therefore fails these initial implicit trials despite correct fixture artifacts.

## Financial decision review

Both saved results independently reconcile:
- EUR 150,000 revenue less EUR 90,000 operating costs = EUR 60,000 operating
  profit over the three months.
- EUR 70,000 principal is cash outflow, not an operating expense.
- Base balances EUR 30,000 / -20,000 / 0; delayed balances EUR 30,000 /
  -70,000 / -50,000. Both first become negative in February.
- Reserve gaps are EUR 30,000 base and EUR 80,000 delayed. These are net monthly
  cash requirements, not financing approval or proven facility availability.
- The repeated invoice ID counts once. The delayed case moves receipts one
  calendar month and excludes receipts outside the stated horizon.

Codex clearly bounds funding sufficiency to opening/month-end balances, discusses
intra-month timing and unmodeled obligations, and treats principal deferral as
an unapproved sensitivity. Its net-cash target is presented as a monthly model
threshold rather than proof that a particular facility covers all payment dates.

Claude financial findings:
1. **Contract gap — undeclared consecutive-horizon restriction.**
   `trials/claude/finance/analyze.py:142` rejects a period list of
   `['2026-01', '2026-03']`. TASK defines the horizon as the listed periods and
   excludes cash outside it; it does not require consecutive periods. An
   independent added probe retained the same ledger but used that period list:
   the Codex program matched the oracle, while Claude raised LedgerError.
   Remove the extra restriction or explicitly narrow the accepted task contract;
   do not describe this implementation as handling every permitted horizon.
2. **Decision wording — distinguish month-end funding from intra-month need.**
   `decision.md:145` says to size funding at EUR 80,000 to cover the collection
   slip. The artifact itself correctly calculates a possible EUR 110,000
   intra-month reserve requirement at lines 121-123. Carry “month-end model
   threshold” into the recommendation and size/approve the facility only after
   the dated cash schedule; otherwise the recommendation is broader than its
   own evidence. The EUR 110,000 arithmetic is correct for the stated worst
   ordering of these modeled monthly flows.
3. `decision.md:148-149` requests a weekly schedule then rerunning analyze.py,
   while the program accepts only YYYY-MM periods. A separate dated schedule
   or changed model is needed to verify intra-month liquidity; rerunning this
   monthly model cannot establish weekly sufficiency.

## Localization meaning review

Both translations preserve supplier-controlled refund confirmation before
cancellation; a request remains distinct from confirmation. Both preserve the
estimated EUR 1,250.50 total, tax exclusion, 10 October 2026 17:00 deadline and
Europe/Istanbul literal. Both keep pending and not-confirmed booking state,
request-only cancellation button semantics, booking/name placeholders, strong
markup, and all required count/ICU branches with active # substitutions.
Turkish numeral agreement is appropriate in identical one/other branch text.

The source hashes match the fixture, and both QA documents appropriately disclose
no native human, live UI or application MessageFormat review. Intl-backed schema
simulation does not demonstrate application runtime interpolation or visual fit.

Minor Claude QA finding: `translation-qa.md:22` claims a formal register
“throughout,” but `tr.json:6` uses singular imperative “İptal talep et.” A brief
button imperative is a plausible interface style; the QA description should
acknowledge that choice rather than claim uniform formal morphology. This is a
QA precision issue, not a reversal of cancellation meaning.

## Independent checks executed

Ran each repository verifier separately against both providers' saved task
directories:
- `evaluations/cases/finance-cash-decision/verify.py`: 7/7 for each provider.
- `evaluations/cases/localization-ui-contract/verify.py`: 8/8 for each provider.
- Additional read-only imported-function probe for the nonconsecutive horizon
  described above. No delivered files were rewritten.

Finance source ledger SHA-256:
`5be1e094f76fde050cd4c8a0c0648f2fe296009b04bfc10720c04e276a967eeb`.
Localization source SHA-256 for both providers:
`5ba07c8da6fe537ed935ead1773026047287b9b6a559480bfa0061c78da2ae82`.

## Reviewed artifact hashes

```json
{
  "trials/codex/finance/analyze.py": "e6604ecd342030013db34a14ca4fbbbc23d3948690f6c48e57ae9db06844e1f7",
  "trials/codex/finance/results.json": "1b313f2fc27f00879c2a9cefad3d2803f7862177f988f7f1646373788956d677",
  "trials/codex/finance/decision.md": "8e6534db3c0e8fc565afa4bfeff0fa79a008a5cb60add096e96f3ded54549fdd",
  "trials/claude/finance/analyze.py": "c3e8ec5c209d3c4b65364c66dfc093d7652c2ca04a0700e3910b2fd899a2bc37",
  "trials/claude/finance/results.json": "1b313f2fc27f00879c2a9cefad3d2803f7862177f988f7f1646373788956d677",
  "trials/claude/finance/decision.md": "e4278f28dccc909ebc86d059b0d7acc197b13fe687c5974b7e8f811f0b05e8fc",
  "trials/codex/localization/tr.json": "825a07c79cc1fe4cfe9cae7f11d7f88e1593456839523298fada7e7eda00109d",
  "trials/codex/localization/translation-qa.md": "48bd8d92e6a11e8e2e082c0c4d65a29a4ee69f9dfc076e08045e2e561935c6f9",
  "trials/claude/localization/tr.json": "aa0ad0a5db084751568a8c5b368dfce7b723f2b3e1cac50c888c3b284f3fbc5e",
  "trials/claude/localization/translation-qa.md": "203230c9fea02ef81ff5caa4c5de6339ce1ea1ea2fa5522d2788c334aff90e9a"
}
```
