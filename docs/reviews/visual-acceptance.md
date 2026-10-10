# Visual acceptance correction

Date: 2026-10-09. Scope: shared review guidance for visual deliverables.

## Failure and decision

The owner rejected the Mirket hero illustration after an agent had recommended
the draft and described its hand as needing refinement. The right hand does not
form a convincing grip: the pencil crosses the extended fingers without readable
thumb/finger contact and occlusion. This is a focal drawing defect requiring
revision. The surrounding page's legibility and restrained styling do not make
the illustration ready for demonstration.

The shared [instructions](../../instructions/AGENTS.md) and
[delivery standards](../../skills/skill-catalog/references/delivery-standards.md)
now route visual work to the expanded
[artifact review](../../skills/work-planning/references/artifact-review.md).
It requires composition and detail inspection, checks specific to each format,
a clear acceptance decision, and repair followed by inspection of the new output.
It also asks authors to resolve demanding representative elements before scaling
the work and to change construction methods when repeated polish fails.

These are corrections to review behavior. They do not establish improved drawing
ability or consistent professional outcomes across every medium.

## Representative visual check

A fresh read-only `gpt-6-sol` reviewer at high effort received the new guidance,
the marketing-page brief and the rendered image. It did not receive the earlier
endorsement, the owner's criticism, the expected finding or prior reviews.
It inspected the full image and an enlarged illustration detail.

- Decision: **needs revision before public demonstration**.
- Finding: the right hand does not convincingly grip the pencil; contact and
  occlusion need to be redrawn and checked at the actual hero size.
- Nonblocking preference: the boundary/environment labels feel detached from
  the reading scene.
- Limit: a desktop still cannot establish responsive or interactive behavior.

The result reproduces the correct rejection of a known failed example. It is a
single development check, not a held-out comparison or proof that the workflow
improves all future artifacts. No new illustration was produced in this change.

Image identity:

- Original: `/Users/nazmi/.codex/worktrees/visual-craft/mirket-pilot/build/visual-pilot/candidate-desktop-light.jpg`.
- Review copy: `/Users/nazmi/.codex/worktrees/visual-acceptance/.ai/build/visual-acceptance/hero-review.jpg`.
- SHA-256: `817ad8b5f1069ff9122da2411c385c8769984f50fa5cbe151a79abac468ff545`.

## Verification and boundaries

Developed separately from the live linked checkout and the earlier visual-craft
candidate. The latter's changes were not promoted. Before integration:

- Harness suite: 581 tests passed.
- Context doctor, generated registry check, POSIX shell syntax and diff whitespace
  checks passed. The authored work-planning payload hash was refreshed.
- Isolated copy installation and file checks passed for Codex and Claude in a
  disposable home; no blockers or pending changes.
- Markdown structure/link check passed. The review guidance was rendered in
  Chrome and visually inspected: hierarchy and table are readable, with no
  horizontal overflow at the inspected 1200-pixel viewport.
- The optional skill-creator quick validator could not run because PyYAML is
  absent from both available Python runtimes. No dependency was installed.

Live global instructions and the affected global skill directories link to the
main harness checkout. Applying the reviewed files there updates those sources;
it does not prove that an already-running client has refreshed its context.
No artwork, application implementation, production dashboard, client settings,
commit or publication is included in this change.

## Continuation: static drawing repair

The original session was reconciled with this correction. All six correction
files matched between main and the acceptance worktree before continuation.
The broader candidate remains separate; its trial and task records now explicitly
record the owner's rejection. The rejected JPEG retains the hash above, and its
Vue source was preserved under the pilot's `build/visual-pilot/rejected-source/`.
No shared instruction or skill was changed during this continuation.

A new static SVG study was constructed from an inspected photographic grip
reference, preserving the book composition while redrawing the writing hand and
pencil. The first attempt was also rejected: a leaf-shaped thumb and nearly
straight index did not resolve the grip. A subsequent construction has a flexed
index, opposing thumb overlap and curled support beneath a shallower pencil.

Primary review inspected the actual 420px and 342px artwork, its enlarged detail,
and a desktop placement study. A fresh read-only reviewer, without producer notes
or prior verdicts, independently found the action legible and no blocking malformed
focal element. It judged the static illustration ready for design review. Rear-book
and pencil overlap remains visually dense; simplified anatomy is intentional.
This is agent judgment of one repair, not owner acceptance or broad quality proof.

Artifacts in the existing Mirket pilot:
`build/grip-redraw/GripStudy.svg`, `hero-420.jpg`, `narrow-342.jpg`,
`grip-detail.jpg`, `placement.jpg`, `IndependentReview.md` and `Readme.md`.
Master SHA-256:
`95b84b2751ad73f572315ced01b8b93ae9fafb5457eb8afff834810282a76961`.
The notes cite the photographic construction reference; no reference image is
embedded or traced. Earlier failed geometry is retained alongside the master.

Structural SVG audits passed. This is a light-theme static study only. Its
fixed desktop placement page is not responsive integration evidence; the 342px
standalone art was separately inspected. The Vue component and its animation
remain unchanged and rejected. No motion, dark-theme or production-integration
claim is made. Prior technical checks above were not rerun or reattributed to
this drawing. No dependency, commit, push, publication or Trixpo change occurred.

## Reopened: static redraw rejected

The owner rejected the static redraw as well. Primary reinspection of
`grip-detail.jpg` and `hero-420.jpg` confirms the remaining focal craft failure:
the upper finger is an elongated hook, the broad thumb merges into a wedge-like
palm, and the supporting fingers read as nested loops rather than a coherent
hand. Suggested contact around the pencil is not a convincing anatomical grip.
The earlier primary endorsement is retracted. The independent review remains a
record of a missed defect, not acceptance evidence. This static study is rejected;
do not integrate, animate or recommend it. Further work needs a different drawing
construction approach, not another contour-polishing pass.

## Continuation: production routing and complete review briefs

The AI-only production trial is retained separately in the `visual-acceptance`
worktree's `docs/reviews/illustration-production-trial.md` and local
`build/illustration-lab/` evidence bundle. Three fresh review approaches accepted
the owner-rejected redraw, including one supplied with labeled visual examples.
These false approvals remain evidence against reliable automated acceptance.

The subsequent narrow correction changes illustration/asset routing from a
mandatory SVG lead to medium selection before registration. Native image tools
handle complex bitmap work when available; explicit vector and motion requirements
remain binding. Review assignments carry the complete applicable brief and separate
producer choices from user requirements. An omitted constraint remains unreviewed;
the delivering agent retains responsibility rather than deferring to reviewer votes.

A fresh author applied this route to one non-destructive raster edit of the Little
Hours otter. Primary inspection at full resolution and 420/342px found the requested
reduction of workshop clutter, clearer face/cup hierarchy, retained two-paw contact
and grounded full-body pose. Master SHA-256:
`4f911bd660c77eadf5e1e9c35d3542645758ba353b742a987ec228bd9f20bf1a`.
This is a useful static refinement, not a new Mirket image or a universal style.

A different fresh reviewer received the complete paper-ribbon production brief
without prior verdicts. It rejected ambiguous red/ivory and blue/ivory continuity
after inspecting full size and 420/342px, whereas the earlier abbreviated-brief
review had accepted it. This is one successful constraint-preservation check, not
a controlled causal comparison or proof that the hand-review failure is solved.
Exact briefs, prompts, screenshots and reports are under the worktree's local
`build/illustration-lab/continuation/`; nothing there is published.

Verification for this correction: 581 tests passed; capability integrity, generated
registry, context doctor, POSIX shell syntax and whitespace checks passed. Changed
Markdown structure/local links passed, and the rendered review guidance was
inspected. Disposable Codex/Claude copy installation and checks passed. The optional
skill-creator validator remains unavailable because PyYAML is absent; no dependency
was installed. These checks do not establish visual quality or fresh live-client
activation. Only the six reviewed routing, review, catalog and evidence files are
eligible for copying to main after checking destination hashes for conflicting edits.

The six files were copied to the main harness after the conflict check and
verified byte-for-byte. Codex and Claude global skill links resolve to that main
checkout; already-running sessions may retain earlier context. No commit, push,
publication, production change or dependency installation was performed.


## 2026-10-09 — combined illustration, diagram and page exercise

Added the development case `visual-illustrated-explainer` to the existing evaluation
engine. Its complete brief requires an original heron illustration, an exact native
publication diagram and a responsive Fieldglass explainer. The producer chooses
style; no palette or visual treatment is made a shared default. Prepare a fresh run:

```sh
python3 ai.py eval prepare --case visual-illustrated-explainer \
  --output build/fieldglass-new-run --model 'actual model or unavailable' \
  --condition candidate --settings 'actual reasoning, tools and budget'
```

The completed local trial used native image generation for the expressive bitmap,
editable SVG for exact relationships and HTML for typography. The illustration was
inspected at native resolution and 420/342px, and the actual page at 1280/390px.
The author caught a revision-label backing that let a line touch the text, repaired
the SVG source and captured both layouts again. The delivering agent inspected
final composition, bird construction, water contact, diagram direction and narrow
reflow. It found no blocking defect for this static brief. This is agent judgment;
the user has not accepted the result. The Mirket illustrations remain rejected.

To challenge review separately, a synthetic copy reversed only the approval path
from `M194 300 V421` to `M194 422 V301`. Both the correct page and incorrect copy
passed all five structural checks. The incorrect copy retained truthful prose and
alt text, so reading labels or accessibility text alone would miss the wrong arrow.
A fresh reviewer received the complete brief, facts and anonymous X/Y pages without
producer notes, prior verdicts or the mutation mapping. Its actual browser review
identified X's arrow pointing from Public journal into Editorial review at both
sizes; Y retained the required direction. This tests one clear semantic defect,
not reliable anatomy review or consistent professional visual quality.

Local evidence is retained in the isolated acceptance checkout under
`build/fieldglass-trial/` (frozen brief, prompt, master, source, before/after captures,
checks and review), `build/fieldglass-negative/` (synthetic copy and actual captures)
and `build/fieldglass-comparison/` (anonymous packages and independent review).
The master SHA-256 is
`01a27ec2bf2d0d2dcab529746fd5f8a60774be528067112c8961dc83245dc4b6`;
correct diagram SHA-256 is
`8296ffd1c0e04c7ee8dc4fb292a6e9fc49c72064b0d3e6ef81dd021d466f53e6`;
reversed diagram SHA-256 is
`4b28643122120d14c0510790b92507a3cc7e78e136c71b1f5eb9d8a40da0d3e5`.
These local artifacts are not bundled with the portable case. A prepared case is an
exercise, not an automatic model runner or an aesthetic certification system.

This is one fresh development artifact and one synthetic negative control, with no
matched baseline run or human quality rating. The exact host backend and image
model version were unavailable. Physical devices, other browser engines,
assistive-technology behavior, animation, scientific anatomical accuracy and broad
transfer to other illustration classes remain unverified. No general visual
instructions were expanded for this exercise.

Verification: all 581 harness tests passed, including 20 focused evaluation tests;
context doctor, generated registry, POSIX shell syntax and whitespace checks passed.
Changed Markdown passed local checks; the new evidence section was rendered and
inspected. Isolated Codex/Claude copy installation and checks passed. The optional
skill-creator validator was not rerun (previously unavailable without PyYAML).
No dependency was installed. Changes were copied into main only after destination
hash conflict checks and verified byte-for-byte; no commit, push or publication.


## 2026-10-10 — repairing product construction communication

Continued one retained Turnstone sharpener failure with the complete product brief.
Inspected both original masters and the [manufacturer's real exterior reference](https://www.moebius-ruppert.com/produkt/logos/).
One native image edit narrowed the blade and exposed the adjoining longitudinal
opening, preserving brass body, swallow engraving, perspective and petrol ground.
The delivering agent inspected 1536×1024 masters and actual 420/342px browser views.
A fresh anonymous M/N reviewer independently preferred the repair for its clearer
cutting relationship at those sizes; no blocking visible defect was observed.

The reviewer also found the original visually plausible: its close-fitting blade
and small clearance resemble the reference. This refines the earlier rejection's
interpretation. The demonstrated gain is clearer communication, not proof that
the original could not function or that the revision is mechanically certified.
Hidden cone, blade penetration and screw support remain unverified. Both versions
are retained. The revision is agent-selected for this fictional static marketing
use, not owner-approved; previous Mirket owner rejections remain in force.

Evidence in the isolated acceptance checkout: `build/sharpener-repair/`, including
complete briefs, exact edit prompt, original/master paths and hashes, anonymous
review, real browser captures and primary reconciliation. Revised master SHA-256:
`79585ab47ce1372a8bc7bcdf1d4d60b837483c88a5eda3fdd39fe8f02ddaded5`.
One generation call; image model version and cost unavailable. No matched baseline,
animation, physical product operation or general visual reliability was established.
No shared workflow or catalog payload was changed for this trial.

All 581 harness tests passed; context doctor, generated registry, shell syntax and
whitespace checks passed. No new dependency, installation, production change,
commit, push or publication. Local artifacts and this evidence update only.


## 2026-10-10 — reconstructing the independent paper bands

The Common Thread bitmap and its image edit remained rejected: colored surfaces
appeared to become ivory at turns, obscuring the three-independent-band requirement.
Changed construction to original native SVG, retaining the full static editorial
brief, three color families, folded paper, open asymmetry and soft directional
shadows. No prior bitmap was traced or embedded. This is a recomposition with a
smoother graphic surface, not a claim of matching the earlier material richness.

The author caught and repaired a red endpoint meeting a blue crease, which made
separate paths look joined. A fresh complete-brief reviewer inspected actual
420/342/960px renders and traced every band and cyclic crossing: red over blue,
ivory over red, blue over ivory. It found the composition ready for those static
placements, with a weak ivory terminal edge. That edge received a restrained
facet; the delivering agent verified the narrow source diff and reinspected all
three sizes. The independent report identifies the preceding source hash; final
refinement acceptance is the delivering agent's judgment, not a claimed new
independent review or owner approval.

Evidence: isolated acceptance checkout `build/paper-reconstruction/`, including
full brief, editable SVG, author/independent notes, first-pass tangency, before-edge
version, exact browser captures and primary reconciliation. Final SVG SHA-256:
`54d27a483ddc297c41ae651c014bc06f7776caa34e3472497ccf4f65cfa9ea36`.
A primary export initially captured the wrong scale/crop; that failed capture is
retained and excluded. Explicit viewport/DPR settings produced the inspected final
960×640 PNG. The comparison page makes the visual tradeoff visible.

The construction now communicates the required independence, by bounded agent
judgment. This does not validate physical paper fabrication, print/vector-editor
imports, animation, broad professional quality or previously failed hand review.
All 581 harness tests and context/generated/shell/whitespace checks passed. SVG
structural audit passed separately from visual review. No shared workflow was
expanded, dependencies installed, products modified, commits made or work published.
