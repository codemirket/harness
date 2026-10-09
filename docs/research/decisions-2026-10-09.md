# A personal harness built around finished work

Fresh research, 9 October 2026. The request was to improve actual UI, animation,
illustration, documents, programming and QA. The decision is a small executable
workbench with craft workflows and independent review, shared by Codex Desktop
and Claude Desktop Code. Keep the proven installer as distribution plumbing.
Do not make catalog size the product.

## What was investigated

Three source reviews inspect 25 candidates, including code, helpers, test behavior,
licenses, runtime requirements and limitations:

- [Visual craft](visual-capabilities-2026-10-09.md): Impeccable, Interface Design,
  Emil Kowalski, Taste, Motion, GSAP, Remotion, vector tools and Figma.
- [Engineering harnesses](engineering-harnesses-2026-10-09.md): Superpowers,
  Gstack, agent-browser, Playwright/MCP, Compound Engineering, GSD and ECC.
- [Documents and assets](document-capabilities-2026-10-09.md): provider skills,
  OOXML authoring libraries, recalculation, rendering and portable runtimes.

These reports link exact inspected revisions and primary documentation. Upstream
implementations were source-reviewed; their entire runtimes were not installed
or benchmarked. Recommendations are judgments from that evidence.

Anthropic's [long-running app harness experiment](https://www.anthropic.com/engineering/harness-design-long-running-apps)
found self-grading lenient and used a separate evaluator with browser interaction.
It also reports that later iterations can overcomplicate a design. That supports
fresh inspection and bounded repair, not compulsory long agent rituals.
The revised [SkillsBench paper](https://arxiv.org/abs/2602.12670) evaluates supplied
skills under matched tasks; its aggregate results do not establish improvement
for this personal harness. We need comparable local trials before such a claim.

## Decisions implemented

| Need | Selected mechanism | Rejected default |
| --- | --- | --- |
| UI design | One lead, context-specific reference, actual stateful screen, rendered critique | Combining multiple aesthetic instruction systems |
| Motion | Existing CSS/WAAPI/project engine, interrupted/reduced-motion checks, sampled frames | Forced GSAP or a static snapshot as motion acceptance |
| Vector | Editable semantic geometry, sharp renders at multiple sizes/backgrounds | Raster wrapped in SVG or XML validity as visual approval |
| Raster illustration | Available native image tool or explicitly configured provider; composition and crop review | Claiming a portable image model is installed by a skill |
| Office | Native host tools when present; independent editable OOXML and isolated render/recalc fallback | Copying restricted provider skill payloads or promising Office parity |
| Markdown | Local structure/link checks, real renderer, verified commands and editorial review | Treating a linter as a technical/editorial reviewer |
| Programming | Existing project contracts/gates, realistic failure fixtures and bounded fresh review | Universal architecture or auto-skipping failed tests |
| QA | Exact artifacts, hashes, exercised outcomes and visible defects | Self-awarded beauty scores, unverified completion hooks |
| Distribution | Shared sources, target adapters, optional self-contained Codex plugin export | Duplicating every skill into host-specific implementations |

The local workbench uses tools already present on this computer. No package,
paid provider, account integration or new MCP runtime was installed. Existing
MCP configuration remains available through the target-neutral registry. A new
MCP server would add value only where the host lacks the corresponding tool;
wrapping an available local CLI in a server is not inherently more capable.

## Reference taste, without a universal template

The user's named references are Apple HIG, Carbon, Uber Base, Polaris and Primer;
admired products are Slack, GitHub, Airbnb, Apple, WhatsApp and OpenAI.
These are preferences, not empirical rankings or permission to copy their assets.

Use a reference for a specific property: a dense working product can study
navigation and state clarity; a booking flow can study search/selection; a public
product page can study narrative, typography and media; a conversation UI can
study feedback and continuity. Preserve the actual project's identity and platform.

[Primer](https://primer.style/) separates product and brand UI. Current
[Polaris documentation](https://shopify.dev/docs/api/polaris) targets Shopify's
surfaces; its components are not a universal dependency for unrelated projects.
[Slack's design discussion](https://slack.design/articles/pillars-of-digital-product-design/)
provides historical design context, not a current component specification.
Some Apple/Base/Carbon pages could not be fully extracted; the visual review
records those retrieval limits instead of claiming comprehensive inspection.

## What is delivered

The [workbench guide](../workbench.md) documents executable browser, vector,
office and Markdown commands. [Fieldwork Studio](../../examples/craft-lab/README.md)
is an original working specimen: responsive material library, search/filter/save,
keyboard-operable preview, empty recovery, editable vector studies and motion.
The office demo produces a two-page brief, three-slide deck and formula workbook.
The code and artifacts make the workflow reviewable; they do not establish that
every future output will meet the user's taste.

A Codex plugin is an optional packaging route for the same source. Current
[OpenAI documentation](https://developers.openai.com/plugins/build/plugins)
supports portable manifests, skill packages and local marketplaces. The exporter
includes the workbench and examples with its foundation snapshot. No plugin
can enforce aesthetic quality merely by installation, and plugin hooks require
host trust. We did not silently replace existing global skill installations with
duplicate plugin copies or enable new hooks.

## Evaluation boundary

The test suite includes seeded broken browser assertions, console/HTTP failures,
invalid scenarios, unsafe vectors, broken Markdown links and OOXML defects.
Real runs render the specimen and office documents using this host's existing
browser/LibreOffice/Poppler. The [verification record](verification-2026-10-09.md)
separates mechanical evidence, visual inspection and remaining gaps.

There has been no blinded, equal-budget old-vs-new model trial. Do not claim a
percentage improvement or "better than" the reviewed harnesses. To measure that,
use the same real project briefs/model/tool budget, anonymous artifact labels,
independent task/craft review, defect counts and time/cost. Retain failed runs.
The current delivery proves concrete capabilities and useful defect detection;
ongoing real work is where personal aesthetic acceptance is calibrated.
