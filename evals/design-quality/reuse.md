# Reuse with caller-owned evidence

These helpers launch no models, install no runtime and reconstruct no history.
They require a caller-selected directory and never default to a previous private
study. Pass a normal project-relative experiment directory **before** the subcommand:

```sh
python3 evals/design-quality/prepare.py --experiment-root .mirket/my-study/experiment initialize
python3 evals/design-quality/prepare.py --experiment-root .mirket/my-study/experiment freeze --improved-context .mirket/my-study/contexts/context-1
python3 evals/design-quality/make-judge-packets.py --experiment-root .mirket/my-study/experiment schedule
python3 evals/design-quality/make-judge-packets.py --experiment-root .mirket/my-study/experiment --schedule .mirket/my-study/experiment/judge-packet-setup/schedule.v3.private.json build --replay-manifest .mirket/my-study/final-replays.json
```

The experiment directory's parent owns `development/`, `dev-dispatch/`,
`contexts/`, `data-audit/` and `judges/`. The experiment directory owns hidden
inputs, seals, initialization, configurations, receipts, observations and packets.
Provide the corresponding inputs and actual project-relative SHA-256 bindings
before invoking a stage. Empty directories fail with the required input path;
there is no automatic fallback to the current study. Absolute roots, traversal,
symlink roots and output overwrites are refused.

For initialization, the caller supplies the development briefs/rubric and bound
development dispatch, four hidden briefs/rubrics/probes and their two sealed
manifests, plus the verified baseline snapshot at `contexts/context-0`. That
snapshot contains its actual `bin/mirket`, guidance and source identities.
Context 0 here means the genuine current twenty-seven-reference baseline at
capture time. Context 1 is a separately frozen intervention. Ordinary interface
guidance stays at context 0 for both arms. Preserve the historical source
provenance; do not invent a baseline by deleting references or rules from today's
guidance. These fixed study helpers do not download private snapshots or supply
execution receipts, judge ratings, calibration results or human acceptance.

A fresh checkout includes synthetic controls, observer, calibration tools and
the analysis code. It does not include this study's private historical contexts,
hidden cases, actual returns or judgments. Use existing local Playwright/Chrome
and the documented control contracts to exercise those public checks; passing a
synthetic control is not a model trial or a quality result. `analyze.py` already
accepts an explicit bound input manifest.

After first-wave unblinding, cases may be exported for public regression checks.
Reusing disclosed cases is a regression study, never a fresh blind evaluation.
A new quality wave needs independently authored sealed cases, real fresh workers,
actual independent judges and a new recorded protocol. Keep failed waves and
unknown measurements. No publication or external effects are authorized by these
local helper commands.

The current main packet schedule is v3. Its 32 label mappings, presentation
order, view policy, calibration identities and scoring rules are unchanged from
v2; only the final I/O helper bindings and provenance note differ. No frozen
README, worker prompt, rubric, initialization or prior score file is edited.
