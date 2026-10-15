# Visual reference pilot

This pilot asks whether Mirket's animated-diagram reference changes delivered
work, rather than merely increasing the amount of guidance available. It has
not been run against two host-agent arms by inclusion in this repository. The
[relay study](../../skills/svg-animation/references/animated-diagram-study.md)
and [browser probe](../../tests/browser_diagram_reference.cjs) establish the
candidate payload and its behavior; they do not measure agent improvement.

## Fixed comparison

Use the same host, model, settings, tools, task budget, brief, site revision and
reviewer criteria for both isolated arms. The intentional difference is access
to this version's reference and routing sentence. Preserve raw retrieval traces;
an agent naming a skill is not evidence that it opened the case study. Keep
review criteria outside workers' write scope. Save every planned run, including
an unsuccessful or incomplete result. Repeat before claiming a general gain.

- **Development case:** repair a locally copied, imperfect animated hero diagram
  with four source paths feeding one central result. Specify actual brand tokens,
  placement sizes and semantic content from the pinned site checkout. The
  original mirket.dev hero is a useful *private* source if its owner approves
  using that exact revision; do not ship its brand or unfinished work as a
  public fixture.
- **Transfer case:** use the same toolchain to draw a fictional flow in which a
  request is received, checked and resolved. Give workers the actual content and
  viewing widths, but no specimen geometry or timing. Keep a fresh unseen brief
  for a later evaluation; these public cases are not blind merely because the
  rubric is in another file.
- **Failure probe:** the shipped `?variant=awkward` version of the relay study
  should pass load, static-fallback, route-progress and reduced-motion checks
  while failing the visual judgment for simultaneous competing focal actions and
  a final emphasis on the wrong stage.

For each run, save editable source, normal-speed video, initial/midpoint/final
frames at desktop and narrow widths, reduced-motion and no-script frames, test
logs, retrieval trace, model/tool configuration and observed cost/latency when
available. A deterministic checker can verify artifact presence, no overflow,
visible semantics, replay and live reduced-motion cancellation. Do not let that
checker assign the visual verdict.

## Independent visual rubric

Randomize labels and ask an independent reviewer to inspect normal playback
before isolated frames. The reviewer should watch each clip twice at its native
speed, without knowing which guidance condition produced it. If they cannot
actually view playback, leave pacing/rest unscored rather than inferring it from
stills. Keep the clip's original timing; changing playback speed would change
the thing being judged. Score each criterion 0–2 with a located observation:

| Criterion | 0 | 1 | 2 |
| --- | --- | --- | --- |
| Semantic clarity | Relationships or labels are misleading/illegible at placement size | Meaning is recoverable but requires effort | Static and animated states both communicate the intended relationship |
| Focal sequence | Competing or contradictory emphasis | Mostly followable with a confusing beat | Motion guides the eye through the intended order |
| Pacing and rest | Rushed, lingering or awkward stop/reversal | Serviceable with a noticeable interruption | Movement and rests support comprehension at normal speed |
| Context fit | Clashes with the brand or page hierarchy | Acceptable but generic | Deliberately fits the page and its content |

Require rationale tied to a frame, time or page location. A zero in semantic
clarity or focal sequence fails that run even if total score is high. Have a
second reviewer inspect disagreements when the decision would alter the guidance.
Record whether the awkward variant was rejected; if it passes, revise the rubric
before drawing conclusions about the reference pack. Human preference remains
distinct from accessibility and implementation defects.

For the fictional transfer brief, mark **context fit N/A** because it supplies
no approved brand or surrounding page. For a real hero transfer, freeze the
source revision, actual copy and brand tokens, viewport placements, model,
tools, and time budget before dispatch. Keep the private website material out
of shipped example assets. Ask reviewers to name the route progression and the
moment the destination responds from playback alone; an appealing static frame
or four nearly simultaneous traces cannot answer that question. Preserve the
retrieval trace and each attempted run, and report model/budget parity only when
it was actually observed. One paired result remains exploratory.
