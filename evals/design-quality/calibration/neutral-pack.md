# Calibration without visible identity metadata

Use `neutral-pack.cjs` for a new study. It reuses the original packer, pixels,
briefs, labels, presentation orders and validation contract, then keeps image
hashes and byte sizes in the private key. The judge packet contains image paths
only. Repeated visible hashes in the earlier study could reveal the identical
control before visual inspection; its calibration results retain that limit.

```sh
node evals/design-quality/calibration/neutral-pack.cjs OBSERVATIONS_DIR NEW_OUTPUT_DIR
node evals/design-quality/calibration/validate-judgment.cjs JUDGMENT.json NEW_OUTPUT_DIR/key.json
```

The destination must be new. Keep `key.json` and any negative-test keys outside
the assigned judge directory. Give each judge only its packet, the rating format
and its owned output location. Lock the original complete ratings before source,
functional outcomes or peer judgments. The same fixed calibration thresholds
apply. Do not use a replay of old ratings as a newly executed calibration.

This removes one metadata cue. It does not prove attention, blinding, perceptual
truth or human acceptance. Synthetic controls remain separate from design gains.
