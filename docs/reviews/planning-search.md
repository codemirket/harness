# Planning and search-source review

Reviewed 2026-10-06. These are pinned observations, not endorsements of upstream claims. No upstream code, installer, hooks, agent workflow or tests were executed. No user-home installs or settings changes were made.

## Decisions

- Use the newly authored [work-planning](../../skills/work-planning/SKILL.md) globally for substantial work. It preserves task identity, outcome-based steps, evidence, ownership, and resumability, without installing hooks or requiring three files for every task.
- Offer the newly authored [search-visibility](../../skills/search-visibility/SKILL.md) as a project opt-in for evidence-led SEO/GEO audits. It separates training, crawling, indexing, retrieval, citation, and actual business outcomes.
- Keep all upstream entries manual references for now: six canonical planning languages require an explicit runtime integration review; all sixteen GEO skills need substantive adaptation. These are not exclusions based merely on opinionated process. Planning has meaningful executable behavior and incomplete runtime assurance; GEO contains material factual and measurement defects.
- Do not wire either upstream installer or auto-updater into the personal harness. Do not enable planning stop gates, replay, periodic loops, or lifecycle hooks through ordinary skill registration.

## Snapshot and license

| Source | Pinned commit | Canonical skills | Physical SKILL.md files | License |
|---|---|---:|---:|---|
| [OthmanAdi/planning-with-files](https://github.com/OthmanAdi/planning-with-files/tree/dab9d16fbd9314448b319d112e99f497d7638d89) | `dab9d16fbd9314448b319d112e99f497d7638d89` | 6 | 18 | MIT, root LICENSE, copyright 2026 Ahmad Adi |
| [zubair-trabzada/geo-seo-claude](https://github.com/zubair-trabzada/geo-seo-claude/tree/989cae01e8ebbc42a9ec798eb9e7cb423f5ec89c) | `989cae01e8ebbc42a9ec798eb9e7cb423f5ec89c` | 16 | 16 | MIT, root LICENSE, copyright 2026 Zubair Trabzada |

MIT permits redistribution/adaptation while retaining copyright and permission notices in copies or substantial portions. Both license bodies were read; recursive license-file discovery found no additional file-specific license files. GEO's white-label helper explicitly identifies contributor Millisa Nwokolo (La Crown Inc.) and MIT licensing. Its optional dashboard loads Bootstrap, Bootstrap Icons, and HTMX remotely; third-party packages/assets have their own terms and were not audited as redistributed dependencies. The two new skills are original writing, not copied source or template payloads.

## Coverage and limitations

[source-index.json](../../registry/source-index.json) lists all 34 physical skill files with exact paths, frontmatter names/descriptions, SHA-256 and immutable source URLs. Twenty-two canonical full bodies were read: the planning English skill and five translations, plus GEO's root router and fifteen subskills. Twelve planning host mirrors are separately indexed rather than inflated into twelve extra capabilities; the `.agents` mirror was partially read and the others were not independently fully reviewed.

GEO supporting files fully read: all five `agents/*.md`; all five root `scripts/*.py`; `scripts/webapp/app.py`; `scripts/webapp/templates/base.html`; `templates/geo-report-template.html`; all six `schema/*.json`; `requirements.txt`; root `CLAUDE.md`; `docs/scoring-methodology.md`; `docs/architecture.md`; `install.sh`, `install-win.sh`, `uninstall.sh`; white-label README and Python helper. Root README was browsed on GitHub. Other dashboard templates, CSS, demonstration outputs, tests, and the remaining docs were indexed or selectively scanned, not fully audited. No GEO executable package is approved for direct installation.

Planning supporting files fully read: both plugin manifests, both hook JSON manifests, `scripts/skill-hook.sh`, `scripts/resolve-plan-dir.sh`, `scripts/check-complete.sh`, `scripts/gate-stop.sh` under the canonical English skill; `.codex/hooks/plugin_dispatch.py`, `run_sh.py`, `pwf-hook.cmd`; reference.md, examples.md, and loop.md template. Root AGENTS.md and LICENSE were read as upstream source material. The root Claude dispatcher, `.agents` mirror, other templates, session-catchup.py, injectors, and several shell helpers were read in targeted sections or scanned. The large remaining shell/Python/PowerShell runtime, host mirrors, and tests were not exhaustively reviewed or run. The canonical English payload alone has over 10,000 lines of executable helpers. The recommendation therefore does not claim runtime security or Windows integration validation.

The authored skills both passed the installed skill creator's `quick_validate.py` using the existing validation virtual environment. This establishes valid skill frontmatter/structure; semantic claims were checked by inspection and provider documentation. There was no live site audit or native Windows execution.

## Planning: retain the task-state pattern; make runtime opt-in

[Canonical skill](https://github.com/OthmanAdi/planning-with-files/blob/dab9d16fbd9314448b319d112e99f497d7638d89/skills/planning-with-files/SKILL.md) and translations `skills/i18n/planning-with-files-{ar,de,es,zh,zht}/SKILL.md` share the main goal: preserve a selected task's plan, findings, and progress. Useful techniques are explicit task identity, preserving source/file pointers, restoring the next action, comparing saved progress with actual changes, and one writer for shared planning state. These informed the original work-planning skill.

The six names are alternatives for a language preference, not six complementary skills. Do not globally install every translation or every IDE mirror. English and translation bodies differ in length/features even while metadata versions match.

The instruction file itself is not inert: Claude frontmatter contains UserPromptSubmit, PreToolUse, PostToolUse, Stop and PreCompact command hooks. [Plugin hooks](https://github.com/OthmanAdi/planning-with-files/blob/dab9d16fbd9314448b319d112e99f497d7638d89/hooks/hooks.json) add SessionStart; [Codex hooks](https://github.com/OthmanAdi/planning-with-files/blob/dab9d16fbd9314448b319d112e99f497d7638d89/hooks/codex-hooks.json) use different adapters and a PermissionRequest route. Copying a directory and configuring a full plugin are different operations. No assumption should be made that another host honors Claude skill hooks.

The system writes selected project task files, `.planning/` metadata, ledgers, mode/nonce/attestation/stop-count files, and user caches such as `~/.cache/pwf-turn`, `pwf-sha` and snapshots. Hook output may enter model-provider requests. The explicit catch-up CLI can read same-project local Claude/Codex/OpenCode history. Its reviewed entrypoint returns immediately in default no-history mode; metadata/replay modes require explicit selection. Metadata processing can still read transcript records internally even when output contains only counts. Do not describe this as reading only filesystem metadata.

[Gate implementation](https://github.com/OthmanAdi/planning-with-files/blob/dab9d16fbd9314448b319d112e99f497d7638d89/skills/planning-with-files/scripts/check-complete.sh) has good bounded-continuation ideas: opt-in mode, in-progress state, recursive-hook guard, cap, and ledger-stall detection. But it counts plan text, not product behavior or passing verification. A new ledger line is evidence of activity, not evidence of useful progress. Root `.mode` also influences descendant plans. Do not describe the gate as proof of completion or enable it for all work.

Attestation hashes detect plan-only changes while the stored digest remains trusted. Upstream correctly admits that co-located nonce/hash files do not protect against an actor able to rewrite both, and initialization attestation is not human approval. Retain that distinction. The plan's single-writer model is more dependable than treating completion-count checks as a concurrency lock.

Exclude from global policy: creating three files for every complex task; saving after every two reads; logging every transient error; never retrying the same operation; escalating automatically after three failures; skipping rereads of written artifacts; assumptions about model names, slash commands, or universally available continuation features. Each can conflict with relevant verification or efficient recovery. The `plan-goal` discussion also describes older fallback selection that does not match the stronger explicit-binding guidance at the start; rely on task identity instead of newest-file selection.

Portability: POSIX shell/coreutils on macOS/Linux; optional/trusted Python paths for fast injection and catch-up; bundled PowerShell helpers; Codex Windows adapters rely on Python and Git for Windows shell. This is substantive platform integration, not dependency-free prose. Review and test actual host hook contracts, path quoting, parallel-task selection, cache behavior, collision handling and stop termination in isolated fixtures before enabling an upstream plugin.

## GEO: useful audit questions; unreliable causal and numerical guidance

[Source scoring methodology](https://github.com/zubair-trabzada/geo-seo-claude/blob/989cae01e8ebbc42a9ec798eb9e7cb423f5ec89c/docs/scoring-methodology.md) admits its weights are author judgments rather than controlled-study results. Other parts nevertheless label high scores as likely citation, call low scores invisibility, or turn point increases into traffic and revenue. These are different claims and the disclaimer does not make the latter valid.

[OpenAI's documentation](https://developers.openai.com/api/docs/bots) separates OAI-SearchBot search indexing from GPTBot training; ChatGPT-User performs certain user-triggered requests. [Anthropic's documentation](https://support.claude.com/en/articles/8896518-does-anthropic-crawl-data-from-the-web-and-how-can-site-owners-block-the-crawler) separately identifies ClaudeBot, Claude-SearchBot, and Claude-User. GEO's crawler/visibility prompts conflate these roles and penalize intentionally blocking training crawlers. Do not recommend opening training access as a prerequisite for search visibility.

[Google's current AI search guidance](https://developers.google.com/search/docs/appearance/ai-features) says no special AI text files or special schema are needed for its AI features. Treat missing llms.txt as a consumer-specific consideration, not a universal serious SEO defect. Eligibility still does not guarantee inclusion.

[The schema agent](https://github.com/zubair-trabzada/geo-seo-claude/blob/989cae01e8ebbc42a9ec798eb9e7cb423f5ec89c/agents/geo-schema.md) promotes WebSite+SearchAction as enabling a search box. [Google removed that search result feature in November 2024](https://developers.google.com/search/blog/2024/10/sitelinks-search-box). Its vocabulary-versus-feature distinction also needs correction: retirement of a rich result is not retirement of the underlying Schema.org type. Universal speakable, five-social-profile and Wikipedia requirements are not justified.

[The technical skill](https://github.com/zubair-trabzada/geo-seo-claude/blob/989cae01e8ebbc42a9ec798eb9e7cb423f5ec89c/skills/geo-technical/SKILL.md) recommends canonicalizing pagination to page one, contrary to [Google's pagination guidance](https://developers.google.com/search/docs/specialty/ecommerce/pagination-and-incremental-page-loading), which gives each page its own canonical. Generic fixes such as HSTS includeSubDomains or mandatory SSR should be evaluated against actual architecture and owner intent, not deployed from a checklist. The technical agent also lists missing lazy-loading on hero images as an LCP risk; applying lazy loading to a critical hero can worsen delivery. Measure the actual page.

### Individual dispositions

| Canonical path | Useful component | Required correction |
|---|---|---|
| `geo/` | Route a requested audit to relevant capabilities | Remove broad URL-triggering, unsupported market facts, rigid subagent assumptions |
| `skills/geo-audit/` | Bounded sampling, findings and priorities | Replace unvalidated composite probability claims; distinguish unknown data |
| `skills/geo-brand-mentions/` | Dated cross-platform entity evidence | Verify identity and actual checks; correlation is not causation |
| `skills/geo-citability/` | Direct answers with sourced facts | Remove arbitrary 134–167 word optimum and citation-probability labels |
| `skills/geo-compare/` | Period/action tracking | Never infer missing baseline scores or convert points into revenue |
| `skills/geo-content/` | Provenance, actual expertise, readable coverage | Remove universal length floors and unsupported AI-authorship inference |
| `skills/geo-crawlers/` | Agent-specific access inventory | Correct bot purposes and robots parsing; preserve owner preferences |
| `skills/geo-llmstxt/` | Optional documentation navigation | Verify consumer need; remove universal priority and invented contacts |
| `skills/geo-platform-optimizer/` | Separate platform observations | Remove fabricated certainty about proprietary algorithms and fixed minima |
| `skills/geo-proposal/` | Commercial scope/pricing discussion | User-supplied prices and substantiated projections; no invented benchmarks |
| `skills/geo-prospect/` | Explicitly requested local pipeline tracking | Isolate client data; transactional storage and validated states |
| `skills/geo-report/` | Evidence/action report outline | Consistent methods, truthful uncertainty, no invented financial benefits |
| `skills/geo-report-pdf/` | Branded report presentation | Portable renderer, no blanket disabled sandbox, actual visual verification |
| `skills/geo-schema/` | Extract raw JSON-LD before text conversion | Current feature docs, accurate entities, no mandatory irrelevant types |
| `skills/geo-technical/` | Compare raw/rendered content and index controls | Correct pagination and performance advice; findings must be observed |
| `skills/geo-update/` | Check for upstream changes | Use reviewed pinned catalog updates; do not overwrite managed global trees |

### Executable/helper findings

- [fetch_page.py](https://github.com/zubair-trabzada/geo-seo-claude/blob/989cae01e8ebbc42a9ec798eb9e7cb423f5ec89c/scripts/fetch_page.py): robots parser remembers only the last User-agent in a group, uses case-sensitive keys, and treats any root Disallow as blocked without evaluating a more specific Allow. Wildcard partial restrictions are also lost. These are meaningful false audit results. HTTP responses are unbounded in bytes, redirects are followed, and child sitemap URLs are fetched without an origin constraint. The max_pages parameter bounds returned URLs, not the count of empty child-sitemap requests. No one-second pacing exists in this helper. Parsing headers/JSON-LD before removing elements is useful; later link extraction omits removed navigation/footer links and relative links use the original URL rather than redirect destination.
- [brand_scanner.py](https://github.com/zubair-trabzada/geo-seo-claude/blob/989cae01e8ebbc42a9ec798eb9e7cb423f5ec89c/scripts/brand_scanner.py): YouTube/Reddit/LinkedIn functions return instructions and default false flags without querying those sites. Wikipedia/Wikidata requests swallow failures; first-hit string matching is not entity verification. Do not treat output as a completed multi-platform measurement.
- [citability_scorer.py](https://github.com/zubair-trabzada/geo-seo-claude/blob/989cae01e8ebbc42a9ec798eb9e7cb423f5ec89c/scripts/citability_scorer.py): English regex heuristics reward numbers, proper nouns, and phrases such as original research without checking truth. A score is not model citation probability. Scores vary structurally from some agent rubrics; multilingual suitability is not established.
- [llmstxt_generator.py](https://github.com/zubair-trabzada/geo-seo-claude/blob/989cae01e8ebbc42a9ec798eb9e7cb423f5ec89c/scripts/llmstxt_generator.py): emits `contact@<domain>` without finding an address. The cross-origin check is made before an HTTP request that follows redirects, so its stated redirect protection is ineffective. Homepage-relative URLs are resolved against domain root. Validate generated links and do not publish invented contact information.
- [CRM Flask app](https://github.com/zubair-trabzada/geo-seo-claude/blob/989cae01e8ebbc42a9ec798eb9e7cb423f5ec89c/scripts/webapp/app.py): defaults to localhost and debug false, which is preferable to public binding, but JSON writes truncate in place without atomic replacement or locking. No authentication/CSRF mechanism is visible. Do not expose it on a network or use simultaneous agent/UI writers as a trusted CRM. Its base template loads remote scripts/styles from unpkg/jsDelivr; those requests and page-executing scripts are additional data-flow/dependency considerations.

### Installation and portability

`install.sh` copies sixteen skill trees and five agents into global Claude directories without preflighting existing ownership. It deletes/recreates an internal venv and rewrites Markdown and shebangs. It installs bounded-version Python dependencies from the package index; optional Playwright browser installation is prompted interactively and skipped in noninteractive mode. `install-win.sh` requires Git Bash and installs packages with `pip --user` instead of the isolated venv. Neither is the personal registry's pinned, collision-protecting installer.

`uninstall.sh` removes all matching global `geo-*` skill directories and agent files, not an ownership receipt. Its claim that all dependencies are isolated is not true for the Windows installer path. `geo-update` clones floating current content and overwrites managed trees; it must remain excluded from harness execution. No actual root hooks directory exists in this snapshot, despite optional installer support for copying one.

Runtime: Python >=3.10; requests, BeautifulSoup, lxml, Playwright, Pillow, urllib3, validators, Flask, rich per requirements.txt. PDF instructions require Pandoc and hardcoded macOS Chrome, with `--no-sandbox`; they are not Windows-ready as written. Individual skills reference shared `~/.claude/skills/geo/scripts/`, templates/schema and agents outside their own directory, so an unchanged one-directory catalog copy is incomplete. Codex tool names, agent launching, and install paths require adaptation.

## Harness implications

Global discovery should advertise capabilities without injecting every specialty procedure into each task. Register project opt-ins explicitly; validate the local path and receipt, then read the relevant skill body at task use. A matched keyword is not permission to install a hook-bearing runtime or enable a global updater. Distinguish a skill being discoverable from a host executing hooks or following every instruction: installer evidence can prove files/links/configuration, not model compliance.

The two original skills keep the useful decision points in portable prose, leaving tools, persistent automation, and external writes subject to the user's task and actual host capabilities.

## Persistent registry integration

Source pins, disposition, paths, dependencies, caveats and exact adaptations are recorded in [the catalog](../../registry/catalog.json); physical source entries and canonical/mirror distinctions are in [the source index](../../registry/source-index.json). Project payloads have separate raw and installed hashes. Manual entries are discoverable references and are excluded from installable profiles. No runtime validation is implied by a source hash or static review.
