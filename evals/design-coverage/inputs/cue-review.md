# Lantern Room — rehearsal cue review

Build a polished, self-contained `index.html` for a fictional theatre's rehearsal
review desk. A stage manager filters cue durations, inspects a cue and edits its
note while a colleague revision may arrive. The design should feel appropriate
to concentrated creative work. This is a local simulation, with no real
collaboration or network services, downloads or build dependencies. Use original
visual choices and local/system fonts. The working screen is the primary output.

Use these exact fictional cues:

| ID | Cue | Duration in seconds | Existing notes |
| --- | --- | ---: | ---: |
| C01 | Prologue | 42 | 3 |
| C02 | Harbor | 75 | 0 |
| C03 | Ropewalk | 96 | 2 |
| C04 | Lanterns | 128 | 1 |
| C05 | Homecoming | 166 | 4 |
| C06 | Epilogue | 205 | 0 |

Start with all six cues and a maximum duration of 210 seconds. The user can set a
maximum duration and apply it. Show the actual visible count and sum of visible
durations. An empty result explains the filter and offers Clear filter. A visual
duration comparison is useful if it has an equivalent keyboard-accessible list.

Selecting a cue opens its inspector. If filtering removes the inspected cue,
clear the inspector and communicate why; do not silently display a different cue.
Keep any unsaved note associated with its original cue so it can be recovered
when the cue is selected again.

For C01, start with the note `Hold blackout until bell.` Let the user edit it.
`Load colleague revision` simulates revision 2 containing
`Keep blackout until second bell.` If the local note is dirty, preserve it and
show the colleague's text as a conflict instead of silently overwriting it.
Provide an explicit `Keep my draft` resolution. `Save note locally` then saves
the local note exactly once; repeating Save without a new edit is not a new save.
Label simulated revisions and local saves truthfully. This is not a production
collaboration engine.

Use native, named, keyboard-operable controls, visible focus and meaningful
status announcements. Preserve usable filter, list and editor access at 390 px
width. Respect reduced motion if motion is used.

Expose this public production contract for independent inspection:

- `#max-seconds`: numeric input; `#apply-filter` and `#clear-filter`: buttons.
- `#visible-count` and `#visible-duration`: outputs containing decimal numbers.
- Each visible cue has a button `[data-open-cue="C01|...|C06"]`; unavailable
  filtered cues must not have visible open controls.
- `#inspector`: `data-cue` set to the inspected ID or empty when none is selected.
- `#note`: textarea; `#load-colleague`: button; `#conflict`: visible only during
  conflict; `#remote-note`: the colleague revision; `#keep-draft`: button.
- `#save-note`: button; `#saved-note`: output of the actual saved note;
  `#save-count`: output of the actual local save count; `#status`: visible status.

Also save `decision.md` (at most 700 words): explain the content hierarchy, the
relationship transferred from any reference actually read and adaptations for
this brief, the states exercised and remaining limits. If the available reference
does not cover this combination, name the gap and use ordinary design judgment.
Keep teaching notes and reference names outside the product interface.
