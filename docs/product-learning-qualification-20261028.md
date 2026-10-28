# Product-learning native qualification — 2026-10-28

This is an outcome trial for the R7/R8 product-learning resources. It is not
the unit-test report and it does not claim a skill effect when the native host
does not produce the requested artifacts.

## Frozen protocol

The private trial root is
`/Users/nazmiyilmaz/.codex/task-state/harness-engineering-research-20261028/product-learning-qualification-20261028/`.
The frozen brief and oracle were written before either cell ran. Both cells
received the same evidence and task, used the same Codex CLI, model and effort,
and had no retries or repairs:

| Limit | Frozen value |
| --- | --- |
| Platform | macOS 26.5.2 arm64 |
| Codex CLI | 0.164.0-alpha.11.1 |
| CLI path | `/Applications/ChatGPT.app/Contents/Resources/codex-cli/CodexCLI.app/Contents/MacOS/codex` |
| Model / effort | `gpt-6-astra` / `high` |
| Per-cell wall limit | 90 seconds |
| Shared wall limit | 300 seconds |
| Repairs / retries | 0 / 0 |

The no-skill cell had no `.agents/skills` directory. The with-skill cell had
the six current project/product/marketing skills and their references. The
task required saved evidence review, opportunity map, current/future
positioning, funnel diagnosis, saved copy, copy review, and a staged loop
record.

The independent oracle facts were: `real-quote` is consented real evidence;
`synthetic-quote` is not testimonial proof; `expired-metric` is historical and
expired on 2026-10-01; `buyer-alternative` is the only current buyer
alternative; `future-competitor` is future-only; and activation is the lowest
observed stage (0.05 versus acquisition 0.50 and retention 0.08). These facts
were not copied into the expected output as a model self-report requirement.

## Observed native outcomes

| Cell | Outcome | Independent artifact check |
| --- | --- | --- |
| no-skill | Timed out at 90.012s, exit `-15` | No requested artifact was saved; final manifest contained only frozen inputs. The trace shows evidence inspection began but no completion. |
| with-skill | Timed out at 90.090s, exit `-15` | No requested artifact was saved; final manifest contained only frozen inputs and the supplied skill snapshot. The stderr recorded a model-service `429 Too Many Requests` retry warning. |

The cohort ended after 180.135 seconds, within the shared 300-second budget.
The private runner removed the copied auth file after each cell; no live project,
publication, message, or external write was used.

## Interpretation

This trial is **unqualified**, not a no-skill/with-skill tie and not evidence of
uplift. Neither condition produced a saved copy or funnel diagnosis, so the
independent oracle could not score the intended outcomes. The with-skill cell
also encountered a host/model rate-limit failure; that is an execution
prerequisite, not a product-learning defect attributable to the skill.

The run still establishes two useful limits: a unit checker cannot substitute
for a saved-artifact outcome, and a skill receipt cannot overcome a native
model-service failure. A later qualification must use a fresh frozen specimen
only after the host can complete both identical 90-second cells; it must retain
the same oracle, model/effort, and no-retry budget.
