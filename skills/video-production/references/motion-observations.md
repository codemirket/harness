# HyperFrames observation handoff

HyperFrames remains the sole producer. The optional
scripts/check-motion-observations.mjs checker is a post-capture numeric gate,
not a renderer or a replacement for the pinned HyperFrames contract.
It is harness repository qualification tooling, not a distributed skill tool;
use project-owned checks elsewhere rather than assuming that path exists.

After video_result and independent FFmpeg/FFprobe checks produce a decoded
sample, retain a project-local observation packet with:

- the HyperFrames source and export revisions;
- rational FPS and frame count;
- frame timestamps matched to that FPS, viewport/DPR, theme and locale;
- local font hash and a positive font-loaded observation;
- named beat frames plus midpoint/transition frames;
- stable entity rectangles measured from the decoded source or an independently
  controlled browser geometry extractor;
- moves, holds, settles, inside, position, no_clip and no_overlap checks as applicable.

Run:

    node scripts/check-motion-observations.mjs \
      --observations /absolute/project/.checks/motion-observations.json \
      --source /absolute/project/index.html \
      --export /absolute/project/result.mp4 \
      --font /absolute/project/NotoSans.ttf

Browser-only geometry may omit the export and must remain explicitly
`unbound_browser_capture`. Do not pass an HTML source as if it were an export.

A passing result means only that the supplied numeric observation packet satisfies
the declared relations and that its source/export/font hashes match. Browser geometry
remains a source observation, not decoded-pixel evidence. It does not
prove decoding, playback, pixel inspection, correspondence between source and export, audio/caption sync
or visual quality. Keep the decoded frames and display them with an
image-capable tool before making a visual claim.

Use a deliberate midpoint defect whose endpoints still match as a held-out
control. Also test overlap at a transition, a failed containment relation and a
non-settled final beat. The checker must fail these without changing the shared
repair budget. Do not add a second timeline engine or infer a renderer from this
contract.
