# Motion and vector-agent research (2026-10-27)

This is a read-only research pass for the Codex/Mirket boundary. It looks for
open-source projects and agent methods that improve vector illustration,
interactive motion, or visual verification. Hallmark/Taste remain retained
choices, and HyperFrames remains the selected video renderer. Nothing here was
installed, executed, added to the Mirket runtime, or added to the catalog.

## How the evidence was collected

GitHub REST metadata and pinned default-branch content were fetched on
2026-10-27. The numbers below are a snapshot, not a quality score: stars and
forks are cumulative popularity; `pushed_at` and the latest commit are the
maintenance/activity signals. No clone, installer, upstream test, render, or
independent quality trial was run. Claims made by a repository (for example,
its own benchmark or example quality) are labelled as such and are not treated
as qualification evidence.

The exact pinned sources and local SHA-256 receipts are retained privately at
`/Users/nazmiyilmaz/.codex/task-state/harness-deep-review-20261027/motion-vector`.
The private receipts are not shipped. A source SHA below identifies the exact
body inspected, while the linked GitHub URL identifies the public primary
source and revision.

## Ranked candidates

| Rank | Project | Adoption/activity snapshot | License boundary | Best integration mode | Decision |
|---|---|---|---|---|---|
| 1 | [OpenPencil](https://github.com/open-pencil/open-pencil/tree/fba4995834b9cd843aeb3bdc637cfaa3e7e7f39d) | 9,317 stars / 969 forks; pushed 2026-10-27; latest commit `fba4995` | MIT repository; the README separately identifies an Apache-2.0 OpenPencil Cloud exception | Existing Mirket design route / host-owned tool | Retain and qualify the existing route; no demonstrated gap justifies another vector engine |
| 2 | [MengTo/Skills](https://github.com/MengTo/Skills/tree/a45892d2fdadab75b31fb16abecc02f101b31c43) (illustration) | 7,060 / 824; pushed 2026-10-25; latest `a45892d` | Root MIT; inspect any bundled fonts/assets separately before redistribution | Reference/native project skill | Strongest new artful-illustration reference; adapt methods, not private paths or unverified assets |
| 3 | [Penpot MCP](https://github.com/penpot/penpot/tree/29e79edc667417bd92489302aeeb29dbe3b58707/mcp) | 61,747 / 4,300 for the Penpot project; pushed 2026-10-27; latest `29e79ed` | MPL-2.0 | Host-owned external project MCP / reference | Useful external capability; keep plugin, browser, session, and file ownership outside Mirket |
| 4 | [AgentsORG/design-engineering](https://github.com/AgentsORG/design-engineering/tree/1b69b8cc4484874fcb019cca67263f886c165148) | 35 / 2; pushed 2026-10-27; latest `1b69b8c` | MIT source, with separately credited third-party methods/resources | Reference/native routing and review skill | Good compact routing/eval ideas; avoid copying the whole router or duplicating HyperFrames |
| 5 | [Motion Canvas](https://github.com/motion-canvas/motion-canvas/tree/7b91435c301d530351dcf5ebb91dd139c002e405) | 19,506 / 863; pushed 2026-07-02; latest `7b91435` | MIT | Project dependency comparator | Mature TS/vector animation option, but not an agent/MCP layer and not a HyperFrames replacement |
| 6 | [LangChain design-agent](https://github.com/langchain-ai/design-agent/tree/0088a8d61b3d550bd1656c2cf0b379593a4483b7) | 6 / 0; pushed 2026-10-26; latest `0088a8d` | Apache-2.0; hosted/managed deployment has separate service assumptions | Reference/evaluation method | Valuable visual-verification and brand-token patterns; too new and too broad to admit as a runtime |
| 7 | [Excalimate](https://github.com/excalimate/excalimate/tree/e12ec8001148d09c750badf71d3da4c97fb3fbbb) | 98 / 14; pushed 2026-10-27; latest `e12ec80` | MIT | Optional project-level explainer/MCP tool | Promising for diagrams and managed action sequences; not general illustration or Mirket video runtime |
| 8 | [SVG Creator Skill](https://github.com/upbrew-tech/svg-creator-skill/tree/1eb83d602a992b69a1771c4e0116a9f7de6ffaa2) | 39 / 1; pushed 2026-07-29; latest `1eb83d6` | Apache-2.0 | Reference/native project skill | Direct SVG craft and render/fix loop; low adoption and no independent quality evidence |

### 1. OpenPencil: keep the existing design route

The pinned README describes `.fig`/`.pen` read/write, 100+ built-in AI
operations, a headless CLI, XPath/Figma Plugin API, evaluation hooks, MCP,
components/variants, vectorization, lint/convert/token extraction, and code
export. The MCP can inspect, modify, and export headlessly; the README also
documents `OPENPENCIL_MCP_ROOT` as a one-directory access boundary. Those are
real agent affordances, but they are not proof that every host binding or
export works in this release. The current qualification should exercise
capability identity, wrong-project rejection, permissions, and an actual saved
export.

Primary evidence: [README at the pinned commit](https://github.com/open-pencil/open-pencil/blob/fba4995834b9cd843aeb3bdc637cfaa3e7e7f39d/README.md)
(private SHA-256 `471add1dcbe88d1fb5892f47e08cb1548cb494a87f0146e166a3381a588453a8`)
and [license](https://github.com/open-pencil/open-pencil/blob/fba4995834b9cd843aeb3bdc637cfaa3e7e7f39d/LICENSE)
(private SHA-256 `38ac54d3b48ccb6b3c0044b37bec1d75c21c3dc38b34a5a8c52a3cea34ba514b`).

Integration finding: this is already an admitted Mirket-owned design pairing.
The research found no demonstrated gap that justifies another vector engine;
that conclusion is evidence-based rather than an assumption that every
illustration or raster workflow is the same as OpenPencil. The Cloud Apache
exception and any host-specific executable binding need to stay in their
existing qualification/provenance paths.

### 2. MengTo/Skills illustration: strongest artful illustration method

The illustration README describes 14 hand-authored SVG styles, three examples
per style, 42 original cards, and a fresh-subject holdout after each skill. Its
method is unusually concrete: each style has a `SKILL.md`, `style.json`,
examples, and craft references; `render.mjs` creates a 2x whole-card render,
re-renders a zoom region from the vector, and makes a sheet; `lint.mjs` checks
viewBox, ID prefixes, absence of `<style>`/`<image>`, off-palette colors, and
other structural faults. The framed-panel and flat skills require a brief,
pose/hands-first construction, deterministic scaffolding, 1x block-in, 2x
render, side-by-side comparison, 5–8x hand/face/feet/contact inspection,
critique, repair, and lint. The iterate skill locks the contract, makes gates
observable, separates author and blind reviewer, matches proof to visual work,
and repeats until all gates pass or unknowns are disclosed.

This directly maps to a Mirket skill’s trigger (specific illustration style),
decision method (choose one style and follow its deterministic kit), output
(SVG plus rendered sheet/zoom evidence), and evidence rule (saved pixels plus
lint; never a self-rating). It is a reference method, not a reason to install
the repository’s private paths or to claim its reported 8/10 blind-judge result
as an independent Mirket result.

I also viewed the repository’s pinned `illustration-framed-panel/demo/preview.jpg`
(1280×720, private SHA-256
`9477f0fe7be47b1f4f40fa7dbf010d339aa70a18eb7fee2b61562b278535d3dc`). The
three showcase cards visibly share a violet framed panel, thin navy outlines,
pastel card backgrounds, dashed floor bands, small cloud/leaf accents, and
white UI callouts that break across the frame. At this displayed size the
characters’ silhouettes, props, contact with the floor/chair, and the breakout
hierarchy read cleanly; the sheet also exposes style consistency across
support, design-system, and live-coding subjects. This is visual inspection of
the upstream showcase only—not a Mirket render, a holdout, or proof that every
fine hand/face junction survives export. The required zoom checks remain
necessary.

Primary evidence: [illustration README](https://github.com/MengTo/Skills/blob/a45892d2fdadab75b31fb16abecc02f101b31c43/agent-skills/illustration/README.md)
(private SHA-256 `706c3dfdea6bdf133315da251f904bca7cc114e9f73d8279050f0eab99f339c1`),
[framed-panel skill](https://github.com/MengTo/Skills/blob/a45892d2fdadab75b31fb16abecc02f101b31c43/agent-skills/illustration/illustration-framed-panel/SKILL.md)
(private SHA-256 `b5333635a651587a9b4bdc8d75f82f2f510257bff280ad567248a3a30d502102`),
[flat skill](https://github.com/MengTo/Skills/blob/a45892d2fdadab75b31fb16abecc02f101b31c43/agent-skills/illustration/illustration-flat/SKILL.md)
(private SHA-256 `a9b06cd733a2e5f8429747f12816057377faf2ecbf55940ea23d213577c30744`),
[iterate skill](https://github.com/MengTo/Skills/blob/a45892d2fdadab75b31fb16abecc02f101b31c43/agent-skills/codex/iterate-until-verified/SKILL.md)
(private SHA-256 `ca1ea516ab3a32c40e3e2c5b05251e4696f1d4674cf80cf4e0d2dc1012e65188`),
and [MIT license](https://github.com/MengTo/Skills/blob/a45892d2fdadab75b31fb16abecc02f101b31c43/LICENSE)
(private SHA-256 `b2a071ba22fefd803f4e9d1f5c777dd79c62d191695586a1a73fd4d0ebd051c6`).

Integration finding: adapt the evidence pattern to a narrowly scoped project
illustration skill only if catalog admission and source-to-skill provenance are
approved. Reuse the existing Mirket SVGO/resvg vector-rendering/optimization
path and Codex’s authoring and image-capable inspection; do not introduce a
second rendering engine. Repository examples
and any fonts/assets require their own rights review before shipping.

### 3. Penpot MCP: capable external editor, not Mirket-owned runtime

Penpot’s MCP documentation describes a Plugin API/WebSocket server that can
query, transform, and create design data. The tool surface includes
`execute_code`, high-level overview, API information, shape export, and image
import; the docs also describe creating, renaming, moving, deleting, and
restyling objects. The important operational limits are equally explicit: the
MCP server, plugin server, open Penpot file, and active plugin/tab must all be
present; browser suspension can reject work; sessions/tabs need explicit
identity; and remote mode cannot use local import/export in the same way.

Primary evidence: [MCP README at the pinned source](https://github.com/penpot/penpot/blob/29e79edc667417bd92489302aeeb29dbe3b58707/mcp/README.md)
(private SHA-256 `f34970a33134b7abb995791c08d04beebea724f05799fa16e9fb11b9e7d20984`),
[MCP docs](https://help.penpot.app/mcp/), and [MPL-2.0 license](https://github.com/penpot/penpot/blob/29e79edc667417bd92489302aeeb29dbe3b58707/LICENSE)
(private SHA-256 `1f256ecad192880510e84ad60474eab7589218784b9a50bc7ceee34c2b91f1d5`).

Integration finding: treat it as a Codex/host-owned external project tool or
research comparator. Mirket must not promise Penpot operations from a receipt
or tool listing without an active plugin/session and a saved-result check. Do
not add a duplicate native design tool.

### 4. AgentsORG/design-engineering: compact routing and review ideas

The design-engineering skill routes by phase and design contract before
selecting an owner, explicitly limiting work to one or two owners rather than
five. Its motion cluster states when not to animate, and its SVG/animation
agents are narrow: SVG creation, SVG animation/morphing/vectorization, and a
launch-film analyst that measures native-fps film material and returns rubric
and per-film JSON. The README’s launch-video split is useful for the existing
boundary: HyperFrames builds while the design skill chooses beats, words,
sound, and reconciles the result. Its evaluation notes test routing/retrieval
with loading/progressive-read JSONL and `expected_nodes`/`forbidden_nodes`;
they explicitly do not establish content correctness.

Primary evidence: [skill](https://github.com/AgentsORG/design-engineering/blob/1b69b8cc4484874fcb019cca67263f886c165148/skills/design-engineering/SKILL.md)
(private SHA-256 `a56273bc0feb9e4c665b53454eb04c9aee27dd74121e4a3d18ef8e3fbcd161d4`),
[README](https://github.com/AgentsORG/design-engineering/blob/1b69b8cc4484874fcb019cca67263f886c165148/README.md)
(private SHA-256 `468b19867b9475817e467da74c86cff63cb16a7497190867df17ba619c684948`),
[evaluation notes](https://github.com/AgentsORG/design-engineering/blob/1b69b8cc4484874fcb019cca67263f886c165148/agentsorg-evals.md)
(private SHA-256 `6fde718522c1d44c89f594ce1762b15eb0eca8c8ccd8ebd47abbaafb06de3ffd`),
and [MIT license](https://github.com/AgentsORG/design-engineering/blob/1b69b8cc4484874fcb019cca67263f886c165148/LICENSE)
(private SHA-256 `2b8475776d6f6782842436d231739ca2195634cff2d97bf9afa56933c05372b7`).

Integration finding: the router/eval concepts are suitable references for
existing Mirket skills and the capability matrix. Do not import the entire
agent tree: that would bloat the catalog and duplicate the host-owned agent
coordination boundary. The repository credits third-party methods/resources;
MIT for its own files is not a grant to those third parties’ material.

### 5. Motion Canvas: mature project dependency, not agent orchestration

Motion Canvas is a TypeScript library/editor with generator-based timelines,
real-time preview, vector-focused explanatory animation, and voice-over sync.
The monorepo exposes 2D/core/create/player/UI/Vite packages and examples under
MIT. Its 19.5k stars indicate substantial cumulative adoption, but its latest
public commit in this snapshot is July 2, 2026, materially less current than
OpenPencil, Penpot, MengTo, or Excalimate. No built-in MCP or agent contract
was found in the inspected README.

Primary evidence: [README at the pinned commit](https://github.com/motion-canvas/motion-canvas/blob/7b91435c301d530351dcf5ebb91dd139c002e405/README.md)
(private SHA-256 `9895daa20be79849a312b5cb5b9ed36db2d1a6dfe9731b5e53a67b222abe9cd9`)
and [MIT license](https://github.com/motion-canvas/motion-canvas/blob/7b91435c301d530351dcf5ebb91dd139c002e405/LICENSE)
(private SHA-256 `b3f12201916c5d478263cd210cf17f3d0c007d03bd574e0b77cac6307953068e`).

Integration finding: use only as a project dependency comparator where a
project explicitly chooses it. It does not improve Mirket’s host-owned agent
coordination and must not replace HyperFrames on popularity alone.

### 6. LangChain design-agent: visual-verification method, not a runtime admission

The design-agent README describes a new Content Studio agent with brand tokens,
skills, Paper/MCP integration, sandboxed threads, asynchronous subagents,
render/check steps, and HyperFrames for video. Its visual-verification skill
requires inspecting the final raster, comparing source/output, using thumbs and
crops, and examining a video filmstrip rather than one frame. This is a useful
independent-evidence rule and aligns with the Mirket requirement to view saved
pixels. The project is only six stars and was pushed the day before this
snapshot, so it is a new method signal, not adoption evidence. Hosted Deep
Agents, Slack, Docker, and managed service assumptions are outside the current
Mirket boundary.

Primary evidence: [README](https://github.com/langchain-ai/design-agent/blob/0088a8d61b3d550bd1656c2cf0b379593a4483b7/README.md)
(private SHA-256 `b6fe9939fa0206904fcd85a8d070ecdb8dd071cbaad295d87407687d3ee15185`),
[visual verification skill](https://github.com/langchain-ai/design-agent/blob/0088a8d61b3d550bd1656c2cf0b379593a4483b7/skills/visual-verify/SKILL.md)
(private SHA-256 `d1dc42a397c316ff67dbdf264662773f0794372c103a831f868f94bd2244f7e5`),
and [Apache-2.0 license](https://github.com/langchain-ai/design-agent/blob/0088a8d61b3d550bd1656c2cf0b379593a4483b7/LICENSE)
(private SHA-256 `21c74324584ca851e4ca58a53dda3936a18ec8799506f82c97b35898d16ff043`).

Integration finding: borrow the raster/filmstrip verification rubric as a
research input to feedback-loop tests. Do not add its hosted agent stack or
another orchestrator; Mirket cannot intercept Codex native spawning.

### 7. Excalimate: useful managed animation actions for explainers

Excalimate’s MCP v2 documents bounded, schema-validated, sanitized snapshot and
delta data, a loopback authoring companion, and managed actions such as
`auto_animate`, `apply_animation_preset`, and `upsert_action_sequence`. Its
skills recommend managed actions over raw keyframes, inspect existing action
sequences before editing, set clip ranges, and verify via
`animations_of_item`. The explainer skill adds one concept per step,
progressive callouts, pacing/read-time formulas, and a final summary/camera
check. This is a strong concrete output/evidence pattern for diagrams.

Primary evidence: [README](https://github.com/excalimate/excalimate/blob/e12ec8001148d09c750badf71d3da4c97fb3fbbb/README.md)
(private SHA-256 `a13210ee0817e6ac0b888a1e1b5d434424aa8220ed4a77ce824610e5eee982df`),
[MCP v2](https://github.com/excalimate/excalimate/blob/e12ec8001148d09c750badf71d3da4c97fb3fbbb/docs/mcp-v2.md)
(private SHA-256 `f52ce0714dc20e62ea694eb9c9b82269cf9e7b3860821fa9ae5d4fc04145f545`),
[explainer skill](https://github.com/excalimate/excalimate/blob/e12ec8001148d09c750badf71d3da4c97fb3fbbb/skills/explainer-animations/SKILL.md)
(private SHA-256 `9b18ac561a4e0e23bb81d9ec25f2a58b1af43852be6c2e85fccaaa3672bbfb19`),
[animation patterns](https://github.com/excalimate/excalimate/blob/e12ec8001148d09c750badf71d3da4c97fb3fbbb/skills/animation-patterns/SKILL.md)
(private SHA-256 `3c01b0d6b24de308c1cffc108a10efb0db817dae2464cd7caae081c91f38d8f5`),
and [MIT license](https://github.com/excalimate/excalimate/blob/e12ec8001148d09c750badf71d3da4c97fb3fbbb/LICENSE)
(private SHA-256 `a04d0ba5bde941fe20af5925d7e5c1e83408123c1a2dca4c77d224588b9fd51b`).

Integration finding: optional project-level tool for explainer diagrams only.
Its 98 stars are low adoption, and the README/MCP contract does not establish
general illustration quality or an authenticated server-to-server write path.
Do not add it as a Mirket video runtime or claim a managed action succeeded from
a receipt alone; inspect the saved sequence and decoded export.

### 8. SVG Creator Skill: direct SVG craft reference

The Apache-2.0 skill targets production SVG illustrations, characters, and
animations. Its method emphasizes construction, gradients/five-zone lighting,
character geometry, and a render/verify/fix loop with multiple iterations. The
method is directly useful for project-scoped vector work, but the repository
has only 39 stars, an older July push, and no independent quality evidence in
this pass.

Primary evidence: [README](https://github.com/upbrew-tech/svg-creator-skill/blob/1eb83d602a992b69a1771c4e0116a9f7de6ffaa2/README.md)
(private SHA-256 `9fc6639de8372198f2f831712ee6a7871ffde465ed4ad2bcb7404e8004fe9ab5`),
[skill body](https://github.com/upbrew-tech/svg-creator-skill/blob/1eb83d602a992b69a1771c4e0116a9f7de6ffaa2/skills/svg-creator/SKILL.md)
(private SHA-256 `9c01d512f9b5d2041743097ac5bb8489c38ff03d479d1f90bf21c17c94cf1101`),
and [Apache-2.0 license](https://github.com/upbrew-tech/svg-creator-skill/blob/1eb83d602a992b69a1771c4e0116a9f7de6ffaa2/LICENSE)
(private SHA-256 `82d01b6a89aeeea4d366bb1da52277b43654519baf7e5154cce9dceb8d2d9ced`).

Integration finding: reference/native project skill only. The render/fix rule
is worth comparing with MengTo’s stronger zoom/fixture evidence, but no new
Mirket renderer is justified.

## Investigated but not admitted

- [Logo Design Skill](https://github.com/kaankiziltug/logo-design-skill/tree/734090317a2155c5bf1ae8bd4cc85c6848a0128f) has unusually high current adoption (4,779 stars / 363 forks; pushed 2026-10-20) and a strong brief → concepts → SVG → small-size/one-colour/reversed/shelf tests → checkpoint workflow. It is MIT for skill text/scripts/catalog, but the 1,400+ library SVGs are explicitly third-party trademarks and **not** covered by MIT. It is therefore a reference for methods only; never ship or copy those assets without per-asset rights. Inspected [README](https://github.com/kaankiziltug/logo-design-skill/blob/734090317a2155c5bf1ae8bd4cc85c6848a0128f/README.md) (private SHA-256 `e513d849125861c99e28096e6f48e0487a8de8d4dea7acfd7867ea6b122fdf93`), [skill](https://github.com/kaankiziltug/logo-design-skill/blob/734090317a2155c5bf1ae8bd4cc85c6848a0128f/skills/logo-design/SKILL.md) (private SHA-256 `65ec4bd58732e8cc11e244724e8666de78764b973f6d3b03b233904639cf47c6`), and [license](https://github.com/kaankiziltug/logo-design-skill/blob/734090317a2155c5bf1ae8bd4cc85c6848a0128f/LICENSE) (private SHA-256 `f68cc121d7f856ec3f48993c5082960f267bd752acea4e2f694408ef55d6cfdd`).
- [Paper agent plugins](https://github.com/paper-design/agent-plugins/tree/a8ba8b21be02f68c83470213bc72ea43cf3b6d1f) expose useful structure/style/JSX/HTML editing and design↔code affordances, but the inspected repository has no declared license and the plugin package is private. Keep it link-only/unqualified until rights are clear.
- [Doop](https://github.com/kgoedecke/doop/tree/b28219a779169b53303aec48e8cd45c0e555e042) has a compelling live HTML canvas, approval-bound revisions, screenshots, and self-review, but AGPL-3.0 plus a server/browser/database stack is outside the Mirket boundary. It is a comparison, not an admission.
- [Theatre.js](https://github.com/theatre-js/theatre/tree/6ea82b938ea49609489f6377ded693ccc6ee8f5b) has strong web motion design concepts and 12.8k stars, but public activity is stale and the README says 1.0 development moved to a private repository; its Studio package also has an AGPL-3.0 boundary. Do not use its popularity as current qualification evidence.
- [Rive MCP](https://github.com/ODU33104/rive-mcp/tree/09d916e0aff491bff42ff0c6fc3a5e32b236a639) is explicitly freeware/source-available, not open source, with restrictions on modification, redistribution, reverse engineering, and AI-assisted analysis. Its strong MCP surface is not a licensing reason to adopt it.
- `transitions.dev/transitions` and `riffcast/riffcast` did not resolve as public GitHub repositories in the API pass; they are not candidates on this evidence.

## Resulting research recommendations

1. Use MengTo’s illustration contract as the primary new reference for artful
   SVG: style-specific triggers, deterministic scaffolding, fixed examples,
   zoomed saved-pixel checks, lint, and an independent reviewer. Treat the
   reported quality score as an unverified upstream claim until a Mirket
   holdout trial exists.
2. Keep OpenPencil as the admitted design route and Penpot as a host-owned
   external option. Receipts/tool listings must never substitute for a live
   session and saved-output check.
3. Feed AgentsORG and LangChain’s routing/visual-verification patterns into
   the capability/evidence matrix, while keeping one Codex-owned orchestrator
   and HyperFrames as the video owner.
4. Consider Excalimate only for an explicitly selected diagram project. Its
   managed-action and bounded-snapshot contracts are useful, but its adoption
   and scope do not justify Mirket runtime work.
5. Preserve the rights gates: MIT/Apache/MPL labels apply only to the covered
   source. Third-party logos, fonts, examples, Cloud/hosted services, AGPL
   components, and source-available tools require separate admission or remain
   unqualified.
