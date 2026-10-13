# Reference evidence and transfer

Use the fields needed to keep a reference decision retrievable and actionable.
A short task note can hold one reference. A comparison can use a table or board
with the same evidence; it does not need a new document format.

## Compact record

```text
Decision: audience/task, medium, open question, constraints already accepted.
Reference: title/creator when known; original URL or supplied artifact;
  version/pin or inspected date; discovery URL if different from original.
Evidence: capture/source locations; viewport/use size, theme/content/state;
  action or playback interval actually inspected; blocked or unknown coverage.
Fit: observed strength relevant to this decision; conflicts or tradeoffs;
  why this source is informative; duplicate/common origin if applicable.
Observed: located facts supported by the saved evidence.
Inferred: interpretation, confidence and evidence that would change it.
Proposed: transferable relationship and its concrete adaptation to this brief.
Exclude: source-specific identity, claims, content, assets or behavior to omit.
Rights: ownership/license evidence for any proposed reuse, or unknown;
  distinguish linking/studying from copying a font, image, code or asset.
Handoff: craft owner, current project constraints and smallest useful specimen;
  question the owner must verify in the actual result.
Overlap check, when output exists: paired reference/output locations;
  what matches, what differs, context and any correction still needed.
```

Keep uncertainty local. A missing narrow-screen capture limits responsive claims
but need not invalidate an observed type relationship. A blocked interaction is
unknown rather than passed or failed. Qualify an attractive but unsuitable source
as useful only for the relevant aspect, or omit it when it contributes nothing.
If a required reference cannot be inspected, report the missing evidence and
continue useful work supported by available sources.

Use actual rendering or playback for claims about behavior. Prefer trustworthy
native captures; wait for the relevant state rather than relying on a fixed sleep
alone. Label any stitched image or edited sequence as a composite and retain its
source captures. A screenshot pan is not evidence of website motion. Use the
existing interface or motion owner's observation workflow when deeper inspection
is needed; do not construct a capture framework for an ordinary reference study.

## Worked example: a dispatch console

The following is a hypothetical teaching case. Its artifacts, observations and
product are invented to illustrate the record; they are not claims about a live
service or a completed evaluation. Real work replaces these labels with actual
source and evidence locations.

**Decision.** Operators need to assign selected deliveries without losing their
current filter. The project already has an approved teal palette, native table
selection primitives and an established keyboard journey. The open question is
how to keep selection and the next action apparent in a dense result view.

| Candidate evidence in this hypothetical case | Fit decision |
| --- | --- |
| Supplied reference A: a gallery cover showing a spacious marketing hero, testimonial photo and “12,000 teams” claim; creator attribution available but no original page or asset license | Useful only for the contrast between headline and supporting copy. It provides no evidence for dense selection, keyboard input or narrow layout; exclude from the interaction direction. |
| Reference B: original product demo, captured at 1280px with filtered results, then after selecting rows; keyboard activation of the selection control was exercised; the selected count and Assign action remained above the table | Strong match for making the current selection and next action legible. No narrow view or refresh behavior was inspected. |
| Reference C: repost of B's same demo capture under another gallery title | Retain the original identity and avoid counting the repost as independent support. It adds no new evidence. |

The choice follows task fit and inspected coverage. No popularity, award or
numerical score is needed to explain why B informs the direction better than A.

### Record for B

- **Source and evidence:** original demo URL, creator and inspection date; saved
  filtered and selected captures plus the keyboard-action recording. In a real
  record, include retrievable paths and the relevant recording interval.
- **Observed:** the selected count and Assign action are visible together above
  the table in the captured selected state. Keyboard activation changes the row's
  selected state in the observed demo session.
- **Inferred:** grouping the count with Assign may reduce uncertainty about the
  operation's scope. This is a design hypothesis, not measured task success;
  ambiguity in a working specimen would weaken it.
- **Proposed:** reuse that relationship in the current toolbar using existing
  components. Keep selection visible while filters change and define what the
  project's actual selection model permits. Do not infer cross-filter persistence
  from the reference's appearance.
- **Excluded and rights:** omit B's name, sample dispatch data, palette and icons.
  No source asset is proposed for reuse; its asset licensing remains unverified.
  Retain the project identity and real labels. A's customer claim and photograph
  supply neither valid product evidence nor permission for reuse.
- **Handoff:** `interface-design` builds the affected toolbar/table slice with the
  project's real long labels. Inspect whether count/action scope stays clear in
  supported filtered, empty and narrow states with the existing keyboard journey.
  Source B's unknown states remain unknown; the project's own implementation must
  satisfy its established behavior and verification requirements.

### Paired check after an adaptation

Suppose the first draft accidentally includes A's “12,000 teams” label and its
testimonial photo. The finding pairs A's cover text/photo with the exact draft
component and asset location. Replace the unsupported claim with verified project
content or omit it; replace the photo with an appropriate authorized asset. A
recolor would not resolve either carryover. Shared conventional toolbar grouping
is recorded as a common pattern, not automatically flagged as copying. Reinspect
the corrected draft in its use context; neither similarity counts nor this
specimen establishes rights, usability or human acceptance.

## Compare structure when the sequence is the open decision

For a narrative page or collection, compare what the visitor does and learns:
the organizing unit, navigation's role, opening behavior, sequence and ending.
Use equivalent inspected states and comparable fidelity. A different palette or
typeface can conceal an unchanged interaction structure. Conversely, repeating
a useful navigation convention can improve consistency; novelty is not a gate.

The following is an illustrative comparison, not a claim about inspected sites:

| Reference hypothesis | Structure to inspect | Fit and cost for an instrument collection |
| --- | --- | --- |
| A: chaptered product story | A shared opening followed by staged chapters and a concluding selection | Can explain one unfamiliar mechanism clearly; the visitor may have to traverse unrelated chapters to compare objects |
| B: direct specimen index | Objects are reachable from an index, each with comparable captions and a persistent selection | Supports comparison and return visits; the unfamiliar mechanism needs a concise explanation near its object |
| C: a recolored version of A | Same opening, navigation, chapter sequence and closing action | Tests a visual treatment, not a different browsing concept |

If the brief prioritizes comparing instruments, propose B with a short optional
mechanism demonstration. That hybrid is a hypothesis for `interface-design` to
evaluate with real content and navigation, not an automatically superior solution.
Preserve A's tested benefit and reconsider it if explanation becomes the primary
user task. An already accepted chaptered direction should instead be refined within
its constraints. Use the existing concept-exploration method only when a meaningful
choice remains unsettled; do not create extra planning documents for routine work.

Record the intended change in understanding and observable before/after states
when motion is relevant. An agent's impression of suspense or delight is a critique
hypothesis, not measured audience emotion. No required peak, unique signature move,
exclusive grammar or numerical difference from every historical project applies.

Structural comparison is adapted from Nate Herk's [scroll-craft structure axis](https://github.com/nateherkai/scroll-craft/blob/75d81f74e83692add18cd7a8a8e078b8a887a579/plugins/nateherk-design/skills/scroll-craft/references/uniqueness.md),
MIT, with an original Mirket example. The source's novelty quotas and mandatory
grammar restrictions are omitted; reference access still does not grant asset rights.

## Technique provenance

This reference-study method adapts the source registry, inspected page/state
evidence and paired overlap comparison from Meng To's
[daily-ui-inspiration-capture](https://github.com/MengTo/Skills/blob/83a47fee32f0b6349bff1fede99257a5ef03dc93/agent-skills/codex/daily-ui-inspiration-capture/SKILL.md)
and [audit-reference-originality](https://github.com/MengTo/Skills/blob/83a47fee32f0b6349bff1fede99257a5ef03dc93/agent-skills/codex/audit-reference-originality/SKILL.md),
with sequence observation informed by
[video-to-superprompt](https://github.com/MengTo/Skills/blob/83a47fee32f0b6349bff1fede99257a5ef03dc93/agent-skills/codex/video-to-superprompt/SKILL.md)
and [html-to-interaction-prompts](https://github.com/MengTo/Skills/blob/83a47fee32f0b6349bff1fede99257a5ef03dc93/agent-skills/codex/html-to-interaction-prompts/SKILL.md).
Pin: `83a47fee32f0b6349bff1fede99257a5ef03dc93`, MIT, copyright 2026 Meng To.
The applicable notice accompanies installed guidance. This adaptation includes
no upstream helper, asset pack or demo. Fixed counts, prescribed publication,
daily scheduling and arbitrary similarity targets are omitted. A repository's
root license does not establish rights to every linked or bundled third-party
asset; retain per-item provenance before reuse.
