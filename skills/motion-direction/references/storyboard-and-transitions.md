# Storyboard and transition direction

Use this reference when a motion brief has multiple beats, scenes or a
non-obvious transition. For a single interaction transition, the main skill's
timing and interruption guidance is enough.

## Compact shot card

Before writing animation code, record only the decisions that affect the
sequence:

```text
Intent: what should the viewer understand or feel?
Focal subject: what leads attention in this beat?
Setup -> payoff: what is introduced, and what changes?
Layers: foreground / subject / supporting context / background.
Keyframes: first / one or more meaningful beats / resting frame.
Read and hold: when can the viewer read or act?
Camera or viewport: movement, if any, and the subject relationship that motivates it.
Transition to next beat: cut, hold, match, dissolve or other choice, with reason.
Acceptance frames: the times or frames an independent reviewer will inspect.
Reduced motion: the same information and controls with less movement.
```

Choose the number of beats and acceptance frames from the content and budget.
Compare alternative directions only while the direction is genuinely unresolved;
record why the selected one fits and do not reopen an accepted user choice.
There is no universal requirement for three options, a particular transition,
or a no-hard-cut rule. A hard cut can be the clearest relationship between
scenes; a hold, match or gradual transition should earn its duration by
preserving focus or explaining continuity.

Make camera movement purposeful: it may reveal a subject, preserve its reading
scale, connect adjacent scenes or establish context. If it only adds activity,
remove it. Keep one owner for each animated property and specify what happens
when input interrupts, reverses, cancels or finishes. Review the complete
sequence at normal speed and inspect the acceptance frames from the saved
artifact, not just a storyboard or whole-frame hash.

## Output and evidence

Return the shot card, the editable source, and either a playable specimen or a
clear untested-playback note. For exported video, pass beat times to the
video-production route and independently check decoded frames at the selected
acceptance points. Preserve source, output, findings, repairs and remaining
defects under the shared task review budget; a new specimen does not reset it.

This adapts the storyboard, composition and continuity decision method from
[Huashu Design's storyboard reference](https://github.com/alchaincyf/huashu-design/blob/4fc7bfdbd5ec3fa2d43ce7a6433de60976a5e4b1/references/storyboard-basics.md)
at commit `4fc7bfdbd5ec3fa2d43ce7a6433de60976a5e4b1`. It is independently
worded guidance. The inspected repository is MIT-licensed; this file does not
copy its source text or assets, and any future adapted source/asset must retain
the applicable notice. Do not import its installer, global skill writes, remote font
handling, alternate renderer, voiceover stack, fixed quotas or approval model.
