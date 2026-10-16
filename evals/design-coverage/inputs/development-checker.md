# Development task — local handover receipt

Build a compact, self-contained `index.html` for a fictional community workshop.
A volunteer records that one handover note has been saved locally. Give the
screen a clear content hierarchy, original visual treatment, native named controls
and visible keyboard focus. Keep it usable at 390 px width. No network services,
downloads, build dependencies or real submissions; use local/system fonts.

The page has a `button#save`, `output#state` and `output#saved-count`. Initially
show `Not saved` and count `0`. Activating Save once must show `Saved locally` and
set the actual count to `1`; activating it again must leave the count at `1`.
Explain that the save is a local simulation. Save `decision.md` (at most 300
words) describing the hierarchy, actual checks and verification limits.

This is a development task for collection and checker calibration, separate from
the unseen held-out briefs. Both arms execute it once. It is excluded from the
held-out pilot's quality comparison. The repository also contains explicitly
synthetic positive and negative checker controls; they are not model runs.
