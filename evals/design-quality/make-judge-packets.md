# Neutral craft packets

`make-judge-packets.py` uses the frozen experiment assignment and the existing
analysis contract. It launches no models and supplies no ratings. It never edits
the frozen protocol, prompts, thresholds, rubric or original returns.

The private schedule is `experiment/judge-packet-setup/schedule.v3.private.json`.
It contains 32 analysis-compatible `label_map` rows. Each judge sees all sixteen
held-out artifacts once, in a different fixed permutation. Within each matched
brief/repeat pair, first-arm placement is balanced four/four and reversed across
judges. Earlier schedules are retained. The current v3 updates helper identities for
optional experiment-root I/O portability; its mapping, order, views and
calibration fields equal v2. See reuse.md for a caller-owned directory.

After context 1, all sixteen actual returns, and final root observer replays are
frozen, save a project-relative replay manifest:

```json
{
  "schema_version": 1,
  "kind": "final_root_observer_replays",
  "experiment_freeze": {"path": "EXPERIMENT/freeze.json", "sha256": "ACTUAL"},
  "observer": {"path": "evals/design-quality/observe.cjs", "sha256": "FROZEN"},
  "entries": [
    {"cell_label": "H01", "observation": {"path": "ACTUAL/observations.json", "sha256": "ACTUAL"}, "status": "completed", "reason": ""}
  ]
}
```

The entries must cover all sixteen cells. A failed replay uses
`infrastructure_failed`, retains its actual observation file and gives a reason.
Missing views remain unknown; output identity drift is refused. The builder
checks that observer source matches the frozen observer, and artifact file maps
match actual sealed receipts before and after collection.

```sh
python3 evals/design-quality/make-judge-packets.py build --replay-manifest PATH
```

Only give judges their own `judge-packet-setup/packets/A` or `packets/B`
directory. It contains four raw briefs, neutral PNG names, the craft rubric,
view availability and a blank first-stage score template. It contains no hashes,
byte sizes, condition/source paths, decisions, HTML, logs, guard outcomes or
peer scores. Private maps, original-to-copy identities and packet hashes stay
outside those directories. Every initial 1280/1440/390/320 viewport/full-page
slot and every predeclared scenario capture/final slot is fixed. A missing 1440
view is unknown. Failure-only or preferred frames never substitute for a slot.

Each actual judge fills all sixteen score records, with two located visual
observations per dimension, before making arm guesses. Bind that complete lock:

```sh
python3 evals/design-quality/make-judge-packets.py lock-scores --judge A --input ACTUAL_SCORE_LOCK_INPUT
```

This validates scores, evidence and schedule, saves a new immutable score file,
and only then releases `guess-forms/A.json`. The actual judge fills the guesses
and confidence 0–100 in a separate file, binding the existing lock. Then:

```sh
python3 evals/design-quality/make-judge-packets.py merge-guesses --judge A --input ACTUAL_GUESS_INPUT
```

Repeat for B. `judgments/A.json` and `B.json` follow the exact five-dimension
analysis contract. No scores change during merging. Bind these, the actual
calibration artifacts already recorded in the private schedule, final guard
report, coordinating integrity evidence and private render/schedule identities
into the existing `analyze.py` manifest. Calibration artifacts are never
reconstructed as hand-entered outcomes.

The helper's twelve structural checks are explicitly synthetic and execute zero
models. Actual packet construction awaits real held-out evidence. Read boundaries
and identity hashes are not a sandbox or authentication. Agent ratings and
calibration do not establish human acceptance or absence of judge bias.
