# Engineering harness research — 2026-10-09

The best starting point is a small personal workflow backed by executable evidence: an existing browser runtime, repository-owned acceptance checks, a structured artifact record, and a separate review of the actual rendered output. Adopt useful mechanisms from established projects; do not combine their entire orchestration systems. None of the inspected evidence establishes that a bigger installed skill inventory improves this user's deliverables.

This is fresh primary-source research for Codex Desktop and Claude Desktop Code. It is independent of the repository's current architecture. Eight implementations were inspected at pinned revisions, including hook bodies, runners, test assertions, and documented limitations. No upstream installer, hook, test suite, model call, account connection, or browser session was run. Source inspection establishes what these files implement, not that they work in the user's environment. The report's proposed interfaces and evaluation criteria are recommendations, not delivered software or measured results.

## Source snapshots

The first seven revisions and declared root licenses came from GitHub repository/commit APIs. ECC's revision came from `git ls-remote ... HEAD`, and its license was read directly because the API rate limit was reached. Exact source files were fetched from `raw.githubusercontent.com` at these revisions. Repository license labels do not clear bundled assets or transitive dependency licenses.

| Candidate | Inspected revision | Declared root license | Role in the comparison |
| --- | --- | --- | --- |
| [Superpowers](https://github.com/obra/superpowers/tree/8ca22dba9a94f28898bbce59f2537ff4d87c747d) | `8ca22dba9a94f28898bbce59f2537ff4d87c747d` | MIT | Planning, bounded implementation, review, behavioral evaluations |
| [Gstack](https://github.com/garrytan/gstack/tree/20eb6202fa8ea83a882e7c0463b722cd8a31af1e) | `20eb6202fa8ea83a882e7c0463b722cd8a31af1e` | MIT | Browser QA, executable evidence, completion hooks |
| [agent-browser](https://github.com/vercel-labs/agent-browser/tree/0207911f1bd4d0393eddaa90f2e50f96e0fb8974) | `0207911f1bd4d0393eddaa90f2e50f96e0fb8974` | Apache-2.0 | Agent-facing browser driver and action policy |
| [Playwright](https://github.com/microsoft/playwright/tree/b28411b105a25fcd97014111764a1cf76f0c6bb0) | `b28411b105a25fcd97014111764a1cf76f0c6bb0` | Apache-2.0 | Repeatable assertions, visual differences, test agents |
| [Playwright MCP](https://github.com/microsoft/playwright-mcp/tree/b8b4183e099f136cbec0388a6088d4aa2f6b9685) | `b8b4183e099f136cbec0388a6088d4aa2f6b9685` | Apache-2.0 | Browser tool transport for hosts lacking a native browser |
| [Compound Engineering](https://github.com/EveryInc/compound-engineering-plugin/tree/67035e931c5cb26e80f198a7019a4502a9788273) | `67035e931c5cb26e80f198a7019a4502a9788273` | MIT | Review synthesis, isolated integration, comparative evaluations |
| [GSD, historical repository](https://github.com/gsd-build/get-shit-done/tree/bdcaab2c752d9a33a1a1ca9acf3a3c81fb991815) | `bdcaab2c752d9a33a1a1ca9acf3a3c81fb991815` | MIT | Durable workflow state and verification records |
| [Everything Claude Code / ECC](https://github.com/affaan-m/everything-claude-code/tree/ef648e01899ba3e8dc6371642deaaf64b4477775) | `ef648e01899ba3e8dc6371642deaaf64b4477775` | MIT | Hook orchestration and lightweight edit checks |

**GSD provenance caveat:** its current landing page explicitly redirects active development to [Open GSD / GSD Core](https://github.com/gsd-build/get-shit-done#gsd-has-moved). The code inspected here is the historical snapshot above, whose API commit date was 2026-05-31. It is useful comparative evidence, not a recommendation to install an obsolete origin. The successor implementation was not audited in this pass.

## What the implementations actually do

### 1. Superpowers: strong workflow structure, largely agent-enforced

The [session hook](https://github.com/obra/superpowers/blob/8ca22dba9a94f28898bbce59f2537ff4d87c747d/hooks/session-start#L10-L50) reads the introductory skill and emits host-specific context. That is automatic instruction delivery, not a test gate. The [implementation workflow](https://github.com/obra/superpowers/blob/8ca22dba9a94f28898bbce59f2537ff4d87c747d/skills/subagent-driven-development/SKILL.md#L8-L117) separates implementation, task review, and a final branch review. Its benefit is a concrete process with fresh review context; its cost is added dispatch and ceremony.

The [integration test](https://github.com/obra/superpowers/blob/8ca22dba9a94f28898bbce59f2537ff4d87c747d/tests/claude-code/test-subagent-driven-development-integration.sh#L1-L14) exercises an actual small implementation and checks workflow behavior. It launches authenticated Claude with broad tool access and bypassed permissions at line 167: use an isolated fixture, never copy that invocation into normal personal work. Runtime needs include Bash, Git, the authenticated agent, and fixture tooling.

Its [workspace evaluation](https://github.com/obra/superpowers/blob/8ca22dba9a94f28898bbce59f2537ff4d87c747d/docs/superpowers/specs/2026-07-06-sdd-plan-scoped-workspace-eval-results.md#L123-L186) records misleading early fixtures and corrected controls. This is useful methodological honesty, not proof of better UI or document output. Borrow bounded implementation/review and truthful resume records; do not impose every phase on a trivial edit.

### 2. Gstack: unusually concrete QA, but completion hooks have escape paths

The [QA repair instructions](https://github.com/garrytan/gstack/blob/20eb6202fa8ea83a882e7c0463b722cd8a31af1e/qa/sections/browser-verify.md) require replaying the original interaction and an adjacent happy path, preserving before/after screenshots, and comparing console and structural evidence. Its [functional QA evaluator](https://github.com/garrytan/gstack/blob/20eb6202fa8ea83a882e7c0463b722cd8a31af1e/test/helpers/qa-functional-eval.ts#L17-L54) ties native probe output to structured evidence rather than letting the model invent observed fields. This is the most useful mechanism to borrow.

The [planted-bug evaluator](https://github.com/garrytan/gstack/blob/20eb6202fa8ea83a882e7c0463b722cd8a31af1e/test/skill-e2e-qa-bugs.test.ts#L21-L179) measures detection, false positives, and evidence quality. It can skip when credentials/browser prerequisites are absent, and uses a directed prompt rather than the whole QA workflow; passing it would not validate every product path.

The [Stop hook](https://github.com/garrytan/gstack/blob/20eb6202fa8ea83a882e7c0463b722cd8a31af1e/bin/gstack-verify-gate#L1-L20) binds trust to a command hash, but [allows completion after bounded repeated failures](https://github.com/garrytan/gstack/blob/20eb6202fa8ea83a882e7c0463b722cd8a31af1e/bin/gstack-verify-gate#L194-L222). Trust grants remain agent-runnable. This is a guardrail, not an unbypassable acceptance boundary. The [package](https://github.com/garrytan/gstack/blob/20eb6202fa8ea83a882e7c0463b722cd8a31af1e/package.json) requires Bun/build tooling for its implementation, plus browser/model prerequisites for relevant lanes. Adopt the evidence pattern before adopting this broad runtime.

### 3. agent-browser: useful driver, optional policy, distinct evaluation scope

The [Rust action policy](https://github.com/vercel-labs/agent-browser/blob/0207911f1bd4d0393eddaa90f2e50f96e0fb8974/cli/src/native/policy.rs#L63-L120) implements deny, confirmation, allow-list, and default decisions. The [security documentation](https://github.com/vercel-labs/agent-browser/blob/0207911f1bd4d0393eddaa90f2e50f96e0fb8974/docs/content/docs/security.mdx#L13-L32) explicitly limits domain controls and says local plugins are executables, not sandboxed components. Browser policy therefore complements host permissions; it cannot replace them.

The [evaluation runner](https://github.com/vercel-labs/agent-browser/blob/0207911f1bd4d0393eddaa90f2e50f96e0fb8974/evals/run.ts) has timeouts, machine-readable results, optional paid judging, and nonzero failure exits. However, inspected [command-usage cases](https://github.com/vercel-labs/agent-browser/blob/0207911f1bd4d0393eddaa90f2e50f96e0fb8974/evals/cases/command-usage.ts) grade command/workflow usage, not finished product design.

The pinned [package](https://github.com/vercel-labs/agent-browser/blob/0207911f1bd4d0393eddaa90f2e50f96e0fb8974/package.json#L7-L34) declares Node 24+, pnpm 11+ for its package/development workflow, Rust for native builds, and a postinstall script. Browser provisioning is an additional capability. Consider it when neither host offers adequate browser control; an already available Playwright runtime is a smaller initial dependency surface.

### 4. Playwright: durable regression evidence; test healing needs constraints

The [generator tool implementation](https://github.com/microsoft/playwright/blob/b28411b105a25fcd97014111764a1cf76f0c6bb0/packages/playwright/src/mcp/test/generatorTools.ts#L26-L75) runs a seed test and records a generation journal. The [screenshot matcher](https://github.com/microsoft/playwright/blob/b28411b105a25fcd97014111764a1cf76f0c6bb0/packages/playwright/src/matchers/toMatchSnapshot.ts#L331-L445) produces expected/actual/difference evidence and handles screenshot options. Browser binaries and the project's supported Node/test environment are needed; the library itself can also support a small collector without `@playwright/test`.

A consequential limit appears in the [healer agent](https://github.com/microsoft/playwright/blob/b28411b105a25fcd97014111764a1cf76f0c6bb0/packages/playwright/src/agents/playwright-test-healer.agent.md#L24-L56): after investigating a correct test that still fails, it can mark the test `fixme`, which skips execution. That may document a known defect but must not count as successful repair in this personal harness. Snapshot updates also require deliberate review: similarity to an accepted baseline is regression evidence, not proof that the baseline is attractive or usable.

Prefer Playwright for repeatable critical flows and visual regression where an existing project already uses it. Keep human or fresh-agent inspection of rendered artifacts as a separate completion criterion.

### 5. Playwright MCP: transport choice, not an additional QA methodology

The inspected [CLI](https://github.com/microsoft/playwright-mcp/blob/b8b4183e099f136cbec0388a6088d4aa2f6b9685/cli.js#L18-L32) delegates implementation to Playwright Core. The [package](https://github.com/microsoft/playwright-mcp/blob/b8b4183e099f136cbec0388a6088d4aa2f6b9685/package.json#L10-L45) declares Node 18+ and pins an alpha Playwright version. Its [capability test](https://github.com/microsoft/playwright-mcp/blob/b8b4183e099f136cbec0388a6088d4aa2f6b9685/tests/capabilities.spec.ts#L19-L67) checks the tools exposed, including browser evaluation, uploads, screenshots, and unsafe code execution. This is executable transport regression coverage, not evidence of high-quality application output.

The [maintainer security guidance](https://github.com/microsoft/playwright-mcp#security) explicitly disclaims a security boundary. Origin filters likewise do not constitute a complete network boundary. Prefer isolated local browser contexts and host permission controls; inspect actual exposed tools rather than assuming that a harmless-sounding MCP name grants only read access.

Use it only if it fills a real host capability gap. Running native browser control, agent-browser, and Playwright MCP simultaneously by default adds overlapping tools and state without demonstrated quality benefit.

### 6. Compound Engineering: useful review mechanics and controlled comparisons

The [findings processor](https://github.com/EveryInc/compound-engineering-plugin/blob/67035e931c5cb26e80f198a7019a4502a9788273/skills/ce-code-review/scripts/findings-mechanics.py) validates and combines structured reviewer returns. The [workspace integration code](https://github.com/EveryInc/compound-engineering-plugin/blob/67035e931c5cb26e80f198a7019a4502a9788273/skills/ce-work/scripts/unit_workspace_integration.py#L16-L105) binds integration locks to repository/branch/unit identity and validates a nonce. These are executable coordination mechanisms, beyond review instructions.

The [review-calibration report](https://github.com/EveryInc/compound-engineering-plugin/blob/67035e931c5cb26e80f198a7019a4502a9788273/docs/plans/2026-09-04-review-calibration-eval-report.md#L27-L59) compares selected old/new prompts in fresh Claude/Codex sessions. It explicitly distinguishes transcript semantics from keyword grades and limits what its cases prove. Raw evidence is referenced at a local temporary path, so it was not independently reproduced here.

The [test runner](https://github.com/EveryInc/compound-engineering-plugin/blob/67035e931c5cb26e80f198a7019a4502a9788273/scripts/run-tests.ts#L1-L10) distinguishes timeout-only process issues from assertion failures before a serial retry. Runtime needs include Bun for its CLI/test tooling, Python for inspected mechanics, Git, and agents for live evaluations. Borrow evidence-grounded findings and paired comparisons; avoid importing every review persona into every task.

### 7. GSD: state records are valuable, but record parsing is not observation

The historical [workflow guard](https://github.com/gsd-build/get-shit-done/blob/bdcaab2c752d9a33a1a1ca9acf3a3c81fb991815/hooks/gsd-workflow-guard.js#L1-L12) is opt-in and advisory. Its [output path](https://github.com/gsd-build/get-shit-done/blob/bdcaab2c752d9a33a1a1ca9acf3a3c81fb991815/hooks/gsd-workflow-guard.js#L76-L93) injects context and does not block an edit. Describing this hook as enforced workflow compliance would be incorrect.

The [verification-status tests](https://github.com/gsd-build/get-shit-done/blob/bdcaab2c752d9a33a1a1ca9acf3a3c81fb991815/sdk/src/query/check-verification-status.test.ts#L27-L103) write Markdown PASS/FAIL records and check parsing/routing. Those tests establish record semantics; they do not independently verify that the recorded application behavior happened. Its [package](https://github.com/gsd-build/get-shit-done/blob/bdcaab2c752d9a33a1a1ca9acf3a3c81fb991815/package.json#L47-L93) requires Node 22+, includes the Claude Agent SDK, and separates unit/integration/install/security lanes.

Borrow compact, durable task state for long work. Bind verification records to actual command outputs, source identity, and artifacts. Do not install this historical origin without reviewing its successor and migration behavior.

### 8. ECC: extensive automation, but inspect the exact meaning of “gate”

The [quality hook](https://github.com/affaan-m/everything-claude-code/blob/ef648e01899ba3e8dc6371642deaaf64b4477775/scripts/hooks/quality-gate.js#L57-L148) finds local formatters, can mutate files when its fix option is enabled, and skips unavailable tooling. Even strict-mode failures in this file are logged; the function returns its original input. This particular hook does not establish passing tests or block completion. That conclusion is limited to this script, not every ECC safeguard.

The [hook configuration](https://github.com/affaan-m/everything-claude-code/blob/ef648e01899ba3e8dc6371642deaaf64b4477775/hooks/hooks.json) wires multiple lifecycle and tool hooks, including asynchronous work. The [test runner](https://github.com/affaan-m/everything-claude-code/blob/ef648e01899ba3e8dc6371642deaaf64b4477775/tests/run-all.js#L76-L150) executes test files and treats failed subprocesses as failures. Its package declares Node 18+; individual checks depend on project-installed formatters or Go/Python tooling.

Borrow selective, bounded local checks only where they have measured value. Global formatter hooks can create extra writes, duplicate project tooling, and execute repository-resolved binaries. Hook source and lifecycle wiring must be reviewed together. No comparative product-quality gain was established in this inspection.

## Recommended minimal architecture

This proposal follows the observed mechanisms rather than any existing repository layout.

1. **One short common policy:** user intent, project conventions, permission boundaries, evidence before completion. Host adapters express supported settings and invocation details, not duplicate behavior prose.
2. **Four outcome workflows:** implement/debug; design/build/inspect; author/render/inspect; investigate/synthesize. Load a specialist only when needed. Each workflow declares the artifact or behavior that must be observed.
3. **One execution contract:** commands and browser scenarios produce structured results. Prefer project-owned test commands and existing runtimes. Host hooks may remind or launch trusted checks, but the evidence record remains authoritative when a host lacks the hook.
4. **One artifact reviewer:** inspect the actual application, image, PDF, slides, or document. Classify functional defects, design defects, preferences, and unverified coverage separately. Rendering is a prerequisite for acceptance, not acceptance itself.
5. **One small evaluation set:** measure recurring user tasks against a baseline before adding more prompts, tools, hooks, or reviewer roles.

Keep target integration separate from runtime capability. A skill declaration cannot install a browser, make an image model available, render a DOCX, or authenticate an MCP server. A readiness report should name what is available, missing, optional, and actually exercised. Use host-native tools when suitable; add a transport only to resolve an observed gap.

## First executable deliverable: browser evidence collector

A useful initial implementation can use an already available Playwright library directly. It does not require installing `@playwright/test`, axe, agent-browser, or a second browser MCP merely to gather evidence. Confirm the supplied library and browser executable in the target environment before claiming readiness; this research did not test the local runtime.

### Input contract

Accept a versioned JSON scenario, with an explicit base URL, viewport list, color scheme and reduced-motion modes, ordered actions, named checkpoints, expected assertions, and bounded timeouts. Use a small allow-list of action types: navigate, click, fill, select, press, and scroll. Prefer role/label locators, with CSS as an explicit escape hatch. Assertions should cover visible text, element visibility/count, URL, enabled/disabled state, input value, and optionally accessible names where supported.

Do not accept arbitrary shell or JavaScript strings from scenario data. The collector's own fixed inspection code can use browser evaluation. Restrict the initial runner to authorized local applications and explicit allowed origins; validate navigation destinations and redirects. This check remains a convenience guard, not a complete network sandbox. Mutating a local fixture is different from submitting a production form.

### Capture and observation contract

For every viewport/motion mode and named checkpoint, save a PNG and a JSON record with the scenario ID, action history, assertion outcomes, timestamps, URL, browser/runtime versions, source revision, dirty-tree/content identity, screenshot dimensions and hashes. Keep browser console errors, page exceptions, failed transport requests, and HTTP error responses as separate categories. `requestfailed` alone misses HTTP 4xx/5xx responses.

Capture desktop and mobile states, not just the initial homepage. Record document and relevant element overflow with measured bounds; allow explicit exceptions for intentional carousels or scroll areas. Preserve evidence before and after a repair. For motion, capture normal/reduced-motion variants and named time samples; label them sampled states rather than proof of complete animation correctness. Reduced-motion screenshots do not prove that normal motion is well designed.

Collect failure evidence in `finally` paths, close contexts deterministically, bound all waits, and emit a nonzero exit for assertion failures. Separate `pass`, `fail`, `blocked`, and `inconclusive`; missing browser/runtime, failed launch, and missing required assertions must never become a green empty report. Give each run a fresh output directory and reject collisions or linked output paths.

### Completion contract

Automatic success requires the declared scenarios to execute and their assertions to pass. Visual review remains explicitly pending until someone inspects the screenshots. Manual review records should identify exact artifact hashes so subsequent edits cannot retain stale approval. No screenshots means no visual acceptance claim. No axe dependency means no claim of a comprehensive accessibility audit; keyboard/focus and semantic checks can still be declared and exercised individually.

Do not add snapshot baselines during ordinary repair, broaden thresholds, suppress console categories, mark tests skipped, or silently change required scenarios to turn failures green. Preserve the failed evidence and explain which input or implementation changed. The checker verifies browser-observable behavior; project tests remain necessary for business rules, authorization, transactions, and non-UI failure paths.

## Quality comparisons that would justify changes

Use a frozen baseline, identical task briefs and starting repositories, declared host/model/runtime versions, comparable budgets, and at least three repetitions per task/arm for an initial screen. Three repetitions are a practical pilot, not a statistical guarantee. Compare each host against its own baseline before interpreting cross-model differences. Include abandoned, timed-out, blocked, and partial runs in results.

| Representative task | Independent acceptance evidence | Quality measurement |
| --- | --- | --- |
| Responsive page from a real brief and supplied references | Desktop/mobile captures; navigation/form assertions; no accidental overflow | Blind preference review for hierarchy, typography, spacing, imagery, coherence, and fidelity to the brief |
| Repair an existing SPA checkout with seeded defects and harmless decoys | Hidden behavioral tests; checkout state assertions; before/after reproductions | Defect detection and repair rate; false positives; regressions; unnecessary code churn |
| Animated interactive component | Keyboard/pointer cases; normal/reduced-motion samples; state and cancellation assertions | Temporal continuity, motion purpose, reduced-motion behavior; sampled frames judged separately from full playback |
| Document or slide artifact from supplied facts | Editable source; real renderer output; content extraction and citation checks | Layout, overflow, hierarchy, factual fidelity, editability, and visual consistency |
| Asset-heavy landing page | Actual loaded asset files; dimensions/crop/contrast; provenance and required variants | Brief fit and cohesion; reject placeholder assets and claims based only on filenames |
| Backend correction with concurrency/error requirements | Hidden negative/concurrency tests; exact outputs; fresh review | Correctness, invariants, maintainability, failure behavior, and regression scope |
| Long task interrupted and resumed | Source/artifact identity checks; evidence of completed work retained | Duplicate work, stale completion claims, omissions, and recovery time |

Keep acceptance separate from preference. A beautiful screenshot with a broken form fails. A functionally passing page can still lose the design comparison. Report human preference independently from automated scores, use reviewers blind to harness identity where possible, and disclose disagreements. A model judge can assist triage, but calibrate it against known positive/negative examples and human inspection; never let it rewrite the acceptance oracle.

Start with three arms: native host plus project instructions; native host plus the small outcome workflow; the same workflow plus executable evidence and fresh artifact review. Only then trial a specialist such as Superpowers task execution, Gstack QA, or Compound review on the task where its mechanism should help. This isolates the source of improvement instead of attributing every difference to a bundle.

Record task success, reviewer preference, false positive findings, repairs needed after review, user interventions, elapsed time, tool/model cost where observable, and unauthorized/undesired effects. Promote an addition only when it improves the intended dimension without an unacceptable regression elsewhere. The immediate implementation priority is the collector and representative tasks, not an expanded catalog.

## Research limits and next decisions

This pass inspected selected implementation paths, not complete security audits. No suite was run and no published numeric result was reproduced. Reported upstream evaluations often target workflow behavior or toy/seeded fixtures, and some depend on paid model calls or unavailable local evidence. They cannot establish superiority on the user's repositories, aesthetic preferences, or document work.

Before adopting an upstream component, select a stable release corresponding to reviewed source, inspect its installer and transitive executables, and run an isolated smoke test plus one representative output-quality task. Before writing more orchestration, use the existing browser/runtime capability to deliver one observable, reviewable result. The necessary decisions are which recurring task becomes the first fixture and what its acceptance criteria are; no bulk installation is justified by this report.
