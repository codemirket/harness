# Native skill evaluation contract

The current catalog contains nine global skills and 58 opt-in project skills.
Each has a distinct trigger, output/evidence boundary and five authored
development cases: two positive, two negative and one mixed. These 335 cases
are embedded in [catalog.json](../skills/catalog.json), not a separate test
fixture. The [provenance inventory](../skills/PROVENANCE.md) identifies local
contracts and comparison-only external families. No external prose, helper or
asset was adopted by the native skill bodies.

## Offline gate

Run from the repository root after the Rust build-input guard:

```sh
python3 scripts/check-build-inputs.py
cargo test --locked --offline native_skills::tests --lib
```

The release catalog loader checks the 9/58 skill and eight-profile shape,
canonical IDs and paths, frontmatter, source byte counts and SHA-256 identities,
declared capability vocabulary, profile references, provenance records and exact
five-case mix. The focused unit tests include invalid IDs and changed
frontmatter. The release assembler also verifies that the four pinned local
authoring sources still match their catalog hashes; its 69-file inventory and a
stale-pin refusal were exercised on a disposable copy. This gate starts no model, installs nothing and does not execute
upstream helpers. It does not grade whether the prose or expected outcomes are
good, and it does not check every local Markdown link.

A passing result establishes corpus consistency only. It cannot grade semantic
correctness, activation, source rights, user acceptance, actual calls, latency or
client behavior. A matching hash proves identity, not that a trial ran. The public
cases are development examples and must not be called blind or held-out evidence.

The exact native skill copies passed a disposable Codex app-server discovery
journey on macOS ARM64: one global and one project skill were discovered from
their installed native locations after full setup, and again after two full
updates. The installed SQL capability also allowed opt-in `sql-analysis` to be
planned, selected and diagnosed in a disposable project. See the
[installation qualification](codex-native-installation-qualification.md). This
establishes activation and receipt integrity for those sampled skills, not
model adherence or visual/semantic output quality. The user's real Codex
configuration and `.mirket` home were untouched.

Later [global-skill](m3-global-skill-evaluation.md) and
[project-skill](m4-project-skill-evaluation.md) developmental runs exercised
actual Codex CLI model turns. They observed all nine global body reads across
eighteen turns and two selected project body reads across four turns, with
independently checked saved-output examples and retained failures. These are
sampled public cases; they do not establish matched held-out benefit, the full
58-skill matrix or Desktop GUI behavior.

A later [fresh sampled comparison](m4-skill-acceptance-sample-20261027.md)
matched `test-design` and `threat-model` against no-guide conditions, checked
saved tests against a separate oracle, exercised one trivial nontrigger across
all 67 skill bodies in seven bounded turns,
observed `video-production` form a revision-bound installed-runtime intent, and
compared editable `art-direction` poster outputs. All four art-direction model
turns reported visual inspection, but their JSONL omitted image-view items.
Separate image-only and missing-file controls show that the CLI can access
image content while omitting those items, so that absence cannot establish a
false claim. One guide retry did write a render outside its requested project;
that scope defect remains. The host denied the Video trial's `video_prepare`
effect under the `never` approval policy, so
model-driven Video output remains untested. The sample does not complete this
catalog-wide acceptance gate.

A further [installed vector-art turn](m4-codex-design-skill-tool-qualification.md)
read the exact native skill and verified the Mirket runtime, but Codex refused
the first design write because the host approval policy was `never`. Its
independent installed SVG render is a tool control, not a completed model
skill-to-artifact result. That refusal is retained as a native-client gate.

## Codex CLI acceptance gate — pending

Use the installed Codex CLI model path in disposable projects for the first
release's skill outcome checks. The user waived the separate Desktop GUI test
on 2026-10-27 because the host computer-use tool refuses control of the app.
Do not substitute a protocol smoke, app-server call, guide read or receipt for
an observed model turn and checked artifact. Keep the user's installation and
settings unchanged. Upstream evaluation helpers are not selected by this gate.

Before running acceptance, freeze fresh cases and their expected observable
outcomes outside the worker's write scope. Screen every skill for static integrity,
native activation and representative nontrigger behavior; the five authored cases
per skill are development examples, not held-out trials. Prioritize deeper matched
outcome comparisons for effect-bearing, security, memory, agent-coordination,
design/video and analytical-quality skills; expand when screening exposes a defect.
Keep public tuning cases separate. Compare shared workflow alone against shared
workflow plus the selected guide in clean isolated contexts. For shared workflow
itself, assess its authority, discovery and recovery cases without pretending it
is an additional specialist guide.

Record the following for every planned trial, including failures and unavailable
runs:

- Case identity, frozen input and criterion hashes; guide/corpus/release hashes;
  public-development or held-out split; contamination status and reason.
- Client version, OS/architecture, available host/model configuration identity,
  relevant settings, tool access, initial state and bounded trial budget. Mark
  opaque provider/model versions unknown; do not infer an internal model revision.
- Condition, repetition index and actual guide-read evidence. Capture loaded UTF-8
  bytes, observed calls, unnecessary calls/questions and elapsed time. Unknown
  token usage, costs or timing remain unknown, never zero by default.
- Saved artifacts/effects and their identities, criterion verdicts and the
  independent check or review supporting each verdict. Preserve the failing
  output and refusal cases; worker self-report is insufficient.

For image viewing, first calibrate what the selected client trace exposes.
The current Codex CLI JSONL omitted `view_image` items even in controls that
read image-only content and refused a missing image. Do not infer that no view
occurred solely from the absence of a JSONL tool item; retain a discriminating
image control and independently inspect the saved artifact.

Compare only matched case/client/model/settings/tool/budget conditions. Exclude
contaminated or unmatched pairs from comparative claims while retaining their
records. Use deterministic artifact checks where possible; have an independent
reviewer assess remaining qualitative criteria with the frozen rubric. Record
reviewer identity/role and limitations without claiming human approval from an
agent review. Repeat variable cases when their variance could change the
promotion decision; no universal repetition count or new judge service is needed.

Promotion requires the requested task result and applicable boundaries to pass,
not merely a guide load or increased call count. Treat unauthorized effects,
source loss, false verification and unknown-effect replay as failures, regardless
of an aggregate score. Check keyboard/focus behavior for interfaces, actual saved
Office recalculation/rendering, and analytical validity such as dependent
observations and join grain. Report per-case regressions and unavailable paths
alongside correctness, unnecessary calls/questions, bytes and time.

The offline gate does not complete M4. CLI model comparisons, held-out outcome
assessment and guide-benefit measurements remain open until their evidence is
recorded under this contract. Desktop GUI behavior remains explicitly untested.
Local semantic/design qualification cited by the guides applies only to its
documented package, platform and fixtures.
