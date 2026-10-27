# Design-adoption trials — 2026-10-27

## Scope and frozen protocol

This is the held-out follow-up to `docs/research-20261027-design-stack.md`.
The earlier report marked its adoption test proposed/not run; this report records
the bounded runs actually completed. Inputs were frozen before the current-skill
condition was run at:

`/Users/nazmiyilmaz/.codex/task-state/harness-design-adoption-20261027/trials-20261027`

The frozen briefs are in `frozen/cases.json`, with SHA-256
`ca8ddcafc94f8f0b9e02d73ae74128c8c2e6001c5e722782a0a942df570d4126`.
Each condition had one 180-second authoring turn. The task-level protocol
allowed one shared evidence-directed repair budget across
lead/reviewer/delegate/specimen/repair, but no task-level review or repair turn
was actually demonstrated. The retained `initial-manifest.json`,
`events.jsonl`, `stderr.log`, `result.json`, `budget.json`, and project tree are
the source records. This is an adoption/output comparison, not evidence of
repair improvement, and the authoring budget was not reset by changing
specimens or conditions.
The integrity guard is covered by `tests/test_design_adoption_trials.py`:
five tests reject changed prompts, briefs, skill snapshots, and over-budget
timeouts, and verify command/timeout/runtime metadata in a disposable run.

Cases:

1. A distinctive Driftline product page with an interactive route filter.
2. An editable SVG of a person watering a plant, including hand-can grip,
   water-to-soil contact, grounded feet, and a stool.
3. A three-scene 640x360, 12-fps, 3-second Driftline HyperFrames film with
   scene captions and intentional tide/route motion.

Conditions were no-skill, frozen baseline skill snapshot, and current revised
skills. The baseline snapshot is the accepted pre-adoption specimen and the
current snapshot contains complete skill directories, including on-demand
references. The current snapshot hash record is
`frozen/current-skill-hashes.json`; it includes the three reference bodies for
`vector-art`, `motion-direction`, and `video-production`. The runner refuses an
existing root/trial and now requires explicit `--baseline-skills`, `--codex`,
`--config`, and `--codex-home` paths for reproducibility.

## Runtime identity and limits

- Host: macOS 26.5.2, ARM64.
- Codex executable: Desktop-bundled `codex-cli 0.164.0-alpha.11.1`, SHA-256
  `a04752e4d46a4d89e53e6f61c59c5349a638c8ce172ffbf391fd482b4d902a4e`.
- Matched runner settings: `exec --strict-config --ignore-user-config
  --ignore-rules --ephemeral --skip-git-repo-check --sandbox workspace-write
  --color never --json`, with a disposable `HOME`, the real
  `/Users/nazmiyilmaz/.codex` reused as `CODEX_HOME` for authentication, and a
  180-second timeout. The real config sentinel remained unchanged
  (`58519db0c8d95e39a5566f7660867333ab37543a3bfce6e676fc99a3cc66aeb3`).
- The backend model identity and reasoning setting are not exposed in these
  CLI event records. They are recorded as **unknown**, not inferred from
  thread IDs or output. The args, executable, limits, and config sentinel are
  the matched evidence.
- The run command used the explicit private paths
  `--codex /Applications/ChatGPT.app/Contents/Resources/codex-cli/CodexCLI.app/Contents/MacOS/codex`,
  `--config /Users/nazmiyilmaz/.codex/config.toml`, and
  `--codex-home /Users/nazmiyilmaz/.codex`; these are CLI arguments in the
  runner, not hidden defaults. The baseline path is likewise supplied with
  `--baseline-skills` when a new freeze is made.
- Each result record now retains the effective command argv, timeout, host
  platform, Python runtime, executable hash, and config sentinels. Before a
  launch, the runner rechecks the frozen case hash, prompt, BRIEF, budget,
  initial manifest, and assigned baseline/current skill-tree hashes; changed
  inputs are refused. A timeout above the case's frozen 180-second wall budget
  is refused rather than silently expanding the trial.
- `--strict-config --ignore-user-config` prevented a project MCP configuration
  from supplying the qualified Mirket route. This was intentional and matched
  across conditions. It means the video result is authoring-only and does not
  establish runtime compatibility or an MP4 export.
- The model turns themselves do not qualify Windows, the waived Codex Desktop
  GUI check, live installation behavior, or a model-driven approved video
  render. Independent browser observations are recorded below and are not
  attributed to the model turns.

## Retained output comparison

| Case / condition | Wall time | Saved output | Mechanical result | Independent boundary result |
|---|---:|---|---|---|
| Product / no-skill | 143.096 s | `index.html`, `styles.css`, `app.js` | process exit 0; model's filter checks retained | independent browser review passed success/empty/error; 390px had no overflow |
| Product / baseline | 127.046 s | same three files | process exit 0; model's checks retained | independent browser review passed success/empty/error; 390px had no overflow |
| Product / current | 113.851 s | same three files | process exit 0; independent filter/copy/local-URL oracle passed | saved Quick Look pixels viewed; browser rapid sequence ended on North Loop and 390px had no overflow |
| Illustration / no-skill | 129.353 s | editable SVG, full/detail PNGs | process exit 0; palette/group checks retained | saved full image independently viewed |
| Illustration / baseline | 111.847 s | editable SVG, full/detail PNGs | process exit 0; palette/group checks retained | saved full image independently viewed |
| Illustration / current | 146.521 s | editable SVG, full/small/detail/feet PNGs | process exit 0; independent SVG oracle passed | full and feet detail viewed; grip, pour/contact, stool and grounded feet visible |
| Video / no-skill | 83.478 s | local HTML/CSS/JS and captions | process exit 0; source checks retained | no runtime or pixels |
| Video / baseline | 104.673 s | local editable film and captions | process exit 0; source checks retained | no runtime or pixels |
| Video / current | 102.437 s | local HTML/CSS/JS, captions and notes | process exit 0; independent syntax/caption/local-source oracle passed | no runtime or decoded pixels; no MP4 |

The current product render is retained at
`product-page-current/project/index.html.png`; the current illustration is
retained at `illustration-current/project/preview.png` and
`detail-feet.png`. The saved pixels were opened with the image-capable local
viewer, not inferred from absent/present JSONL image events. Current product
behavior was independently exercised with a DOM stub for line selection,
empty/error/reset/focus states, required rendered copy, and no external URLs.
Current SVG was independently parsed for semantic editable groups, supplied
palette only, no embedded images, and saved render files. Current film was
independently checked for syntax, all 36 caption frames, local references,
absence of network imports, and scene-motion identifiers.

The independent browser review additionally exercised the three UI specimens'
route-filter success, empty, and error states. At 390px, baseline and no-skill
screenshots showed no horizontal overflow. The current specimen's rapid
Harbor → Orchard → North Loop changes ended on the correct North Loop result,
and its 390px view also had no overflow. These are browser observations from
the independent reviewer, separate from model self-report; they do not qualify
the video route or Windows/Desktop GUI paths.

## Findings and interpretation

1. The current skills produce complete, inspectable page and illustration
   specimens in these turns. This is an observed outcome, not a causal quality
   claim: the baseline/no-skill controls inherit the same Codex environment,
   and some JSONL records expose global/inherited skill names even in the
   nominal no-skill condition. No clean causal uplift is claimed.
2. The revised vector skill's reference was actually loaded by the current
   illustration turn, and the output retained semantic groups plus a feet
   detail. An independent lead inspected all three saved full PNGs; each shows
   the watering action, soil contact, hand-can grip, and grounded feet, with
   current also retaining an explicit finger detail. This supports the intended
   craft method, but no consistent quality uplift or reviewer agreement is
   established.
3. The revised motion/video resources were loaded by the current film turn,
   and the source has a deterministic 36-frame caption contract. Neither
   authoring success nor a tool receipt proves playback, intended-object motion,
   or export. The model correctly preserved source and reported the unavailable
   Mirket route; this is a concrete qualification limit, not a renderer result.
4. The product page's static image is useful visual evidence, but Quick Look is
   not a browser. The independent browser review covered success/empty/error,
   rapid final-selection correctness, and 390px overflow; focus-visible
   screenshots, keyboard traversal, and the complete state matrix remain
   unqualified.
5. Raw traces show concrete out-of-project temporary writes: baseline item 14
   redirected `/private/tmp/driftline-headless-render.log` and exited 133 before
   item 16 read the actual crash log; earlier baseline and no-skill turns also
   redirected `local-render.log` and `driftline-render.log`. These are scope
   defects even though final project manifests remain readable. The traces also
   expose inherited installed-skill contamination during nominal no-skill
   turns, so the controls are not clean causal baselines. A prior creative
   retry writing outside its requested project remains a separate historical
   defect.

## Separate qualified video evidence

The direct, qualified runtime controls retained under
`~/.codex/task-state/harness-deep-review-20261027/video-{reedit,captions}-fixture`
are separate from this model trial. A viewed reedited MP4 changed 3,434
decoded heading pixels; a viewed caption PNG changed 11,993 pixels during the
cue and zero afterward. Those checks demonstrate runtime output controls, not
that a model-driven trial exported successfully or bypassed host approval.
The earlier model-driven video request was blocked by host approval; this run
was intentionally not relabeled as a successful export.

## Remaining unqualified paths

- Comparable-capability independent reviewer and one shared repair turn were
  not run in this bounded comparison; no repair claim is made.
- Product page focus-visible screenshots, keyboard traversal, and the full
  interaction-state matrix remain unqualified; the independently observed
  browser states and 390px overflow checks are recorded above.
- HyperFrames playback, decoded frames, MP4 export, and approval/permission
  recovery need the qualified Mirket route and explicit host approval.
- Windows packaging/capability fidelity and the waived Desktop GUI check remain
  unverified.
