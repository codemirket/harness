# Frame review plan

Use this when a sequence has several beats, timed captions or a timing/layout
change. A static title edit with no timing impact can use its existing affected
frame checks. Keep the plan in the project's existing brief or review notes;
no additional manifest, checker or permanent workflow file is required.

## Name the frames before rendering

Use the selected rational FPS `p/q` and requested frame count `N`. Frame `f`
samples time `f*q/p` seconds, with `0 <= f < N`. The duration is `N*q/p`, but
that endpoint is outside the saved sequence. Review frame `N-1` as the final
picture; seeking exactly to duration can conceal an end-frame defect.

For each affected beat, record the subject, intended visible state and frame
indices in a small table. Include:

- frame `0`, frame `N-1` and the landed reading/payoff states;
- the frame before, at and after each affected scene or caption boundary;
- a midpoint inside each affected move or transition, plus the point of maximum
  overlap when different; matching endpoints do not establish a clean path;
- the intended subject itself, so a moving decoration cannot hide a frozen
  title, chart or character.

Drop out-of-range indices and duplicates. Caption intervals are end-exclusive:
for a cue `[a,b)`, test `a` and `b-1` for its text, `a-1` for the preceding state
when present, and `b` for removal or replacement when `b < N`. A frame at a
boundary needs an expected state, not just a request to look for differences.
After repair, revisit the affected neighbours as well as the repaired frame.

When a brief supplies BPM, calculate each musical anchor from its own index:

```text
frame(k) = round(k * 60 * p / (BPM * q))
```

Do not repeatedly add a rounded frames-per-beat value. At 30 FPS and 128 BPM,
beat 32 is frame 450; adding 14 frames 32 times reaches frame 448. Treat frame
indices as the stable authored anchors and record the chosen rounding rule.
Imported beat times instead use `round(seconds*p/q)`. Respect a deliberate
off-beat cut; the beat grid supplies timing coordinates, not creative judgment.

## Inspect the saved delivery

Use the existing qualified `video_prepare` → `video_start` → `video_result`
route and the pinned authoring reference's private FFmpeg decode recipe. An
upstream snapshot, keyframe path or project browser observation can explain the
source, but it does not substitute for these decoded frames. Preserve the
initial export and evidence before a repair. Sample more densely only where a
defect or fast motion needs it; playback still requires its own observation.

At the smallest delivery size selected in the brief, inspect actual decoded
type, focal hierarchy, caption/footer collision and player/UI obstruction.
Choose safe regions from that delivery target rather than imposing universal
percentages. Measure the number of frames a reading line remains landed and
unobstructed, then inspect normal-speed readability in its language. A contact
sheet of widely spaced instants cannot establish a reading hold or smoothness.

For supplied audio, compare the saved stream's start/end with its authored
placement and inspect or listen near joins and the expected tail. Container
duration, an audio-stream listing and clean decoding cannot establish that the
intended sound survived. Preserve intentional silence and short/late placements;
a padded full-length stream alone does not prove the source remained audible.

Record findings with frame/time, subject, visible defect, smallest useful
repair and recheck indices. Separate source measurements, decoded-pixel
inspection, playback and heard-audio evidence. If a tool/model cannot inspect
pixels or hear playback, retain that limitation; a numeric pass cannot fill it.
Stop at the existing shared review/repair budget, or a capability/rights gap,
and hand off the remaining defects and exact checked export revision.

## Source and compatibility boundary

This independently written method selects the beat-grid calculation and
between-beat review from HyperFrames commit
`10a4ed56cb09159afec0dba3ea784e3162ee2e71`:
[motion](https://github.com/heygen-com/hyperframes/blob/10a4ed56cb09159afec0dba3ea784e3162ee2e71/skills/hyperframes/references/motion.md)
(“Motion craft”, “Seeing motion”) and
[review](https://github.com/heygen-com/hyperframes/blob/10a4ed56cb09159afec0dba3ea784e3162ee2e71/skills/hyperframes/references/review.md)
(“Measure, then look”, “Notes and fixes”). Its
[root license](https://github.com/heygen-com/hyperframes/blob/10a4ed56cb09159afec0dba3ea784e3162ee2e71/LICENSE)
is Apache-2.0. No upstream text, script, template, font, footage or sound is
copied here; linked assets retain their own rights requirements.

The source review does not update Mirket's HyperFrames 0.8.197 runtime pin.
Current upstream instructions for GSAP registration, scene templates, cloud,
registry, media providers, telemetry and `npx` setup are outside this method.
Use the local literal `data-no-timeline`, selected font/assets and caption
contracts; do not replace them with the current upstream defaults. The plan
uses frame arithmetic and saved-output checks, so it needs no newer runtime API
or additional production dependency.
