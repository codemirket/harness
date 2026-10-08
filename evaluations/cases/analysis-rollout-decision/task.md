# Decide whether to expand the checkout pilot

Work only inside this scratch `workspace/`. The fixture is fictional; use only the
supplied data and Python standard library. Do not install dependencies or make
external calls. Read `data/contract.md` before analyzing the rollout.

The product lead sees higher pooled conversion in the pilot and proposes expanding
it to every account tomorrow. Produce a reproducible analysis and a short decision
brief. Create `analyze.py` with this command contract:

```sh
python3 analyze.py --data data --output results.json
```

The command must calculate from the supplied directory, accept other files with the
same schema, and write the JSON shape in `data/output-schema.json`. Produce
`results.json` with that command and `decision.md` with your recommendation,
traceable numerical evidence, limitations, and the next useful decision or check.
Explain the population, grain and denominator, and why the pooled result does or
does not justify a causal conclusion. Link the local input files near claims that
use them. Do not change anything under `data/`.

You can run `python3 -I ../verify.py .` here. It checks calculation behavior on the
fixture and additional datasets, not the quality of the recommendation. The
recommendation and the implementation also require review against `rubric.md`.
