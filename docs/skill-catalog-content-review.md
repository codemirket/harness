# Skill catalog content review

On 2026-10-26, a focused source review inspected the nine global and 58 opt-in
project bodies, the approved catalog design and reference inventory, provenance,
the source-to-release assembler, and the retained sampled project-skill
evaluation. It preserves the accepted catalog shape and native Codex ownership
of agents. This is a content and consistency review, not a completed model
acceptance run.

## Concrete gaps and repairs

The visual and paged bodies referred to a shared budget without defining it in
any installed body. Project selection installs only the selected entrypoints,
so a standalone entry could not recover the intended limit. Sixteen affected
bodies now state the applicable default time and final-repair limit, allow one
target repair, preserve an explicit task budget, and share the largest applicable
budget for combined work. The defaults are 15 minutes/two final repairs for
static, vector and diagram work, 30 minutes/three for interfaces and paged work,
and 45 minutes/four for motion. They bound visual production/review and do not
lower project tests or release requirements. Missing evidence and defects remain
reportable at the limit.

Eight bodies received deeper guidance where an actual design decision was
under-specified:

| Body | Added decision and observable proof |
| --- | --- |
| art-direction | A subject-derived visual idea, focal asset in its real crop, meaningful concept alternatives only when needed, and a medium appropriate to the requested output. |
| interface-composition | Application/public-page composition, content density, hierarchy, explicit reflow, meaningful control behavior and inspection of other affected instances. |
| design-critique | Located visual relationships and user consequences, severity by task effect, matched before/after conditions and disclosed tradeoffs. |
| design-systems | Governing versus observed patterns, the actual token-to-consumer source, migration compatibility and composed state/theme pressure. |
| typography | Real hierarchy/numerals, target-renderer shaping, joining/grapheme-safe animation, fallback metrics and readable settled motion states. |
| motion-direction | Purposeful beats, event/timeline timing, enter/hold/exit/reverse/cancel contracts, logical resting state and honest storyboard-versus-playback evidence. |
| ui-motion | Mechanism chosen from lifecycle, one property owner, semantic visibility, identity/focus continuity, cleanup and reduced-motion completion behavior. |
| scroll-motion | Scene handoffs, scrub versus event replay, document-flow fallback, responsive pinning, anchor/focus reachability and history restoration. |

Eight additional bodies received only the previously undefined budget boundary:
vector-art, raster-art, openpencil, diagrams, svg-lottie, video-production,
document-artifacts and slide-deck-authoring. Their existing operation contracts
remain intact. Global bodies, names, descriptions, capability vocabulary,
profiles and source-family classifications are unchanged.

## Reference and rights boundary

The release catalog already retained approved comparison-source identity, but
those locators were not available in the selected native project entrypoints.
Seven deeper design/motion bodies now link a small number of relevant approved
sources on demand. Repository URLs preserve their recorded revision; the Geist
and GOV.UK typography pages are explicitly moving comparison locators. The
links give a decision-relevant reading route without loading the whole catalog.

The new prose is independently authored. This pass did not fetch or adapt any
external source body and does not claim a source/license audit. Comparison
sources are advisory, not adopted workflow, bundled library, permission to copy
prose/assets/fonts or proof of runtime readiness. Anthropic DOCX and Trail of
Bits remain link-only under the original rights record. Actual operations still
require the qualified Mirket capability or explicitly appropriate host tool.

## Verification and limits

The catalog digests and byte counts are refreshed from saved files. Validation
checks the 9/58 shape, unchanged global bytes/descriptions/IDs/profiles,
frontmatter agreement, reference locators against the approved inventory,
per-body identity and existing source-to-release refusal tests. The four
authoring-basis pins and all 335 public development case IDs remain unchanged.

On macOS 26.5.2 ARM64, `python3 -m unittest tests/test_assemble_skills.py` passed
all four existing tests, including changed/extra bytes, stale authoring pins and
symlink refusals. A separate saved-source inspection passed the shape,
frontmatter, digest, unchanged-field and approved-link checks above; the exact
assembler inventory contained 69 files. Seven bodies contain 15 optional URLs,
all present in the approved reference inventory. The skill-creator
`quick_validate.py` command was attempted but its Python environment lacks
PyYAML; no dependency was installed to enable it. The source checks cover the
actual simple frontmatter form, not arbitrary YAML syntax.

No real Mirket installation, client config,
model/provider setting, dependency, commit, push or publication was changed.
No Rust compilation or installed-client integration was run in this content
pass; those gates are owned by the integrating workstream.

Static consistency does not prove the skills improve decisions, creative
quality, trigger precision or tool use. The current full-matrix status remains
`planned-not-run`. The earlier sampled evidence in
[project-skill evaluation](m4-project-skill-evaluation.md) remains historical;
it exercised the earlier exact bodies and does not qualify the revisions here.
Matched no-skill baselines, held-out/repeated outcomes, the model's own rendered
creative inspection, actual Desktop activation, Windows activation and human
acceptance remain separate gates.
