> Domain review evidence. Final IDs, status and counts are authoritative in [the catalog](../../registry/catalog.json); source-provider descriptions are not endorsements.

# Expanded ECC, marketing, awesome and ADHD review

Review completed against pinned checkouts on 2026-10-06. This is a source/content audit, not a claim that every example compiles or that installing a skill improves measured outcomes. No upstream scripts, installers, hooks, dependency installation or database/infrastructure commands were executed. No user-home or repository files were changed in this expanded assignment.

## Deliverables and exact coverage

- `sources.json`: four repositories, local checkout roots, exact commit SHAs, source URLs and license notices.
- `inventory.json`: 2,996 canonical `skills/**/SKILL.md` entries: ECC 293, marketing 50, awesome 2,652, ADHD 1. Every entry has exact source/path/frontmatter name/description, bundled file list and pinned permalink. All descriptions are present. Indexing is not semantic review.
- `candidates.json`: 70 individually evaluated candidates, with 53 usable opt-ins (52 direct-with-caveats, one direct), 13 adapt-first, two companion-dependent, one runtime-dependent, one license-review. 45 ECC, 11 marketing, 13 awesome, one ADHD.
- Every candidate main SKILL.md was read in full. Every selected directory's non-eval text content was read; Docker's referenced external ECC harness remains explicitly unaudited. Marketing `evals/evals.json` fixtures were indexed but not assessed or executed. Optional external integration manuals, project rules, linked skills and all public documentation links were not exhaustively audited. Candidate notes identify material outside-directory dependencies.
- `reviewed_files` enumerates actual bundled non-eval reads. Rails additionally had `rules/ruby/patterns.md` reviewed; ADHD's root README/LICENSE, hooks manifest and `hooks/always-on.mjs` were reviewed. The ADHD Gemini command is included in the skill directory but no Gemini command registration is proposed.
- Earlier same-commit audit supplied completed reads for API contracts/connectors/design, benchmarking, research, release audit, product positioning, CRO and competitors. These were reused; no claim of a new upstream revision.
- Additional inspected but unapproved routes: awesome `postgres-best-practices` main wrapper only (its 1,490-line compiled guide and many rules remain unreviewed), `postgresql-optimization` main wrapper (nine companion skills), and initial marketing SEO/schema/analytics/A-B testing/customer-research content with known issues. The earlier review remains at `docs/reviews/marketing.md`.

## Registry shape

Keep global guidance small and original. None of these 70 upstream directories needs global automatic activation. Expose the broad inventory for search; surface candidate status, required toolchain, side effects, source provenance and correction notes before project registration. Install exact directories with pinned content and license notices. Never install a whole upstream plugin merely to acquire its prompt.

Treat `direct-with-caveats` as usable only when the catalog prepends the concrete notes in `caveats`. These notes resolve bounded example mistakes or preferences without restructuring a helper implementation. `adapt-first` requires actual source/example repair plus meaningful verification before enabling automatic install. Do not let an installer downgrade those statuses. A title such as production-ready or risk:safe is an upstream claim, not assurance.

The richer opt-in requirement changes the earlier narrow recommendation: opinionated TDD, coverage targets, ADR process or output preferences may be chosen for a project. They are not automatically rejected for being opinionated. They still cannot override the user's actual scope, existing gates, authorizations or capabilities.

## High-value domain choices

| Domain | Preferred reviewed entries | Selection boundary |
|---|---|---|
| API and system contracts | ECC contract-first, api-design, api-connector-builder, hexagonal-architecture | Use meaningful independent consumers or architectural boundaries; avoid imposing layers on trivial work. |
| TypeScript/Python/Ruby/PHP backends | ECC nestjs-patterns, fastapi-patterns, django-patterns, rails-patterns, laravel-patterns | Match the existing framework; compile examples and supply real authorization/idempotency. |
| Java/.NET backend and persistence | ECC springboot-patterns, jpa-patterns, dotnet-patterns | Existing framework/runtime and actual transaction/security requirements govern. |
| Database work | ECC mysql-patterns and jpa-patterns | Reviewed PostgreSQL/Redis general guides contain substantive errors; use current official docs and original data-migration guidance pending repair. |
| Mobile | ECC flutter-dart-code-review, react-native-patterns, android-clean-architecture, compose-multiplatform-patterns, kotlin-coroutines-flows, swiftui-patterns, swift-protocol-di-testing | Load the relevant platform/language subset, not every mobile skill. Flutter implementation and Swift persistence helpers need repair. |
| Tests | ECC react-testing, golang-testing, rust-testing, cpp-testing | Behavior/evidence first, existing gates retained. Python fixture and E2E samples remain adapt-first. |
| Performance | ECC benchmark-optimization-loop, latency-critical-systems | Measured baseline, bounded hypotheses, replay/holdout, correctness and honest limitations. |
| Infrastructure and reliability | ECC kubernetes-patterns; awesome terraform-specialist, observability-engineer, grpc-golang | No implicit production mutations; detect installed tools and supported versions. |
| Security design/review | awesome threat-modeling-expert, security-auditor | Evidence, trust boundaries and resource-level authorization; no automatic intrusive scans or certification claims. |
| Documentation/writing | ECC architecture-decision-records, article-writing, brand-voice | Preserve project formats and supplied voice; no invented facts or arbitrary extra approval loops. |
| Research/business | ECC market-research, investor-materials, investor-outreach | Use sourced evidence and consistent canonical numbers; outreach means drafting unless sending is expressly authorized. |
| Marketing foundation | marketing product-marketing, competitors, cro | Persistent project context, dated competitor evidence, prioritized conversion hypotheses. |
| Growth/operations | marketing launch, pricing, onboarding, emails, revops, referrals, community-marketing, co-marketing; ECC marketing-campaign/content-engine | Frameworks and drafts, not activation authority; numerical claims require evidence. |
| Output preference | ADHD i-have-adhd from original repo | Explicit session opt-in; retain completeness and stop-mode behavior, never infer diagnosis. |

## Findings that materially change selection

1. **PostgreSQL**: awesome `postgresql` says partitioned-table foreign keys are unsupported, partial indexes cannot support ON CONFLICT and now() is volatile. These are incorrect for modern PostgreSQL. It also overstates index-only scans, prefix requirements and JSONB opclasses. ECC `postgres-patterns` contains categorical cursor performance, RLS and configuration advice requiring revision. Primary checks: [PostgreSQL INSERT](https://www.postgresql.org/docs/current/sql-insert.html), [ALTER table guidance](https://www.postgresql.org/docs/current/ddl-alter.html), [partitioning](https://www.postgresql.org/docs/current/ddl-partitioning.html).
2. **Redis**: ECC conflates write-through with atomic strong consistency and leaves important Stream pending-recovery/cluster concerns unresolved. Awesome `redis` calls a single-instance lease Redlock, uses static worker identity for a token, exposes all interfaces/admin UI, and presents non-atomic rate limiting and lossy BRPOP queues. These require substantive repair, not a generic disclaimer.
3. **Executable helpers**: awesome read-only PostgreSQL query helper uses an unnamed psycopg2 cursor; fetching 10,000 rows does not bound all client buffering. SQL-limit rewriting is string-based, TLS defaults permit plaintext, Unix permissions only warn, Windows ACLs are unchecked and results/errors may contain sensitive data. Read-only database credentials are the real control. Main, README, config example, requirements and every line of query.py were read; none executed.
4. **Infrastructure**: ECC Docker production overlays inherit development configuration unless explicitly reset. Awesome SRE dashboard burn equation divides short-window errors by long-window errors rather than allowed error budget. Awesome k6 examples invent or misstate commands/options. Kubernetes is usable with explicit correction that PDBs do not regulate Deployment rolling updates ([Kubernetes disruption docs](https://kubernetes.io/docs/concepts/workloads/pods/disruptions/)).
5. **Async/mobile**: awesome Rust async pool Drop uses an unsuitable borrowed value in a spawned static task and releases the permit before returning the resource. ECC Swift actor persistence can overwrite corrupt data after decoding failure and mutates cache before a write succeeds. ECC Flutter patterns have incomplete const hierarchies, unsafe token-refresh error completion, missing-product zero pricing and inconsistent rebuild claims. Prefer the Flutter review checklist with precise notes.
6. **Testing**: ECC Python examples use a fixed temp.txt and insufficiently isolated database fixture; repair first. React's JSDOM contrast claim is false but can be corrected in a short catalog note while retaining its useful behavioral-test workflow. Test coverage percentages are preferences, not evidence or permission to weaken gates.
7. **Marketing evidence**: almost every selected marketing guide mixes useful decision frameworks with unsourced precise lifts, rates or company anecdotes. Use these as hypotheses, not predicted results. Pricing surveys do not prove causal demand. RevOps per-contact counters do not implement a shared round-robin allocator. Outreach, campaign activation, payouts, retargeting and CRM changes remain separate actions.
8. **ADHD**: original skill explicitly disables implicit invocation for Codex and limits display grouping without limiting search, analysis or complete results. It persists until stopped. Do not adopt its clinical generalizations as medical facts or fabricate time estimates. Optional SessionStart hook reads an explicit always-on flag and injects the rules; skill-only registration does not need or install that hook.

## Licensing and portability

ECC and marketingskills are MIT at the pinned root. Carry their full root notice. All selected ECC directories are prompt-only; brand-voice adds a Markdown schema. No entire ECC plugin, hook pack, MCP setup, auto-learning or background runtime is required for these direct choices.

Awesome is **not uniformly MIT**. Its [LICENSE-CONTENT](https://github.com/sickn33/agentic-awesome-skills/blob/bdacd76ed9e388733b5f91a5c75a4e8183a7c0b5/LICENSE-CONTENT) licenses original non-code content CC-BY-4.0, while MIT applies to original code/tooling. Its [source manifest](https://github.com/sickn33/agentic-awesome-skills/blob/bdacd76ed9e388733b5f91a5c75a4e8183a7c0b5/docs/sources/sources.md) lists third-party overrides; vague Compatible or Proprietary labels do not establish permission. The five direct awesome choices have no more-specific bundled notice found and use the repository content default; this is not an independent provenance guarantee. Carry creator/source attribution, license URL, notices and modification indication when sharing CC BY material ([full license](https://creativecommons.org/licenses/by/4.0/legalcode.en)). BagelHole-imported and Apache query-helper records are held until the complete upstream notice chain is retained. Do not relabel taste-skill copies MIT merely because this aggregate repo has an MIT code license.

ADHD is MIT under its own root notice. Prefer its original repo over the awesome copy to retain explicit invocation metadata and avoid duplicate names.

Prompt files work in Codex and Claude on macOS and Windows. Applying a language/framework skill requires the actual compiler, SDK or package set. iOS/macOS builds require Apple tooling on macOS; Windows desktop builds require the corresponding Windows SDK. Bash examples need adaptation under PowerShell. Do not install dependencies from examples without the project's authorization. `documentation-lookup` specifically needs Context7 MCP; research-ops and terraform-infrastructure are routers with unavailable-companion failure modes. Their candidate status records this rather than pretending they are standalone.

## Remaining review queue

The inventory retains thousands of discoverable unreviewed choices. Useful next scoped audits include ECC ClickHouse/Ktor/Exposed/Quarkus/Angular patterns, awesome Supabase PostgreSQL rules from their actual primary origin, focused Redis official patterns, search/index pipelines, queues/event-driven resilience and specialty security tooling. These are review opportunities, not approved candidates. Root-agent work covers official OpenAI/Google/Anthropic skills and the research, parsing, context, desktop orchestration, releases and migration gaps; those sources are outside this report's ownership.

## Reproducibility

Pinned sources, selected paths, review coverage, caveats and dependencies are retained in `registry/catalog.json`; the full discovery inventory is in `source-index.json`. Review artifacts described above were working inputs to that registry. No upstream code was executed during review.

## Pinned source revisions

- ecc: [affaan-m/ECC at ef648e01899ba3e8dc6371642deaaf64b4477775](https://github.com/affaan-m/ECC/tree/ef648e01899ba3e8dc6371642deaaf64b4477775)
- marketing: [coreyhaines31/marketingskills at dda3841f0b294e01e93b1541486beefbfab0915e](https://github.com/coreyhaines31/marketingskills/tree/dda3841f0b294e01e93b1541486beefbfab0915e)
- awesome: [sickn33/agentic-awesome-skills at bdacd76ed9e388733b5f91a5c75a4e8183a7c0b5](https://github.com/sickn33/agentic-awesome-skills/tree/bdacd76ed9e388733b5f91a5c75a4e8183a7c0b5)
- adhd: [ayghri/i-have-adhd at 839872f9d1cd634fed642b4589ce7226199cc15f](https://github.com/ayghri/i-have-adhd/tree/839872f9d1cd634fed642b4589ce7226199cc15f)

## Evaluated candidate index

| Candidate | Status | Category |
|---|---|---|
| [ecc/android-clean-architecture](https://github.com/affaan-m/ECC/tree/ef648e01899ba3e8dc6371642deaaf64b4477775/skills/android-clean-architecture) | direct-with-caveats | mobile-kotlin |
| [ecc/api-connector-builder](https://github.com/affaan-m/ECC/tree/ef648e01899ba3e8dc6371642deaaf64b4477775/skills/api-connector-builder) | direct-with-caveats | api-integration |
| [ecc/api-design](https://github.com/affaan-m/ECC/tree/ef648e01899ba3e8dc6371642deaaf64b4477775/skills/api-design) | direct-with-caveats | api-design |
| [ecc/architecture-decision-records](https://github.com/affaan-m/ECC/tree/ef648e01899ba3e8dc6371642deaaf64b4477775/skills/architecture-decision-records) | direct-with-caveats | architecture-documentation |
| [ecc/article-writing](https://github.com/affaan-m/ECC/tree/ef648e01899ba3e8dc6371642deaaf64b4477775/skills/article-writing) | direct-with-caveats | writing-longform |
| [ecc/benchmark-optimization-loop](https://github.com/affaan-m/ECC/tree/ef648e01899ba3e8dc6371642deaaf64b4477775/skills/benchmark-optimization-loop) | direct-with-caveats | performance |
| [ecc/brand-voice](https://github.com/affaan-m/ECC/tree/ef648e01899ba3e8dc6371642deaaf64b4477775/skills/brand-voice) | direct-with-caveats | writing-brand |
| [ecc/compose-multiplatform-patterns](https://github.com/affaan-m/ECC/tree/ef648e01899ba3e8dc6371642deaaf64b4477775/skills/compose-multiplatform-patterns) | direct-with-caveats | mobile-kotlin |
| [ecc/content-engine](https://github.com/affaan-m/ECC/tree/ef648e01899ba3e8dc6371642deaaf64b4477775/skills/content-engine) | direct-with-caveats | marketing-content |
| [ecc/contract-first](https://github.com/affaan-m/ECC/tree/ef648e01899ba3e8dc6371642deaaf64b4477775/skills/contract-first) | direct-with-caveats | api-contracts |
| [ecc/cpp-testing](https://github.com/affaan-m/ECC/tree/ef648e01899ba3e8dc6371642deaaf64b4477775/skills/cpp-testing) | direct-with-caveats | testing-cpp |
| [ecc/dart-flutter-patterns](https://github.com/affaan-m/ECC/tree/ef648e01899ba3e8dc6371642deaaf64b4477775/skills/dart-flutter-patterns) | adapt-first | mobile-flutter |
| [ecc/django-patterns](https://github.com/affaan-m/ECC/tree/ef648e01899ba3e8dc6371642deaaf64b4477775/skills/django-patterns) | direct-with-caveats | backend-python |
| [ecc/docker-patterns](https://github.com/affaan-m/ECC/tree/ef648e01899ba3e8dc6371642deaaf64b4477775/skills/docker-patterns) | adapt-first | containers |
| [ecc/documentation-lookup](https://github.com/affaan-m/ECC/tree/ef648e01899ba3e8dc6371642deaaf64b4477775/skills/documentation-lookup) | requires-runtime | research-developer |
| [ecc/dotnet-patterns](https://github.com/affaan-m/ECC/tree/ef648e01899ba3e8dc6371642deaaf64b4477775/skills/dotnet-patterns) | direct-with-caveats | backend-dotnet |
| [ecc/fastapi-patterns](https://github.com/affaan-m/ECC/tree/ef648e01899ba3e8dc6371642deaaf64b4477775/skills/fastapi-patterns) | direct-with-caveats | backend-python |
| [ecc/flutter-dart-code-review](https://github.com/affaan-m/ECC/tree/ef648e01899ba3e8dc6371642deaaf64b4477775/skills/flutter-dart-code-review) | direct-with-caveats | mobile-flutter |
| [ecc/golang-testing](https://github.com/affaan-m/ECC/tree/ef648e01899ba3e8dc6371642deaaf64b4477775/skills/golang-testing) | direct-with-caveats | testing-go |
| [ecc/hexagonal-architecture](https://github.com/affaan-m/ECC/tree/ef648e01899ba3e8dc6371642deaaf64b4477775/skills/hexagonal-architecture) | direct-with-caveats | architecture |
| [ecc/investor-materials](https://github.com/affaan-m/ECC/tree/ef648e01899ba3e8dc6371642deaaf64b4477775/skills/investor-materials) | direct-with-caveats | business-fundraising |
| [ecc/investor-outreach](https://github.com/affaan-m/ECC/tree/ef648e01899ba3e8dc6371642deaaf64b4477775/skills/investor-outreach) | direct-with-caveats | business-fundraising |
| [ecc/jpa-patterns](https://github.com/affaan-m/ECC/tree/ef648e01899ba3e8dc6371642deaaf64b4477775/skills/jpa-patterns) | direct-with-caveats | database-java |
| [ecc/kotlin-coroutines-flows](https://github.com/affaan-m/ECC/tree/ef648e01899ba3e8dc6371642deaaf64b4477775/skills/kotlin-coroutines-flows) | direct-with-caveats | concurrency |
| [ecc/kubernetes-patterns](https://github.com/affaan-m/ECC/tree/ef648e01899ba3e8dc6371642deaaf64b4477775/skills/kubernetes-patterns) | direct-with-caveats | infrastructure |
| [ecc/laravel-patterns](https://github.com/affaan-m/ECC/tree/ef648e01899ba3e8dc6371642deaaf64b4477775/skills/laravel-patterns) | direct-with-caveats | backend-php |
| [ecc/latency-critical-systems](https://github.com/affaan-m/ECC/tree/ef648e01899ba3e8dc6371642deaaf64b4477775/skills/latency-critical-systems) | direct-with-caveats | performance |
| [ecc/market-research](https://github.com/affaan-m/ECC/tree/ef648e01899ba3e8dc6371642deaaf64b4477775/skills/market-research) | direct-with-caveats | research-business |
| [ecc/marketing-campaign](https://github.com/affaan-m/ECC/tree/ef648e01899ba3e8dc6371642deaaf64b4477775/skills/marketing-campaign) | direct-with-caveats | marketing-strategy |
| [ecc/mysql-patterns](https://github.com/affaan-m/ECC/tree/ef648e01899ba3e8dc6371642deaaf64b4477775/skills/mysql-patterns) | direct-with-caveats | databases |
| [ecc/nestjs-patterns](https://github.com/affaan-m/ECC/tree/ef648e01899ba3e8dc6371642deaaf64b4477775/skills/nestjs-patterns) | direct-with-caveats | backend-typescript |
| [ecc/postgres-patterns](https://github.com/affaan-m/ECC/tree/ef648e01899ba3e8dc6371642deaaf64b4477775/skills/postgres-patterns) | adapt-first | databases |
| [ecc/prisma-patterns](https://github.com/affaan-m/ECC/tree/ef648e01899ba3e8dc6371642deaaf64b4477775/skills/prisma-patterns) | adapt-first | databases |
| [ecc/production-audit](https://github.com/affaan-m/ECC/tree/ef648e01899ba3e8dc6371642deaaf64b4477775/skills/production-audit) | direct-with-caveats | release-readiness |
| [ecc/python-testing](https://github.com/affaan-m/ECC/tree/ef648e01899ba3e8dc6371642deaaf64b4477775/skills/python-testing) | adapt-first | testing-python |
| [ecc/rails-patterns](https://github.com/affaan-m/ECC/tree/ef648e01899ba3e8dc6371642deaaf64b4477775/skills/rails-patterns) | direct-with-caveats | backend-ruby |
| [ecc/react-native-patterns](https://github.com/affaan-m/ECC/tree/ef648e01899ba3e8dc6371642deaaf64b4477775/skills/react-native-patterns) | direct-with-caveats | mobile-react-native |
| [ecc/react-testing](https://github.com/affaan-m/ECC/tree/ef648e01899ba3e8dc6371642deaaf64b4477775/skills/react-testing) | direct-with-caveats | testing-web |
| [ecc/redis-patterns](https://github.com/affaan-m/ECC/tree/ef648e01899ba3e8dc6371642deaaf64b4477775/skills/redis-patterns) | adapt-first | databases |
| [ecc/research-ops](https://github.com/affaan-m/ECC/tree/ef648e01899ba3e8dc6371642deaaf64b4477775/skills/research-ops) | requires-companions | research |
| [ecc/rust-testing](https://github.com/affaan-m/ECC/tree/ef648e01899ba3e8dc6371642deaaf64b4477775/skills/rust-testing) | direct-with-caveats | testing-rust |
| [ecc/springboot-patterns](https://github.com/affaan-m/ECC/tree/ef648e01899ba3e8dc6371642deaaf64b4477775/skills/springboot-patterns) | direct-with-caveats | backend-java |
| [ecc/swift-actor-persistence](https://github.com/affaan-m/ECC/tree/ef648e01899ba3e8dc6371642deaaf64b4477775/skills/swift-actor-persistence) | adapt-first | mobile-apple |
| [ecc/swift-protocol-di-testing](https://github.com/affaan-m/ECC/tree/ef648e01899ba3e8dc6371642deaaf64b4477775/skills/swift-protocol-di-testing) | direct-with-caveats | testing-apple |
| [ecc/swiftui-patterns](https://github.com/affaan-m/ECC/tree/ef648e01899ba3e8dc6371642deaaf64b4477775/skills/swiftui-patterns) | direct-with-caveats | mobile-apple |
| [marketing/co-marketing](https://github.com/coreyhaines31/marketingskills/tree/dda3841f0b294e01e93b1541486beefbfab0915e/skills/co-marketing) | direct-with-caveats | marketing-partnerships |
| [marketing/community-marketing](https://github.com/coreyhaines31/marketingskills/tree/dda3841f0b294e01e93b1541486beefbfab0915e/skills/community-marketing) | direct-with-caveats | marketing-community |
| [marketing/competitors](https://github.com/coreyhaines31/marketingskills/tree/dda3841f0b294e01e93b1541486beefbfab0915e/skills/competitors) | direct-with-caveats | marketing-competitive-intelligence |
| [marketing/cro](https://github.com/coreyhaines31/marketingskills/tree/dda3841f0b294e01e93b1541486beefbfab0915e/skills/cro) | direct-with-caveats | marketing-conversion |
| [marketing/emails](https://github.com/coreyhaines31/marketingskills/tree/dda3841f0b294e01e93b1541486beefbfab0915e/skills/emails) | direct-with-caveats | marketing-email |
| [marketing/launch](https://github.com/coreyhaines31/marketingskills/tree/dda3841f0b294e01e93b1541486beefbfab0915e/skills/launch) | direct-with-caveats | marketing-launch |
| [marketing/onboarding](https://github.com/coreyhaines31/marketingskills/tree/dda3841f0b294e01e93b1541486beefbfab0915e/skills/onboarding) | direct-with-caveats | product-onboarding |
| [marketing/pricing](https://github.com/coreyhaines31/marketingskills/tree/dda3841f0b294e01e93b1541486beefbfab0915e/skills/pricing) | direct-with-caveats | marketing-pricing |
| [marketing/product-marketing](https://github.com/coreyhaines31/marketingskills/tree/dda3841f0b294e01e93b1541486beefbfab0915e/skills/product-marketing) | direct-with-caveats | marketing-positioning |
| [marketing/referrals](https://github.com/coreyhaines31/marketingskills/tree/dda3841f0b294e01e93b1541486beefbfab0915e/skills/referrals) | direct-with-caveats | marketing-referrals |
| [marketing/revops](https://github.com/coreyhaines31/marketingskills/tree/dda3841f0b294e01e93b1541486beefbfab0915e/skills/revops) | direct-with-caveats | marketing-operations |
| [awesome/grpc-golang](https://github.com/sickn33/agentic-awesome-skills/tree/bdacd76ed9e388733b5f91a5c75a4e8183a7c0b5/skills/grpc-golang) | direct-with-caveats | backend-go |
| [awesome/k6-load-testing](https://github.com/sickn33/agentic-awesome-skills/tree/bdacd76ed9e388733b5f91a5c75a4e8183a7c0b5/skills/k6-load-testing) | adapt-first | testing-performance |
| [awesome/observability-engineer](https://github.com/sickn33/agentic-awesome-skills/tree/bdacd76ed9e388733b5f91a5c75a4e8183a7c0b5/skills/observability-engineer) | direct-with-caveats | observability |
| [awesome/postgres-readonly-queries](https://github.com/sickn33/agentic-awesome-skills/tree/bdacd76ed9e388733b5f91a5c75a4e8183a7c0b5/skills/postgres-readonly-queries) | adapt-first | database-postgresql |
| [awesome/postgresql](https://github.com/sickn33/agentic-awesome-skills/tree/bdacd76ed9e388733b5f91a5c75a4e8183a7c0b5/skills/postgresql) | adapt-first | database-postgresql |
| [awesome/redis](https://github.com/sickn33/agentic-awesome-skills/tree/bdacd76ed9e388733b5f91a5c75a4e8183a7c0b5/skills/redis) | adapt-first | database-redis |
| [awesome/rust-async-patterns](https://github.com/sickn33/agentic-awesome-skills/tree/bdacd76ed9e388733b5f91a5c75a4e8183a7c0b5/skills/rust-async-patterns) | adapt-first | concurrency |
| [awesome/security-auditor](https://github.com/sickn33/agentic-awesome-skills/tree/bdacd76ed9e388733b5f91a5c75a4e8183a7c0b5/skills/security-auditor) | direct-with-caveats | security |
| [awesome/sre-dashboards](https://github.com/sickn33/agentic-awesome-skills/tree/bdacd76ed9e388733b5f91a5c75a4e8183a7c0b5/skills/sre-dashboards) | adapt-first | observability |
| [awesome/terraform-aws](https://github.com/sickn33/agentic-awesome-skills/tree/bdacd76ed9e388733b5f91a5c75a4e8183a7c0b5/skills/terraform-aws) | license-review | infrastructure |
| [awesome/terraform-infrastructure](https://github.com/sickn33/agentic-awesome-skills/tree/bdacd76ed9e388733b5f91a5c75a4e8183a7c0b5/skills/terraform-infrastructure) | requires-companions | infrastructure |
| [awesome/terraform-specialist](https://github.com/sickn33/agentic-awesome-skills/tree/bdacd76ed9e388733b5f91a5c75a4e8183a7c0b5/skills/terraform-specialist) | direct-with-caveats | infrastructure |
| [awesome/threat-modeling-expert](https://github.com/sickn33/agentic-awesome-skills/tree/bdacd76ed9e388733b5f91a5c75a4e8183a7c0b5/skills/threat-modeling-expert) | direct-with-caveats | security |
| [adhd/i-have-adhd](https://github.com/ayghri/i-have-adhd/tree/839872f9d1cd634fed642b4589ce7226199cc15f/skills/i-have-adhd) | direct | productivity |
