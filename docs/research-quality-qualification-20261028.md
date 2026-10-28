# Research-quality calibration

Status: protocol and deterministic calibration implemented; native attempts are
partial and rate-limited. Three disposable roots were created while correcting
the protocol; they are one evidence ledger, not independent cohorts or budget
resets. This record is not live-web coverage or a catalog acceptance result.

The protocol in `tests/research-quality-trials.py` uses six small synthetic
source cases, each matched with `no-skill` and `with-skill` conditions:

- current fact with a counterexample;
- stale superseded source;
- swapped product attribution;
- partially supported compound claim;
- unavailable evidence requiring abstention;
- wrong-project capability transfer.

The corpus is copied into each disposable project. The required verdicts and
counterexamples are stored in a frozen oracle outside the project, with a hash
of the case definitions. The model does not receive the oracle. The corrected
protocol records a skill read only when a completed native command event
contains the exact frozen body. `selection_observed` remains unknown; reading
or copying a skill is not proof of native activation or useful application.
The final runner no longer forces skill loading through `AGENTS.md`.

The shared ceiling is 1,200 seconds for all twelve cells, 90 seconds per cell,
zero repairs and zero retries. The runner copies an explicitly supplied auth
file into a private disposable `CODEX_HOME` only for the turn and removes the
copy afterward. It starts the explicitly selected Codex executable; the trial
task forbids recursive agents or additional Codex processes, network research,
live installation edits and external effects. A native timeout remains a failed
execution even if an answer file exists. Host metadata is not provider identity.

The deterministic grader checks verdict, exact source attribution, stale and
wrong-project exclusions, required counterexamples and abstention. It reports
`skill_read_observed`, `selection_observed` (unknown), `execution_observed`,
control contamination and `outcome_observed` independently. Separate frozen
review controls reject constant PASS, false findings against a clean artifact,
and findings with the wrong source locator. Those controls test the evaluator;
they do not constitute a native model-reviewer calibration result.

The unit tests cover frozen-input mutation, all six negative controls, correct
answers, reviewer calibration and separation of selection/execution/outcome.
They use only the saved synthetic corpus and do not establish recall or
completeness for live web search.

## Bounded native attempts and complete ledger

The final corrected frozen root was
`/Users/Shared/mirket-research-quality-20261028-final`.
It sealed the six cases, hidden oracle, corpus, skill snapshot and explicit
Astra/high CLI identity on macOS 26.5.2 ARM64. This root declared a
1,200-second deadline, 90 seconds per cell, zero retries and zero repairs;
the cross-root ledger below shows why the original no-retry protocol failed.

All attempts across the three roots are listed below. The first two roots were
started while the protocol was being corrected and cannot be treated as valid
qualification cells; their elapsed turns still count toward task effort.

| Root | Cell | Result | Captured seconds | Validity |
| --- | --- | --- | ---: | --- |
| `mirket-research-quality-20261028` | current-counterexample / no-skill | exit 0; old runner had no completion field | 17.115 | invalid pre-correction |
| `mirket-research-quality-20261028-corrected` | current-counterexample / no-skill | exit 0; old runner had no completion field | 35.067 | invalid pre-correction |
| `mirket-research-quality-20261028-corrected` | current-counterexample / with-skill | started, interrupted before result | unknown | invalid pre-correction |
| `mirket-research-quality-20261028-final` | current-counterexample / no-skill | exit 0, `turn.completed`, answer pass | 34.967 | valid execution; no activation expected |
| `mirket-research-quality-20261028-final` | current-counterexample / with-skill | exit 0, `turn.completed`, answer pass | 34.357 | valid execution; trace read skill file |
| `mirket-research-quality-20261028-final` | stale-source / no-skill | 90s timeout; 429 retry-limit stderr; no answer | 90.065 | valid failed execution |

The captured total is **211.571 seconds**, plus one interrupted pre-correction
turn whose duration was not retained. Repeated current-counterexample cells are
counted as repeated attempts; the original zero-retry cohort claim is invalid.
The first root did not retain model/effort fields; the corrected and final roots
were configured Astra/high. The final with-skill project contained an `AGENTS.md`
instruction that explicitly required reading the skill, so its trace is evidence
of execution of that instruction, not an unnamed natural-selection result.

The next no-skill cell (`stale-source`) reached its 90-second ceiling without a
native completion. Its stderr recorded a 429 Too Many Requests retry-limit
failure. The process was terminated by the frozen deadline. The cohort was
stopped immediately; no alternate model/provider, retry, repair or resumed
cell was attempted. Nine final-root cells are therefore **not run**, and the
partial ledger cannot establish catalog-wide activation, research quality uplift
or stale/unavailable/wrong-project execution outcomes. Earlier roots remain
listed as effort and invalid diagnostic evidence; they are not merged into the
valid final-root result.

Auth copies were removed from all disposable cells after stopping. The native
attempt encountered provider rate limiting after protocol and grader defects
had also been found. The final corrected runner passed 13 deterministic tests;
it has not had a new native qualification. Any future run needs a new task
decision after that prerequisite is resolved; it must not reopen this frozen
cohort or silently reset its shared budget.
