# R3 evidence-binding qualification — 2026-10-28

This qualification used a disposable copy of the retained D1 design/motion
packet. It did not modify the original evidence or the user's live Mirket
installation.

## Actual retained inputs

Origin packet:

`/Users/nazmiyilmaz/.codex/task-state/harness-design-scope-20261027/packet-root/`

The copied packet retained packet ID `d1-real-export`, the saved 640×360
H.264 MP4, the decoded frame-24 PNG, and the actual
`export/runtime-receipt.json`. The receipt's terminal result records:

- operation: `op-18e27c217e884700-b211-0`
- Mirket release: `motion-authoring-20261027`
- project root: `/private/var/folders/4z/v18wnt491wd53z5kctrk2rgc0000gn/T/mirket-motion-authoring-GsZSQV/project`
- effect: `succeeded` / `confirmed`
- artifact revision: `sha256:6d81087091f3e5188764ad2cda7582edc84e822ae1be5b0d49c8c68f00b81de8`
- measured profile: 640×360, 12 fps, 36 frames, 3 seconds

The media probe was copied from the receipt's actual `ffprobe` payload into
`export/media-probe.json`; it was not fabricated or re-rendered. The retained
receipt's `result` wrapper is accepted as a qualification wrapper around the
canonical host report, with the outer receipt hash still checked.

Disposable roots:

- clean qualification: `/Users/nazmiyilmaz/.codex/task-state/harness-evidence-bindings-qualification-20261028.2gznvH/`
- mutation cases: `/Users/nazmiyilmaz/.codex/task-state/harness-evidence-bindings-mutations-20261028.EQNAWZ/`
- origin/hash record: `/Users/nazmiyilmaz/.codex/task-state/harness-evidence-bindings-qualification-20261028.2gznvH-origin.json`

## Clean actual validation

Command:

```sh
python3 scripts/check-design-evidence.py \
  --packet /Users/nazmiyilmaz/.codex/task-state/harness-evidence-bindings-qualification-20261028.2gznvH/review.json \
  --root /Users/nazmiyilmaz/.codex/task-state/harness-evidence-bindings-qualification-20261028.2gznvH
```

Result: valid. The checker reported:

```json
{
  "operation_receipt": "bound",
  "media_probe": "checked",
  "execution_identity": "frozen_record_only; provider_identity_not_authenticated",
  "rights_references": "recorded"
}
```

The operation binding points to the actual copied runtime receipt and MP4.
The media binding points to the actual copied export, the retained ffprobe
measurements, and the packet's decoded-export capture. The PNG digest is never
compared to the MP4 revision.

The packet also binds a retained Noto Sans input and a copied project CSS asset
to explicit rights rows and per-path SHA-256 values. The CSS copy is outside
the source manifest so its mutation exercises the rights binding independently
of source-manifest checking.

## Independent mutation outcomes

Each case started from a fresh copy of the clean disposable root. Receipt
mutations updated only the copied outer receipt hash and binding hash, so the
checker reached the semantic mismatch rather than failing merely on a stale
record hash.

| Mutation | Outcome | Refusal |
|---|---|---|
| Receipt result project root changed to `/private/other-selected-project` | exit 1 | `operation receipt project differs from the binding` |
| Receipt result artifact revision changed while MP4 remained unchanged | exit 1 | `operation receipt artifact revision differs from the binding` |
| Rights-bound `export/reference-only.css` bytes changed; source manifest unchanged | exit 1 | `stale referenced asset bytes: export/reference-only.css` |

## Execution identity and limits

The frozen tool observations came from the retained runtime receipt and the
measured executable paths on macOS `darwin-arm64`; tool paths and hashes were
recorded for Mirket, Node, adapter, Chromium, FFmpeg and ffprobe. The original
receipt did not retain a Codex release ID, model, or reasoning-effort value;
the packet records those fields explicitly as `not-recorded-in-retained-receipt`.
Therefore the execution result remains record-only and does not authenticate
provider-side identity.

This is an actual-byte record-integrity qualification, not a visual acceptance:
the checker does not decode media, inspect pixels, exercise playback, or prove
rights beyond detecting changes to the recorded local artifacts. The retained
visual report remains limited to frame 24, as stated by the original packet.
