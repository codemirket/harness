# Harness inventory

Generated from `d8013f53969db336d813274d1fbfba53a89ad0e2` by [extract.py](../../evals/harness-map/extract.py). Full contracts, provenance, locks and graph edges are in [inventory.json](inventory.json).

This is a declaration inventory, not an installed-capability or quality report.

## Roles

| ID | Lead skill | Optional support | Aliases |
| --- | --- | --- | --- |
| frontend-engineering | frontend-engineering | engineering-judgment, test-design | Frontend Engineer |
| backend-engineering | backend-engineering | database-systems, test-design | Backend Engineer |
| brand-guidelines | brand-guidelines | interface-design, marketing-writing, design-inspiration | Brand Designer |
| web-design | interface-design | frontend-engineering, brand-guidelines, design-inspiration | Frontend Designer, UI Designer, Web Designer, design-revision, Design Revisioning, Design Revision |
| mobile-design | mobile-design | interface-design | Mobile App Designer |
| seo | search-visibility | data-analysis | SEO Expert, SEO Analyst, GEO Specialist |
| deep-research | research-and-synthesis | document-parsing | Deep Researcher |
| animation | motion-design | page-transitions, advanced-motion, svg-animation, graphic-design, interface-design | Motion Designer |
| project-review | architecture-review | security-judgment, test-design | Architecture Reviewer |
| planning | work-planning | context-management | Project Manager, Project Planning |
| product-management | product-management | customer-research, experiment-design | Product Manager |
| database | database-systems | data-migrations | Database Admin, Database Administrator, DBA |
| systems-engineering | systems-engineering | architecture-review, security-judgment, release-operations, debugging | System Engineer, Systems Engineer, Site Reliability Engineer, SRE |
| devops-infrastructure | infrastructure-engineering | release-operations, ci-maintenance | Devops Engineer, Infrastructure Engineer |
| marketing | marketing-writing | customer-research, conversion-review, market-analysis, search-visibility, experiment-design | Marketing Expert |
| executive-strategy | executive-strategy | product-management, data-analysis | CEO, Chief Executive Officer |
| data-analysis | data-analysis | database-systems, research-and-synthesis | Data Analyst |
| project-scaffolding | project-scaffolding | architecture-review, test-design | Project Scaffolding |
| optimization | performance-engineering | frontend-engineering, database-systems | Optimization Engineer, Performance Engineer |
| financial-analysis | financial-analysis | data-analysis, executive-strategy, spreadsheet-analysis | CFO, Chief Financial Officer, Financial Analyst |
| technical-leadership | technical-leadership | architecture-review, systems-engineering, agent-coordination | CTO, Chief Technology Officer, Senior Team Lead Engineer, Engineering Lead |
| market-analysis | market-analysis | research-and-synthesis, data-analysis | Market Analyst Expert, Market Analyst |
| venture-validation | venture-validation | product-management, market-analysis, financial-analysis | Entrepreneur, Entreprenuer |
| localization | localization | document-workflow, test-design | Native Translator, Translator, Localization Specialist |
| spreadsheet-analysis | spreadsheet-analysis | data-analysis, office-authoring | Excel Expert, Spreadsheet Expert |
| mobile-engineering | mobile-engineering | mobile-design, backend-engineering, test-design | Mobile App Engineer, Mobile Engineer |
| it-operations | it-operations | debugging, security-judgment, systems-engineering | IT Expert, IT Engineer |
| documentation | document-workflow | document-parsing, office-authoring | Documentation Expert, Technical Writer |
| graphic-design | graphic-design | brand-guidelines, svg-creation, interface-design, design-inspiration | Graphic Designer, illustration, Illustration, Illustrator, visual-assets, Crafting Visual Assets, Visual Asset Designer |
| security-engineering | security-judgment | architecture-review, test-design, mcp-integration | Security Engineer, Application Security Engineer |
| quality-assurance | test-design | debugging, engineering-judgment, frontend-engineering, interface-design | QA Engineer, Quality Engineer, Software Test Engineer, interface-qa, Interface QA Engineering, QA Engineer for Design, Design QA, Visual QA |
| ai-engineering | ai-system-evaluation | backend-engineering, data-analysis, security-judgment, mcp-integration | AI Engineer, LLM Engineer, Agent Engineer |
| data-engineering | data-engineering | database-systems, data-migrations, data-analysis | Data Engineer, Analytics Engineer |
| accessibility-engineering | accessibility-engineering | frontend-engineering, mobile-design, test-design | Accessibility Engineer, Accessibility Specialist |
| user-research | customer-research | product-management, data-analysis, experiment-design | UX Researcher, User Researcher |
| desktop-engineering | desktop-integration | frontend-engineering, security-judgment, release-operations | Desktop App Engineer, Desktop Engineer |
| mcp-engineering | mcp-integration | backend-engineering, security-judgment, test-design | MCP Engineer, MCP Server Engineer |
| experimentation | experiment-design | data-analysis, product-management | Experimentation Analyst, Experiment Designer, A/B Testing Analyst |
| release-engineering | release-operations | ci-maintenance, infrastructure-engineering, data-migrations, systems-engineering | Release Engineer, Release Manager |
| data-migration | data-migrations | database-systems, data-engineering, release-operations | Data Migration Engineer, Database Migration Engineer, Migration Engineer |
| office-production | office-authoring | document-workflow, graphic-design, spreadsheet-analysis, document-parsing | Document Designer, Presentation Designer, Office Document Specialist |

## Worker workflows

| ID | Default role | Allowed tools | Required invocation | Verification tools |
| --- | --- | --- | --- | --- |
| analysis | deep-research | workspace_read, workspace_list |  |  |
| structured-response | data-analysis |  |  |  |
| source-implementation | backend-engineering | workspace_read, workspace_list, workspace_write | workspace_write, workspace_read | workspace_read |
| source-review | project-review | workspace_read, workspace_list | workspace_read |  |
| visual-review | quality-assurance | workspace_read, workspace_list, design/browser_screenshot, design/odiff_compare, inspect_image | inspect_image | inspect_image |
| web-interface | web-design | workspace_read, workspace_list, workspace_write, design/browser_screenshot, inspect_image | workspace_write, workspace_read, design/browser_screenshot, inspect_image | workspace_read, inspect_image |
| editable-interface | web-design | workspace_read, workspace_list, design/high_level_overview, design/penpot_api_info, design/execute_code, design/export_shape, design/import_image, inspect_image | design/high_level_overview, design/execute_code, design/export_shape, inspect_image | inspect_image |
| static-graphic | graphic-design | workspace_read, workspace_list, workspace_write, design/static_render, inspect_image | design/static_render, inspect_image | inspect_image |
| diagram | graphic-design | workspace_read, workspace_list, workspace_write, design/kroki_render, inspect_image | design/kroki_render, inspect_image | inspect_image |
| motion | animation | workspace_read, workspace_list, workspace_write, design/hyperframes_render, inspect_image | design/hyperframes_render, inspect_image | inspect_image |
| interactive-motion | animation | workspace_read, workspace_list, workspace_write, design/browser_screenshot, inspect_image | workspace_write, workspace_read, design/browser_screenshot, inspect_image | workspace_read, inspect_image |
| paged-media | office-production | workspace_read, workspace_list, workspace_write, design/typst_compile, inspect_image | design/typst_compile, inspect_image | inspect_image |
| vector-art | graphic-design | workspace_read, workspace_list, workspace_write, design/vector_render, inspect_image | design/vector_render, inspect_image, workspace_read | inspect_image |
| raster-processing | graphic-design | workspace_read, workspace_list, design/raster_transform, inspect_image | design/raster_transform, inspect_image | inspect_image |
| vector-optimization | graphic-design | workspace_read, workspace_list, design/svg_optimize, design/browser_screenshot, inspect_image | design/svg_optimize, design/browser_screenshot, inspect_image, workspace_read | inspect_image |
| pixel-comparison | quality-assurance | design/odiff_compare | design/odiff_compare | design/odiff_compare |
| design-tokens | brand-guidelines | workspace_read, workspace_list, design/tokens_compile | design/tokens_compile, workspace_read | workspace_read |

**Runtime exception:** `editable-interface` declares Penpot operations that the owned dispatcher currently rejects. See [tools.rs](../../src/router/tools.rs) and the main map.

## Profiles

| ID | Kind | Skills |
| --- | --- | --- |
| project-foundation | starter | architecture-review, debugging, test-design, release-operations |
| engineering-core | specialist | debugging, test-design, architecture-review, ci-maintenance |
| architecture | specialist | systems-engineering, matt-domain-modeling, matt-codebase-design, ecc-contract-first, ecc-architecture-decision-records, ecc-hexagonal-architecture |
| planning-matt | alternative | matt-grill-with-docs, matt-research, matt-to-spec, matt-to-tickets, matt-wayfinder |
| execution-matt | alternative | matt-implement-spec, matt-handoff |
| execution-superpowers | alternative | superpowers-writing-plans, superpowers-executing-plans, superpowers-dispatching-parallel-agents, superpowers-receiving-code-review |
| collaboration | specialist | agent-coordination, context-management, matt-handoff, matt-to-questionnaire |
| maintenance | specialist | ci-maintenance, matt-triage, matt-code-review, matt-retro, matt-writing-for-agents |
| frontend-web | starter | frontend-engineering, emil-mobile-native, emil-break-ui, motion-design, openai-playwright |
| full-stack | starter | backend-engineering, frontend-engineering, ecc-api-design, ecc-contract-first, database-systems, data-migrations, test-design, openai-security-best-practices, ci-maintenance, release-operations |
| backend-node | specialist | backend-engineering, ecc-nestjs-patterns, ecc-api-design, ecc-contract-first, ecc-api-connector |
| backend-python | specialist | backend-engineering, ecc-fastapi-patterns, ecc-django-patterns, ecc-api-design, test-design |
| backend-dotnet | specialist | backend-engineering, openai-aspnet-core, ecc-dotnet-patterns, ecc-api-design |
| backend-java | specialist | backend-engineering, ecc-springboot-patterns, ecc-jpa-patterns, ecc-api-design |
| backend-go | specialist | backend-engineering, awesome-grpc-golang, ecc-golang-testing, ecc-contract-first |
| backend-ruby | specialist | backend-engineering, ecc-rails-patterns, ecc-api-design, test-design |
| backend-php | specialist | backend-engineering, ecc-laravel-patterns, ecc-api-design, test-design |
| testing-web | specialist | test-design, ecc-react-testing, openai-playwright, emil-break-ui, ci-maintenance |
| testing-systems | specialist | ecc-cpp-testing, ecc-rust-testing, ecc-golang-testing, ecc-benchmark-optimization |
| mobile-apple | specialist | mobile-engineering, mobile-design, ecc-swiftui-patterns, ecc-swift-protocol-di-testing, test-design |
| mobile-android | specialist | mobile-engineering, mobile-design, ecc-android-clean-architecture, ecc-compose-multiplatform-patterns, ecc-kotlin-coroutines-flows, test-design |
| mobile-react-native | specialist | mobile-engineering, mobile-design, ecc-react-native-patterns, test-design, motion-design |
| mobile-flutter | specialist | mobile-engineering, mobile-design, ecc-flutter-dart-code-review, test-design, architecture-review |
| desktop-crossplatform | specialist | desktop-integration, frontend-engineering, test-design, release-operations |
| desktop-windows | specialist | openai-winui-app, desktop-integration, test-design, release-operations |
| databases | specialist | database-systems, data-migrations, ecc-mysql-patterns |
| integrations | specialist | ecc-contract-first, ecc-api-connector, openai-cli-creator, mcp-integration |
| security | specialist | openai-security-threat-model, awesome-security-auditor, openai-security-best-practices |
| performance | specialist | performance-engineering, ecc-benchmark-optimization, ecc-latency-critical-systems, awesome-observability-engineer |
| cloud-gcp | specialist | google-cloud-run-basics, google-cloud-sql-basics, google-cloud-logging-query-generation, release-operations |
| kubernetes | specialist | ecc-kubernetes-patterns, google-gke-basics, google-gke-reliability, google-gke-workload-troubleshooting |
| operations | starter | infrastructure-engineering, awesome-terraform-specialist, awesome-observability-engineer, ecc-production-audit, release-operations, ci-maintenance |
| data-analytics | specialist | data-analysis, openai-jupyter-notebook, google-bigquery-basics, google-bigquery-optimization |
| data-pipelines | specialist | data-engineering, document-parsing, google-managed-airflow-dag-authoring, data-migrations |
| ai-systems | specialist | ai-system-evaluation, mcp-integration, research-and-synthesis |
| research | starter | research-and-synthesis, document-parsing, google-retrieving-developer-knowledge, open-design-research-decision-room |
| documents | starter | office-authoring, document-parsing, openai-pdf |
| media | specialist | openai-transcribe, openai-speech |
| design-prototyping | specialist | design-inspiration, emil-prototype, open-design-reference-design-contract, open-design-login-flow, open-design-faq-page, emil-pick-ui-library |
| design-redesign | specialist | interface-design, emil-break-ui, emil-mobile-native, motion-design |
| marketing-strategy | starter | marketing-product-context, marketing-competitors, customer-research, ecc-market-research, marketing-pricing, marketing-launch, ecc-marketing-campaign |
| marketing-growth | specialist | experiment-design, conversion-review, search-visibility, marketing-cro, marketing-onboarding, marketing-emails, marketing-referrals, marketing-revops |
| marketing-content | specialist | ecc-brand-voice, ecc-article-writing, ecc-content-engine |
| marketing-community | specialist | marketing-community-marketing, marketing-co-marketing, ecc-content-engine |
| fundraising | specialist | ecc-investor-materials, ecc-investor-outreach, ecc-market-research |
| product-discovery | specialist | product-management, gstack-openclaw-office-hours, gstack-openclaw-ceo-review, customer-research, matt-domain-modeling |
| learning | specialist | matt-teach, matt-scaffold-exercises, matt-writing-for-agents |
| focus-mode | alternative | adhd-i-have-adhd |
| react-next | specialist | frontend-engineering, vercel-react-best-practices, test-design |
| react-composition | specialist | vercel-composition-patterns, addy-api-and-interface-design |
| react-native-vercel | specialist | mobile-engineering, mobile-design, vercel-react-native-skills, test-design |
| react-view-transitions | specialist | vercel-react-view-transitions, motion-design |
| web-quality | specialist | frontend-engineering, addy-code-review-and-quality, addy-performance-optimization, test-design |
| engineering-addy | alternative | addy-context-engineering, addy-planning-and-task-breakdown, addy-incremental-implementation, addy-debugging-and-error-recovery, addy-test-driven-development, addy-documentation-and-adrs |
| api-reliability | specialist | addy-api-and-interface-design, addy-security-and-hardening, addy-observability-and-instrumentation |
| idea-research | specialist | addy-idea-refine, addy-source-driven-development, research-and-synthesis |
| search-visibility | specialist | search-visibility |
| svg-studio | specialist | svg-creation, svg-animation |
| motion-studio | starter | graphic-design, motion-design, page-transitions, advanced-motion, svg-creation, svg-animation |
| portable-foundation | specialist | architecture-review, debugging, test-design, release-operations |
| brand-system | specialist | brand-guidelines, svg-creation |
| project-start | specialist | project-scaffolding |
| business-decisions | specialist | executive-strategy, product-management, data-analysis |

## Skills

| ID | Delivery | Category | Source or local body | Requires |
| --- | --- | --- | --- | --- |
| skill-catalog | embedded | catalog | [skills/skill-catalog/SKILL.md](../../skills/skill-catalog/SKILL.md) |  |
| engineering-judgment | embedded | coding | [skills/engineering-judgment/SKILL.md](../../skills/engineering-judgment/SKILL.md) |  |
| interface-design | embedded | design | [skills/interface-design/SKILL.md](../../skills/interface-design/SKILL.md) |  |
| marketing-writing | embedded | marketing | [skills/marketing-writing/SKILL.md](../../skills/marketing-writing/SKILL.md) |  |
| document-workflow | embedded | documents | [skills/document-workflow/SKILL.md](../../skills/document-workflow/SKILL.md) |  |
| debugging | embedded | coding | [skills/debugging/SKILL.md](../../skills/debugging/SKILL.md) |  |
| test-design | embedded | coding | [skills/test-design/SKILL.md](../../skills/test-design/SKILL.md) |  |
| architecture-review | embedded | coding | [skills/architecture-review/SKILL.md](../../skills/architecture-review/SKILL.md) |  |
| motion-design | embedded | design | [skills/motion-design/SKILL.md](../../skills/motion-design/SKILL.md) |  |
| svg-animation | embedded | design | [skills/svg-animation/SKILL.md](../../skills/svg-animation/SKILL.md) |  |
| svg-creation | embedded | design | [skills/svg-creation/SKILL.md](../../skills/svg-creation/SKILL.md) |  |
| experiment-design | embedded | marketing | [skills/experiment-design/SKILL.md](../../skills/experiment-design/SKILL.md) |  |
| conversion-review | embedded | marketing | [skills/conversion-review/SKILL.md](../../skills/conversion-review/SKILL.md) |  |
| customer-research | embedded | marketing | [skills/customer-research/SKILL.md](../../skills/customer-research/SKILL.md) |  |
| matt-domain-modeling | upstream | architecture | matt |  |
| emil-mobile-native | upstream | mobile-web | emil |  |
| emil-break-ui | upstream | ui-testing | emil |  |
| emil-ask-sonner | upstream | ui-library | emil |  |
| ecc-contract-first | upstream | api-contracts | ecc |  |
| ecc-api-connector | upstream | api-integration | ecc |  |
| ecc-api-design | upstream | api-design | ecc |  |
| ecc-benchmark-optimization | upstream | performance | ecc |  |
| ecc-market-research | upstream | research-business | ecc |  |
| marketing-product-context | upstream | marketing-positioning | marketing |  |
| marketing-competitors | upstream | marketing-competitive-intelligence | marketing |  |
| anthropic-mcp-builder | reference | coding | anthropic |  |
| native-document-tools | reference | documents | anthropic |  |
| matt-code-review | upstream | review | matt | matt-setup-matt-pocock-skills |
| matt-codebase-design | upstream | architecture | matt |  |
| matt-diagnosing-bugs | upstream | debugging | matt |  |
| matt-grill-with-docs | upstream | planning | matt | matt-grilling, matt-domain-modeling |
| matt-implement | upstream | implementation | matt | matt-tdd, matt-code-review |
| matt-implement-spec | upstream | orchestration | matt | matt-tdd, matt-code-review, matt-to-spec, matt-to-tickets, matt-setup-matt-pocock-skills |
| matt-improve-codebase-architecture | upstream | architecture | matt | matt-codebase-design, matt-grilling, matt-domain-modeling |
| matt-prototype | upstream | prototyping | matt |  |
| matt-research | upstream | research | matt |  |
| matt-retro | upstream | maintenance | matt | matt-writing-for-agents |
| matt-setup-matt-pocock-skills | upstream | configuration | matt |  |
| matt-tdd | upstream | testing | matt | matt-codebase-design |
| matt-to-spec | upstream | planning | matt | matt-setup-matt-pocock-skills |
| matt-to-tickets | upstream | planning | matt | matt-setup-matt-pocock-skills |
| matt-triage | upstream | maintenance | matt | matt-setup-matt-pocock-skills, matt-grilling, matt-domain-modeling |
| matt-wayfinder | upstream | orchestration | matt | matt-setup-matt-pocock-skills, matt-grilling, matt-domain-modeling, matt-research, matt-prototype |
| matt-wizard | upstream | operations | matt |  |
| matt-setup-ts-deep-modules | upstream | architecture | matt | matt-codebase-design |
| matt-git-guardrails-claude-code | upstream | safety | matt |  |
| matt-migrate-to-shoehorn | upstream | testing | matt |  |
| matt-scaffold-exercises | upstream | learning | matt |  |
| matt-setup-pre-commit | upstream | configuration | matt |  |
| matt-grill-me | upstream | planning | matt | matt-grilling |
| matt-grilling | upstream | planning | matt |  |
| matt-handoff | upstream | collaboration | matt |  |
| matt-teach | upstream | learning | matt |  |
| matt-to-questionnaire | upstream | collaboration | matt |  |
| matt-wait-what | upstream | communication | matt |  |
| matt-writing-for-agents | upstream | documentation | matt |  |
| superpowers-dispatching-parallel-agents | upstream | orchestration | superpowers |  |
| superpowers-executing-plans | upstream | orchestration | superpowers | superpowers-subagent-driven-development, superpowers-using-git-worktrees, superpowers-test-driven-development, superpowers-systematic-debugging, superpowers-verification-before-completion, superpowers-requesting-code-review, superpowers-finishing-a-development-branch |
| superpowers-finishing-a-development-branch | upstream | coding | superpowers |  |
| superpowers-receiving-code-review | upstream | coding | superpowers |  |
| superpowers-requesting-code-review | upstream | coding | superpowers |  |
| superpowers-subagent-driven-development | upstream | orchestration | superpowers | superpowers-using-git-worktrees, superpowers-requesting-code-review, superpowers-finishing-a-development-branch |
| superpowers-systematic-debugging | upstream | coding | superpowers | superpowers-test-driven-development, superpowers-verification-before-completion |
| superpowers-test-driven-development | upstream | coding | superpowers |  |
| superpowers-using-git-worktrees | upstream | coding | superpowers |  |
| superpowers-verification-before-completion | upstream | coding | superpowers |  |
| superpowers-writing-plans | upstream | coding | superpowers |  |
| gstack-openclaw-ceo-review | upstream | planning | gstack |  |
| gstack-openclaw-investigate | upstream | debugging | gstack |  |
| gstack-openclaw-office-hours | upstream | planning | gstack |  |
| gstack-openclaw-retro | upstream | maintenance | gstack |  |
| gstack-runtime-gstack | reference | lifecycle | gstack |  |
| gstack-runtime-autoplan | reference | orchestration | gstack |  |
| gstack-runtime-benchmark | reference | lifecycle | gstack |  |
| gstack-runtime-benchmark-models | reference | lifecycle | gstack |  |
| gstack-runtime-browse | reference | lifecycle | gstack |  |
| gstack-runtime-hackernews-frontpage | reference | lifecycle | gstack |  |
| gstack-runtime-canary | reference | lifecycle | gstack |  |
| gstack-runtime-careful | reference | lifecycle | gstack |  |
| gstack-runtime-codex | reference | lifecycle | gstack |  |
| gstack-runtime-context-restore | reference | orchestration | gstack |  |
| gstack-runtime-context-save | reference | orchestration | gstack |  |
| gstack-runtime-cso | reference | lifecycle | gstack |  |
| gstack-runtime-design-consultation | reference | lifecycle | gstack |  |
| gstack-runtime-design-html | reference | lifecycle | gstack |  |
| gstack-runtime-design-review | reference | lifecycle | gstack |  |
| gstack-runtime-design-shotgun | reference | lifecycle | gstack |  |
| gstack-runtime-deslop-shared-libs | reference | lifecycle | gstack |  |
| gstack-runtime-devex-review | reference | lifecycle | gstack |  |
| gstack-runtime-document-generate | reference | lifecycle | gstack |  |
| gstack-runtime-document-release | reference | lifecycle | gstack |  |
| gstack-runtime-freeze | reference | lifecycle | gstack |  |
| gstack-runtime-gstack-upgrade | reference | lifecycle | gstack |  |
| gstack-runtime-guard | reference | lifecycle | gstack |  |
| gstack-runtime-health | reference | lifecycle | gstack |  |
| gstack-runtime-investigate | reference | lifecycle | gstack |  |
| gstack-runtime-ios-clean | reference | lifecycle | gstack |  |
| gstack-runtime-ios-design-review | reference | lifecycle | gstack |  |
| gstack-runtime-ios-fix | reference | lifecycle | gstack |  |
| gstack-runtime-ios-qa | reference | lifecycle | gstack |  |
| gstack-runtime-ios-sync | reference | lifecycle | gstack |  |
| gstack-runtime-land-and-deploy | reference | lifecycle | gstack |  |
| gstack-runtime-landing-report | reference | lifecycle | gstack |  |
| gstack-runtime-learn | reference | lifecycle | gstack |  |
| gstack-runtime-make-pdf | reference | lifecycle | gstack |  |
| gstack-runtime-office-hours | reference | lifecycle | gstack |  |
| gstack-runtime-open-gstack-browser | reference | lifecycle | gstack |  |
| gstack-runtime-pair-agent | reference | orchestration | gstack |  |
| gstack-runtime-plan-ceo-review | reference | orchestration | gstack |  |
| gstack-runtime-plan-design-review | reference | orchestration | gstack |  |
| gstack-runtime-plan-devex-review | reference | orchestration | gstack |  |
| gstack-runtime-plan-eng-review | reference | orchestration | gstack |  |
| gstack-runtime-plan-tune | reference | orchestration | gstack |  |
| gstack-runtime-qa | reference | lifecycle | gstack |  |
| gstack-runtime-qa-only | reference | lifecycle | gstack |  |
| gstack-runtime-retro | reference | lifecycle | gstack |  |
| gstack-runtime-review | reference | lifecycle | gstack |  |
| gstack-runtime-scrape | reference | lifecycle | gstack |  |
| gstack-runtime-setup-browser-cookies | reference | lifecycle | gstack |  |
| gstack-runtime-setup-deploy | reference | lifecycle | gstack |  |
| gstack-runtime-setup-gbrain | reference | lifecycle | gstack |  |
| gstack-runtime-ship | reference | lifecycle | gstack |  |
| gstack-runtime-skillify | reference | lifecycle | gstack |  |
| gstack-runtime-spec | reference | orchestration | gstack |  |
| gstack-runtime-sync-gbrain | reference | lifecycle | gstack |  |
| gstack-runtime-test-audit | reference | lifecycle | gstack |  |
| gstack-runtime-unfreeze | reference | lifecycle | gstack |  |
| context-mode-context-mode | reference | context-management | context-mode |  |
| context-mode-ctx-doctor | reference | context-management | context-mode |  |
| context-mode-ctx-index | reference | context-management | context-mode |  |
| context-mode-ctx-insight | reference | context-management | context-mode |  |
| context-mode-ctx-purge | reference | context-management | context-mode |  |
| context-mode-ctx-search | reference | context-management | context-mode |  |
| context-mode-ctx-stats | reference | context-management | context-mode |  |
| context-mode-ctx-upgrade | reference | context-management | context-mode |  |
| graphify-tool | reference | architecture | graphify |  |
| emil-prototype | upstream | prototyping | emil |  |
| emil-pick-ui-library | upstream | ui-library | emil |  |
| emil-animation-vocabulary | upstream | motion | emil |  |
| emil-animate | reference | motion | emil |  |
| emil-emil-design-eng | reference | motion | emil |  |
| emil-review-animations | reference | motion | emil |  |
| emil-find-animation-opportunities | reference | motion | emil |  |
| emil-improve-animations | reference | motion | emil |  |
| emil-animate-expo | reference | motion | emil |  |
| emil-apple-design | reference | native-design | emil |  |
| emil-write-swift | reference | native-design | emil |  |
| ui-ux-pro-max-ui-ux-pro-max | reference | design-system | ui-ux-pro-max |  |
| ui-ux-pro-max-brand | reference | design-system | ui-ux-pro-max |  |
| ui-ux-pro-max-ui-styling | reference | design-tools | ui-ux-pro-max |  |
| hallmark-hallmark | reference | design-system | hallmark |  |
| open-design-reference-design-contract | upstream | design-system | open-design |  |
| open-design-research-decision-room | upstream | customer-research | open-design |  |
| open-design-login-flow | upstream | prototyping | open-design |  |
| open-design-faq-page | upstream | prototyping | open-design |  |
| gsap-reference | reference | motion | gsap-official |  |
| open-design-design-brief | reference | design-system | open-design |  |
| impeccable-impeccable | reference | design-system | impeccable |  |
| archify-archify | reference | diagrams | archify |  |
| ecc-android-clean-architecture | upstream | mobile-kotlin | ecc |  |
| ecc-architecture-decision-records | upstream | architecture-documentation | ecc |  |
| ecc-article-writing | upstream | writing-longform | ecc |  |
| ecc-brand-voice | upstream | writing-brand | ecc |  |
| ecc-compose-multiplatform-patterns | upstream | mobile-kotlin | ecc |  |
| ecc-content-engine | upstream | marketing-content | ecc |  |
| ecc-cpp-testing | upstream | testing-cpp | ecc |  |
| ecc-dart-flutter-patterns | reference | mobile-flutter | ecc |  |
| ecc-django-patterns | upstream | backend-python | ecc |  |
| ecc-docker-patterns | reference | containers | ecc |  |
| ecc-documentation-lookup | reference | research-developer | ecc |  |
| ecc-dotnet-patterns | upstream | backend-dotnet | ecc |  |
| ecc-fastapi-patterns | upstream | backend-python | ecc |  |
| ecc-flutter-dart-code-review | upstream | mobile-flutter | ecc |  |
| ecc-golang-testing | upstream | testing-go | ecc |  |
| ecc-hexagonal-architecture | upstream | architecture | ecc |  |
| ecc-investor-materials | upstream | business-fundraising | ecc |  |
| ecc-investor-outreach | upstream | business-fundraising | ecc |  |
| ecc-jpa-patterns | upstream | database-java | ecc |  |
| ecc-kotlin-coroutines-flows | upstream | concurrency | ecc |  |
| ecc-kubernetes-patterns | upstream | infrastructure | ecc |  |
| ecc-laravel-patterns | upstream | backend-php | ecc |  |
| ecc-latency-critical-systems | upstream | performance | ecc |  |
| ecc-marketing-campaign | upstream | marketing-strategy | ecc |  |
| ecc-mysql-patterns | upstream | databases | ecc |  |
| ecc-nestjs-patterns | upstream | backend-typescript | ecc |  |
| ecc-postgres-patterns | reference | databases | ecc |  |
| ecc-prisma-patterns | reference | databases | ecc |  |
| ecc-production-audit | upstream | release-readiness | ecc |  |
| ecc-python-testing | reference | testing-python | ecc |  |
| ecc-rails-patterns | upstream | backend-ruby | ecc |  |
| ecc-react-native-patterns | upstream | mobile-react-native | ecc |  |
| ecc-react-testing | upstream | testing-web | ecc |  |
| ecc-redis-patterns | reference | databases | ecc |  |
| ecc-research-ops | reference | research | ecc |  |
| ecc-rust-testing | upstream | testing-rust | ecc |  |
| ecc-springboot-patterns | upstream | backend-java | ecc |  |
| ecc-swift-actor-persistence | reference | mobile-apple | ecc |  |
| ecc-swift-protocol-di-testing | upstream | testing-apple | ecc |  |
| ecc-swiftui-patterns | upstream | mobile-apple | ecc |  |
| marketing-co-marketing | upstream | marketing-partnerships | marketing |  |
| marketing-community-marketing | upstream | marketing-community | marketing |  |
| marketing-cro | upstream | marketing-conversion | marketing |  |
| marketing-emails | upstream | marketing-email | marketing |  |
| marketing-launch | upstream | marketing-launch | marketing |  |
| marketing-onboarding | upstream | product-onboarding | marketing |  |
| marketing-pricing | upstream | marketing-pricing | marketing |  |
| marketing-referrals | upstream | marketing-referrals | marketing |  |
| marketing-revops | upstream | marketing-operations | marketing |  |
| awesome-grpc-golang | upstream | backend-go | awesome |  |
| awesome-k6-load-testing | reference | testing-performance | awesome |  |
| awesome-observability-engineer | upstream | observability | awesome |  |
| awesome-postgres-readonly-queries | reference | database-postgresql | awesome |  |
| awesome-postgresql | reference | database-postgresql | awesome |  |
| awesome-redis | reference | database-redis | awesome |  |
| awesome-rust-async-patterns | reference | concurrency | awesome |  |
| awesome-security-auditor | upstream | security | awesome |  |
| awesome-sre-dashboards | reference | observability | awesome |  |
| awesome-terraform-aws | reference | infrastructure | awesome |  |
| awesome-terraform-infrastructure | reference | infrastructure | awesome |  |
| awesome-terraform-specialist | upstream | infrastructure | awesome |  |
| awesome-threat-modeling-expert | upstream | security | awesome |  |
| adhd-i-have-adhd | upstream | productivity | adhd |  |
| openai-aspnet-core | upstream | backend-dotnet | openai |  |
| openai-winui-app | upstream | desktop-windows | openai |  |
| openai-security-threat-model | upstream | security | openai |  |
| openai-security-best-practices | upstream | security | openai |  |
| openai-pdf | upstream | documents | openai |  |
| openai-cli-creator | upstream | integrations | openai |  |
| openai-jupyter-notebook | upstream | data-science | openai |  |
| openai-speech | upstream | media | openai |  |
| openai-transcribe | upstream | media | openai |  |
| openai-playwright | upstream | browser-testing | openai |  |
| openai-netlify-deploy | upstream | deployment | openai |  |
| google-cloud-run-basics | upstream | cloud-gcp | google |  |
| google-cloud-sql-basics | upstream | database | google |  |
| google-bigquery-basics | upstream | data-analytics | google |  |
| google-bigquery-optimization | upstream | data-analytics | google |  |
| google-gke-basics | upstream | cloud-kubernetes | google |  |
| google-gke-reliability | upstream | cloud-kubernetes | google |  |
| google-gke-workload-troubleshooting | upstream | operations | google |  |
| google-managed-airflow-dag-authoring | upstream | data-pipelines | google |  |
| google-retrieving-developer-knowledge | upstream | research | google |  |
| google-cloud-logging-query-generation | upstream | observability | google |  |
| google-developing-genkit-js | upstream | ai-engineering | google |  |
| google-developing-genkit-python | upstream | ai-engineering | google |  |
| caveman-voice | upstream | communication | caveman |  |
| research-and-synthesis | embedded | research | [skills/research-and-synthesis/SKILL.md](../../skills/research-and-synthesis/SKILL.md) |  |
| document-parsing | embedded | documents | [skills/document-parsing/SKILL.md](../../skills/document-parsing/SKILL.md) |  |
| agent-coordination | embedded | collaboration | [skills/agent-coordination/SKILL.md](../../skills/agent-coordination/SKILL.md) |  |
| context-management | embedded | context | [skills/context-management/SKILL.md](../../skills/context-management/SKILL.md) |  |
| desktop-integration | embedded | desktop | [skills/desktop-integration/SKILL.md](../../skills/desktop-integration/SKILL.md) |  |
| release-operations | embedded | deployment | [skills/release-operations/SKILL.md](../../skills/release-operations/SKILL.md) |  |
| data-migrations | embedded | database | [skills/data-migrations/SKILL.md](../../skills/data-migrations/SKILL.md) |  |
| office-authoring | embedded | documents | [skills/office-authoring/SKILL.md](../../skills/office-authoring/SKILL.md) |  |
| ci-maintenance | embedded | maintenance | [skills/ci-maintenance/SKILL.md](../../skills/ci-maintenance/SKILL.md) |  |
| mcp-integration | embedded | integrations | [skills/mcp-integration/SKILL.md](../../skills/mcp-integration/SKILL.md) |  |
| ai-system-evaluation | embedded | ai-engineering | [skills/ai-system-evaluation/SKILL.md](../../skills/ai-system-evaluation/SKILL.md) |  |
| frontend-engineering | embedded | frontend | [skills/frontend-engineering/SKILL.md](../../skills/frontend-engineering/SKILL.md) |  |
| database-systems | embedded | database | [skills/database-systems/SKILL.md](../../skills/database-systems/SKILL.md) |  |
| security-judgment | embedded | security | [skills/security-judgment/SKILL.md](../../skills/security-judgment/SKILL.md) |  |
| work-planning | embedded | planning | [skills/work-planning/SKILL.md](../../skills/work-planning/SKILL.md) |  |
| search-visibility | embedded | search | [skills/search-visibility/SKILL.md](../../skills/search-visibility/SKILL.md) |  |
| vercel-browser-agent-browser | package | browser-automation | vercel-agent-browser |  |
| vercel-browser-agentcore | package | cloud-automation | vercel-agent-browser |  |
| vercel-browser-core | package | browser-automation | vercel-agent-browser |  |
| vercel-browser-derive-client | reference | api-engineering | vercel-agent-browser |  |
| vercel-browser-dogfood | reference | testing | vercel-agent-browser |  |
| vercel-browser-electron | reference | desktop-automation | vercel-agent-browser |  |
| vercel-browser-protected-vercel-deployments | reference | deployment | vercel-agent-browser |  |
| vercel-browser-slack | reference | collaboration | vercel-agent-browser |  |
| vercel-browser-vercel-sandbox | package | cloud-automation | vercel-agent-browser |  |
| vercel-browser-webmcp-gen | reference | browser-automation | vercel-agent-browser |  |
| vercel-composition-patterns | upstream | frontend-engineering | vercel-agent-skills |  |
| vercel-deploy-to-vercel | reference | deployment | vercel-agent-skills |  |
| vercel-react-best-practices | upstream | frontend-engineering | vercel-agent-skills |  |
| vercel-react-native-skills | upstream | react-native | vercel-agent-skills |  |
| vercel-react-view-transitions | upstream | frontend-engineering | vercel-agent-skills |  |
| vercel-vercel-cli-with-tokens | reference | deployment | vercel-agent-skills |  |
| vercel-vercel-optimize | package | performance | vercel-agent-skills |  |
| vercel-web-design-guidelines | reference | interface-design | vercel-agent-skills |  |
| vercel-writing-guidelines | reference | technical-writing | vercel-agent-skills |  |
| addy-api-and-interface-design | upstream | api-design | addy |  |
| addy-browser-testing-with-devtools | reference | engineering-workflow | addy |  |
| addy-ci-cd-and-automation | reference | engineering-workflow | addy |  |
| addy-code-review-and-quality | upstream | code-review | addy |  |
| addy-code-simplification | reference | engineering-workflow | addy |  |
| addy-constraint-driven-development | reference | engineering-workflow | addy |  |
| addy-context-engineering | upstream | agent-context | addy |  |
| addy-debugging-and-error-recovery | upstream | debugging | addy |  |
| addy-deprecation-and-migration | reference | engineering-workflow | addy |  |
| addy-documentation-and-adrs | upstream | architecture-documentation | addy |  |
| addy-doubt-driven-development | reference | engineering-workflow | addy |  |
| addy-frontend-ui-engineering | reference | engineering-workflow | addy |  |
| addy-git-workflow-and-versioning | reference | engineering-workflow | addy |  |
| addy-idea-refine | upstream | product-discovery | addy |  |
| addy-incremental-implementation | upstream | implementation | addy |  |
| addy-interview-me | reference | engineering-workflow | addy |  |
| addy-observability-and-instrumentation | upstream | observability | addy |  |
| addy-performance-optimization | upstream | performance | addy |  |
| addy-planning-and-task-breakdown | upstream | planning | addy |  |
| addy-security-and-hardening | upstream | security | addy |  |
| addy-shipping-and-launch | reference | engineering-workflow | addy |  |
| addy-source-driven-development | upstream | technical-research | addy |  |
| addy-spec-driven-development | reference | engineering-workflow | addy |  |
| addy-test-driven-development | upstream | testing | addy |  |
| addy-using-agent-skills | reference | engineering-workflow | addy |  |
| planning-with-files | reference | planning | planning-with-files |  |
| geo-seo-claude-geo | reference | marketing | geo-seo-claude |  |
| geo-seo-claude-geo-audit | reference | marketing | geo-seo-claude |  |
| geo-seo-claude-geo-brand-mentions | reference | marketing | geo-seo-claude |  |
| geo-seo-claude-geo-citability | reference | marketing | geo-seo-claude |  |
| geo-seo-claude-geo-compare | reference | marketing | geo-seo-claude |  |
| geo-seo-claude-geo-content | reference | marketing | geo-seo-claude |  |
| geo-seo-claude-geo-crawlers | reference | marketing | geo-seo-claude |  |
| geo-seo-claude-geo-llmstxt | reference | marketing | geo-seo-claude |  |
| geo-seo-claude-geo-platform-optimizer | reference | marketing | geo-seo-claude |  |
| geo-seo-claude-geo-proposal | reference | marketing | geo-seo-claude |  |
| geo-seo-claude-geo-prospect | reference | marketing | geo-seo-claude |  |
| geo-seo-claude-geo-report | reference | marketing | geo-seo-claude |  |
| geo-seo-claude-geo-report-pdf | reference | marketing | geo-seo-claude |  |
| geo-seo-claude-geo-schema | reference | marketing | geo-seo-claude |  |
| geo-seo-claude-geo-technical | reference | marketing | geo-seo-claude |  |
| geo-seo-claude-geo-update | reference | marketing | geo-seo-claude |  |
| page-transitions | embedded | design | [skills/page-transitions/SKILL.md](../../skills/page-transitions/SKILL.md) |  |
| advanced-motion | embedded | design | [skills/advanced-motion/SKILL.md](../../skills/advanced-motion/SKILL.md) |  |
| product-management | embedded | planning | [skills/product-management/SKILL.md](../../skills/product-management/SKILL.md) |  |
| data-analysis | embedded | data | [skills/data-analysis/SKILL.md](../../skills/data-analysis/SKILL.md) |  |
| backend-engineering | embedded | engineering | [skills/backend-engineering/SKILL.md](../../skills/backend-engineering/SKILL.md) |  |
| systems-engineering | embedded | engineering | [skills/systems-engineering/SKILL.md](../../skills/systems-engineering/SKILL.md) |  |
| infrastructure-engineering | embedded | operations | [skills/infrastructure-engineering/SKILL.md](../../skills/infrastructure-engineering/SKILL.md) |  |
| project-scaffolding | embedded | engineering | [skills/project-scaffolding/SKILL.md](../../skills/project-scaffolding/SKILL.md) |  |
| performance-engineering | embedded | engineering | [skills/performance-engineering/SKILL.md](../../skills/performance-engineering/SKILL.md) |  |
| brand-guidelines | embedded | design | [skills/brand-guidelines/SKILL.md](../../skills/brand-guidelines/SKILL.md) |  |
| mobile-design | embedded | design | [skills/mobile-design/SKILL.md](../../skills/mobile-design/SKILL.md) |  |
| executive-strategy | embedded | planning | [skills/executive-strategy/SKILL.md](../../skills/executive-strategy/SKILL.md) |  |
| financial-analysis | embedded | planning | [skills/financial-analysis/SKILL.md](../../skills/financial-analysis/SKILL.md) |  |
| market-analysis | embedded | research | [skills/market-analysis/SKILL.md](../../skills/market-analysis/SKILL.md) |  |
| venture-validation | embedded | planning | [skills/venture-validation/SKILL.md](../../skills/venture-validation/SKILL.md) |  |
| localization | embedded | documents | [skills/localization/SKILL.md](../../skills/localization/SKILL.md) |  |
| spreadsheet-analysis | embedded | documents | [skills/spreadsheet-analysis/SKILL.md](../../skills/spreadsheet-analysis/SKILL.md) |  |
| graphic-design | embedded | design | [skills/graphic-design/SKILL.md](../../skills/graphic-design/SKILL.md) |  |
| technical-leadership | embedded | engineering | [skills/technical-leadership/SKILL.md](../../skills/technical-leadership/SKILL.md) |  |
| mobile-engineering | embedded | engineering | [skills/mobile-engineering/SKILL.md](../../skills/mobile-engineering/SKILL.md) |  |
| it-operations | embedded | engineering | [skills/it-operations/SKILL.md](../../skills/it-operations/SKILL.md) |  |
| data-engineering | embedded | data | [skills/data-engineering/SKILL.md](../../skills/data-engineering/SKILL.md) |  |
| accessibility-engineering | embedded | accessibility | [skills/accessibility-engineering/SKILL.md](../../skills/accessibility-engineering/SKILL.md) |  |
| design-inspiration | embedded | design | [skills/design-inspiration/SKILL.md](../../skills/design-inspiration/SKILL.md) |  |

## Source pins

| Source | Repository / URL | Commit | License summary |
| --- | --- | --- | --- |
| graphify | https://github.com/Graphify-Labs/graphify/tree/5c7b84792f453582676548185aaec3824d51dfe2 | 5c7b84792f453582676548185aaec3824d51dfe2 | Apache-2.0 with retained MIT portions |
| marketing | https://github.com/coreyhaines31/marketingskills/tree/dda3841f0b294e01e93b1541486beefbfab0915e | dda3841f0b294e01e93b1541486beefbfab0915e | MIT |
| ui-ux-pro-max | https://github.com/nextlevelbuilder/ui-ux-pro-max-skill/tree/477bcb28c9812b385cb51a4605ddf30d7b2266e2 | 477bcb28c9812b385cb51a4605ddf30d7b2266e2 | MIT |
| hallmark | https://github.com/Nutlope/hallmark/tree/13ac0ec7e148655948100b6396439e481361d690 | 13ac0ec7e148655948100b6396439e481361d690 | MIT |
| taste | https://github.com/Leonxlnx/taste-skill/tree/f624f37a8c650b9cc550253104925efe4c8de246 | f624f37a8c650b9cc550253104925efe4c8de246 | MIT |
| anthropic | https://github.com/anthropics/skills/tree/683bc88e56f3e09ba94f7055977f3d3aa499f202 | 683bc88e56f3e09ba94f7055977f3d3aa499f202 | Per-skill; office document implementations have restrictive source-available licenses |
| superpowers | https://github.com/obra/superpowers/tree/8ca22dba9a94f28898bbce59f2537ff4d87c747d | 8ca22dba9a94f28898bbce59f2537ff4d87c747d | MIT |
| matt | https://github.com/mattpocock/skills/tree/4588b32ecab9ecc9fc8cc6b6c5e7d675b6004b0d | 4588b32ecab9ecc9fc8cc6b6c5e7d675b6004b0d | MIT (PR skill also copied third-party material; attribution retained) |
| ecc | https://github.com/affaan-m/ECC/tree/ef648e01899ba3e8dc6371642deaaf64b4477775 | ef648e01899ba3e8dc6371642deaaf64b4477775 | MIT |
| caveman | https://github.com/JuliusBrussee/caveman/tree/99aafe151a1be72be783e662858e8a0955add59f | 99aafe151a1be72be783e662858e8a0955add59f | Apache-2.0 with retained notices |
| archify | https://github.com/tt-a1i/archify/tree/73aaa0696e8f72c232ea710e6fa94fd953f3e773 | 73aaa0696e8f72c232ea710e6fa94fd953f3e773 | MIT; separate asset licenses |
| emil | https://github.com/emilkowalski/skills/tree/e8a175de22ae1e49370fc144c1f3bb9aeedf988d | e8a175de22ae1e49370fc144c1f3bb9aeedf988d | MIT |
| gstack | https://github.com/garrytan/gstack/tree/c285d88b90d39116ccfa2b901f80ea0fce0b26eb | c285d88b90d39116ccfa2b901f80ea0fce0b26eb | MIT; main design package also Apache-2.0 notices |
| context-mode | https://github.com/mksglu/context-mode/tree/e5fcca6802d484db3f4ec04f6d2021938852b2a0 | e5fcca6802d484db3f4ec04f6d2021938852b2a0 | Elastic-2.0 |
| front-end-checklist | https://github.com/thedaviddias/Front-End-Checklist/tree/e8d14d049010e2ce25afcd5e8609dfcbbe63e47e | e8d14d049010e2ce25afcd5e8609dfcbbe63e47e | MIT |
| open-design | https://github.com/nexu-io/open-design/tree/53231d40b778d88eba23f35547bf99485d3ae9fc | 53231d40b778d88eba23f35547bf99485d3ae9fc | Apache-2.0 root; file-specific upstream declarations |
| impeccable | https://github.com/pbakaus/impeccable/tree/4e8504f10106a1cd7a99e37a401aa368d1d55576 | 4e8504f10106a1cd7a99e37a401aa368d1d55576 | Apache-2.0; MIT-derived platform references |
| awesome | https://github.com/sickn33/agentic-awesome-skills/tree/bdacd76ed9e388733b5f91a5c75a4e8183a7c0b5 | bdacd76ed9e388733b5f91a5c75a4e8183a7c0b5 | CC-BY-4.0 for original non-code content; MIT tooling; per-source exceptions |
| adhd | https://github.com/ayghri/i-have-adhd/tree/839872f9d1cd634fed642b4589ce7226199cc15f | 839872f9d1cd634fed642b4589ce7226199cc15f | MIT |
| openai | https://github.com/openai/skills/tree/49f948faa9258a0c61caceaf225e179651397431 | 49f948faa9258a0c61caceaf225e179651397431 | Per-skill Apache-2.0; deprecated in favor of openai/plugins |
| google | https://github.com/google/skills/tree/d6eee396ed6a51871cc3f0979c5ff1be26d35d10 | d6eee396ed6a51871cc3f0979c5ff1be26d35d10 | Apache-2.0 |
| openai-plugins | https://github.com/openai/plugins/tree/5fd93af4cd0c623e020d0cc7e9ce178b4ac1f70f | 5fd93af4cd0c623e020d0cc7e9ce178b4ac1f70f | Per plugin; no blanket root license; some manifests declare MIT without bundled terms |
| vercel-agent-browser | https://github.com/vercel-labs/agent-browser/tree/6d3e22c673a44271d0c213c2fef722e0aeba627d | 6d3e22c673a44271d0c213c2fef722e0aeba627d | Apache-2.0; bundled MPL-2.0 component and retained third-party notices |
| vercel-agent-skills | https://github.com/vercel-labs/agent-skills/tree/063bee94c3f4df8453406c830b0a7df0f2860278 | 063bee94c3f4df8453406c830b0a7df0f2860278 | MIT declared in root README and React skill frontmatter; no standalone LICENSE or NOTICE in this revision |
| addy | https://github.com/addyosmani/agent-skills/tree/1401c8b8030e023baeebb31781a6653fe8e93026 | 1401c8b8030e023baeebb31781a6653fe8e93026 | MIT |
| planning-with-files | https://github.com/OthmanAdi/planning-with-files/tree/dab9d16fbd9314448b319d112e99f497d7638d89 | dab9d16fbd9314448b319d112e99f497d7638d89 | MIT |
| geo-seo-claude | https://github.com/zubair-trabzada/geo-seo-claude/tree/989cae01e8ebbc42a9ec798eb9e7cb423f5ec89c | 989cae01e8ebbc42a9ec798eb9e7cb423f5ec89c | MIT |
| ibelick-ui | https://github.com/ibelick/ui-skills/tree/9a0cb1e610e3a1a4f0f2723bf2e7680ac1b89cf5 | 9a0cb1e610e3a1a4f0f2723bf2e7680ac1b89cf5 | MIT (selected public repository material only) |
| lottiefiles-motion | https://github.com/LottieFiles/motion-design-skill/tree/f9a8a041b85185ee4881b3471d3415e939aac772 | f9a8a041b85185ee4881b3471d3415e939aac772 | MIT (selected public repository material only) |
| kyle-motion | https://github.com/kylezantos/design-motion-principles/tree/4a9ca879f24a361f4dca4174fe2da0f67b5ddee3 | 4a9ca879f24a361f4dca4174fe2da0f67b5ddee3 | MIT (selected public repository material only) |
| owl-concepts | https://github.com/Owl-Listener/designer-skills/tree/9a6930cf84a822eb458624bd11c61aac5bbdf224 | 9a6930cf84a822eb458624bd11c61aac5bbdf224 | MIT (selected public repository material only) |
| libraries-craft | https://github.com/Jakubantalik/Libraries.dev/tree/a27566557b4def79bcdcedc71858c0fb28020b6d | a27566557b4def79bcdcedc71858c0fb28020b6d | MIT (selected public repository material only) |
| mengto-inspiration | https://github.com/MengTo/Skills/tree/83a47fee32f0b6349bff1fede99257a5ef03dc93 | 83a47fee32f0b6349bff1fede99257a5ef03dc93 | MIT (selected repository guidance; third-party works and assets retain separate rights) |
| wondel-discovery-positioning | https://github.com/wondelai/skills/tree/c172996495bed0fcd26896a9416b2093fd7073f0 | c172996495bed0fcd26896a9416b2093fd7073f0 | MIT (selected repository guidance; third-party works and assets retain separate rights) |
| scroll-craft | https://github.com/nateherkai/scroll-craft/tree/75d81f74e83692add18cd7a8a8e078b8a887a579 | 75d81f74e83692add18cd7a8a8e078b8a887a579 | MIT (selected repository techniques; external client assets and service outputs retain separate rights) |
| gsap-official | https://github.com/greensock/gsap-skills/tree/aed9cfd3277740755f6bfc1155c7aa645403b760 | aed9cfd3277740755f6bfc1155c7aa645403b760 | MIT |
| diagram-design | https://github.com/cathrynlavery/diagram-design/tree/550fd66edff058bc73056c3416b76d69cfe865ba | 550fd66edff058bc73056c3416b76d69cfe865ba | MIT for selected methods; bundled third-party assets excluded |
| motion-primitives | https://github.com/ibelick/motion-primitives/tree/120f64f6ca60348e251f929e9c81f11ccbe45eda | 120f64f6ca60348e251f929e9c81f11ccbe45eda | README claims MIT but linked LICENSE.md is absent at reviewed revision |
| canvas-ui | https://github.com/DavidHDev/canvas-ui/tree/a04a99b235299ee79a123a16ccb2e6cf9da74b6b | a04a99b235299ee79a123a16ccb2e6cf9da74b6b | Custom MIT + Commons Clause; component redistribution restricted |
| better-design | https://github.com/marvkr/better-design/tree/e146625f3041ef8089a2013587435fc763dc3b15 | e146625f3041ef8089a2013587435fc763dc3b15 | MIT for reviewed public material |
| knowledge-work-design | https://github.com/anthropics/knowledge-work-plugins/tree/33eeda1e8fd856455b05479d13a0f85b5f803744 | 33eeda1e8fd856455b05479d13a0f85b5f803744 | Apache-2.0 for selected design guidance; other plugin licenses are separate |
| design-anatomy | https://github.com/wheresryan22/anatomy/tree/1482e8469b0a9e0d8cc18ea2740a01423d29ae80 | 1482e8469b0a9e0d8cc18ea2740a01423d29ae80 | MIT |
| editorial-illustration | https://github.com/helloianneo/ian-xiaohei-illustrations/tree/4102eb807f03bcb6e538a16e8b31b41db8b5b954 | 4102eb807f03bcb6e538a16e8b31b41db8b5b954 | MIT |
| anime-runtime-review | https://github.com/juliangarnier/anime/tree/01b81be1df6843ccfe0a71c0699a746bf740dd77 | 01b81be1df6843ccfe0a71c0699a746bf740dd77 | MIT |
| motion-runtime-review | https://github.com/motiondivision/motion/tree/c5fab238ab4f8d8132d1cb05d6607439b3904058 | c5fab238ab4f8d8132d1cb05d6607439b3904058 | MIT |
| radix-primitives-review | https://github.com/radix-ui/primitives/tree/eb3a2b7091293e27df8f9ba906632d8daa3d81a3 | eb3a2b7091293e27df8f9ba906632d8daa3d81a3 | MIT |
| design-md-review | https://github.com/voltagent/awesome-design-md/tree/3b605329240ce735b510037cca322f1831fdd7f8 | 3b605329240ce735b510037cca322f1831fdd7f8 | MIT |

## Team facets

| Facet | Applies when | Assignments |
| --- | --- | --- |
| product-discovery | The task includes deciding which user problem, journey or product scope to pursue. | product-management:produce, user-research:advise |
| web-interface | The task builds or substantially revises a working browser screen or public page. | web-design:produce, frontend-engineering:produce, quality-assurance:review |
| mobile-journey | The task changes a native or mobile-web journey, input or lifecycle behavior. | mobile-design:produce, mobile-engineering:produce, quality-assurance:review |
| desktop-application | The task changes an installed desktop feature or its OS integration. | desktop-engineering:produce, quality-assurance:review |
| service-contract | The task changes API, worker or service behavior with real consumers. | backend-engineering:produce, quality-assurance:review |
| persistent-data | The task changes query behavior, schema design, transactions or database operations. | database:produce |
| data-pipeline | The task builds ingestion or transformation with published datasets. | data-engineering:produce, data-analysis:review |
| data-migration | The task changes persisted representations with backfill or cutover. | data-migration:produce, database:review |
| security-boundary | The task changes trust, authorization, secrets or privileged tool effects. | security-engineering:review |
| systems-reliability | The task changes topology, capacity, queues or failure containment. | systems-engineering:produce |
| infrastructure-change | The task changes desired environment configuration or infrastructure resources. | devops-infrastructure:produce |
| release-rollout | The requested scope includes release preparation, promotion or authorized rollout. | release-engineering:produce, quality-assurance:review |
| performance-work | The task requests a measured latency, throughput, resource or cost improvement. | optimization:produce |
| ai-feature | The task changes an LLM, retrieval or tool-using application feature. | ai-engineering:produce, quality-assurance:review |
| mcp-tools | The task develops an MCP client/server or exposes tools and resources. | mcp-engineering:produce, security-engineering:review |
| brand-identity | The deliverable defines or evolves brand identity across actual applications. | brand-guidelines:produce, graphic-design:produce |
| visual-assets | The deliverable includes graphics, illustration or a coherent asset family. | graphic-design:produce |
| motion-behavior | The task adds meaningful motion or animation to a user experience. | animation:produce, quality-assurance:review |
| accessibility | The task includes accessibility requirements, an audit or an affected inclusive journey. | accessibility-engineering:produce, quality-assurance:review |
| market-opportunity | The decision depends on market boundaries, competition or reachable demand. | market-analysis:produce |
| venture-validation | The task tests a business thesis before committing resources. | venture-validation:produce, market-analysis:advise |
| financial-decision | The requested decision depends on economics, cash timing or financing. | financial-analysis:produce |
| quantitative-analysis | The task interprets product or business data, reconciles metrics or explains a trend. | data-analysis:produce |
| experiment-analysis | The task designs or interprets a causal experiment. | experimentation:produce, data-analysis:review |
| customer-study | The task studies user behavior or synthesizes qualitative customer evidence. | user-research:produce |
| marketing-message | The deliverable includes positioning, offer or persuasive channel copy. | marketing:produce |
| search-visibility | The task investigates organic or AI search crawl, indexing or discoverability. | seo:produce |
| localized-content | The deliverable translates or adapts content for a locale. | localization:produce |
| office-artifact | The requested output is an editable document or presentation with layout requirements. | office-production:produce, documentation:review |
| workbook | The requested deliverable is a spreadsheet or connected workbook with formulas. | spreadsheet-analysis:produce |
| technical-documentation | The task creates or substantially revises API/repository documentation or technical explanation. | documentation:produce |
| architecture-change | The task needs a consequential structural boundary or architecture review. | project-review:advise, technical-leadership:produce |
| delivery-coordination | The task spans interdependent workstreams with uncertain sequencing or ownership. | planning:produce |
| company-strategy | The decision concerns company-wide allocation, concentration or strategic direction. | executive-strategy:produce |
| project-bootstrap | The task creates a new project/package or runnable scaffold. | project-scaffolding:produce |
| it-repair | The task diagnoses or repairs a workstation, network or existing service path. | it-operations:produce |
| evidence-research | The task requires a source-grounded synthesis of uncertain external facts. | deep-research:produce |

## Embedded reference files

All files under authored skill `references/` directories, including data and specimens. This count is distinct from catalog delivery `reference` entries and the 27 design-reference instances.

- [skills/advanced-motion/references/art-direction-and-medium.md](../../skills/advanced-motion/references/art-direction-and-medium.md)
- [skills/advanced-motion/references/connected-layers.md](../../skills/advanced-motion/references/connected-layers.md)
- [skills/advanced-motion/references/engine-selection.md](../../skills/advanced-motion/references/engine-selection.md)
- [skills/advanced-motion/references/gsap-integration.md](../../skills/advanced-motion/references/gsap-integration.md)
- [skills/advanced-motion/references/local-video.md](../../skills/advanced-motion/references/local-video.md)
- [skills/advanced-motion/references/rendered-examples.md](../../skills/advanced-motion/references/rendered-examples.md)
- [skills/advanced-motion/references/scene-mechanics.md](../../skills/advanced-motion/references/scene-mechanics.md)
- [skills/advanced-motion/references/scroll-media-and-verification.md](../../skills/advanced-motion/references/scroll-media-and-verification.md)
- [skills/advanced-motion/references/surface-and-procedural-craft.md](../../skills/advanced-motion/references/surface-and-procedural-craft.md)
- [skills/agent-coordination/references/claude-code.md](../../skills/agent-coordination/references/claude-code.md)
- [skills/agent-coordination/references/model-routing.md](../../skills/agent-coordination/references/model-routing.md)
- [skills/agent-coordination/references/task-evidence.md](../../skills/agent-coordination/references/task-evidence.md)
- [skills/ai-system-evaluation/references/feedback-controls.md](../../skills/ai-system-evaluation/references/feedback-controls.md)
- [skills/ai-system-evaluation/references/harness-development.md](../../skills/ai-system-evaluation/references/harness-development.md)
- [skills/backend-engineering/references/command-service.md](../../skills/backend-engineering/references/command-service.md)
- [skills/brand-guidelines/references/fieldwork-system.md](../../skills/brand-guidelines/references/fieldwork-system.md)
- [skills/context-management/references/context-lifecycle.md](../../skills/context-management/references/context-lifecycle.md)
- [skills/context-management/references/repository-retrieval.md](../../skills/context-management/references/repository-retrieval.md)
- [skills/customer-research/references/usability-studies.md](../../skills/customer-research/references/usability-studies.md)
- [skills/data-analysis/references/grain-and-comparisons.md](../../skills/data-analysis/references/grain-and-comparisons.md)
- [skills/data-engineering/references/replay-and-late-events.md](../../skills/data-engineering/references/replay-and-late-events.md)
- [skills/database-systems/references/document-databases.md](../../skills/database-systems/references/document-databases.md)
- [skills/database-systems/references/postgresql.md](../../skills/database-systems/references/postgresql.md)
- [skills/database-systems/references/redis.md](../../skills/database-systems/references/redis.md)
- [skills/database-systems/references/sqlite.md](../../skills/database-systems/references/sqlite.md)
- [skills/design-inspiration/references/design-languages/android-adaptive-continuity.md](../../skills/design-inspiration/references/design-languages/android-adaptive-continuity.md)
- [skills/design-inspiration/references/design-languages/apple-liquid-glass.md](../../skills/design-inspiration/references/design-languages/apple-liquid-glass.md)
- [skills/design-inspiration/references/design-languages/aria-alternative-input.md](../../skills/design-inspiration/references/design-languages/aria-alternative-input.md)
- [skills/design-inspiration/references/design-languages/atlassian-collaboration.md](../../skills/design-inspiration/references/design-languages/atlassian-collaboration.md)
- [skills/design-inspiration/references/design-languages/carbon-operations.md](../../skills/design-inspiration/references/design-languages/carbon-operations.md)
- [skills/design-inspiration/references/design-languages/casa-subject-identity.md](../../skills/design-inspiration/references/design-languages/casa-subject-identity.md)
- [skills/design-inspiration/references/design-languages/comparison-at-use-size.md](../../skills/design-inspiration/references/design-languages/comparison-at-use-size.md)
- [skills/design-inspiration/references/design-languages/coverage.json](../../skills/design-inspiration/references/design-languages/coverage.json)
- [skills/design-inspiration/references/design-languages/coverage.md](../../skills/design-inspiration/references/design-languages/coverage.md)
- [skills/design-inspiration/references/design-languages/duolingo-performance.md](../../skills/design-inspiration/references/design-languages/duolingo-performance.md)
- [skills/design-inspiration/references/design-languages/fiori-complex-workspace.md](../../skills/design-inspiration/references/design-languages/fiori-complex-workspace.md)
- [skills/design-inspiration/references/design-languages/fluent-dimensional-symbols.md](../../skills/design-inspiration/references/design-languages/fluent-dimensional-symbols.md)
- [skills/design-inspiration/references/design-languages/fluent-spatial.md](../../skills/design-inspiration/references/design-languages/fluent-spatial.md)
- [skills/design-inspiration/references/design-languages/ft-relationship-notation.md](../../skills/design-inspiration/references/design-languages/ft-relationship-notation.md)
- [skills/design-inspiration/references/design-languages/geist-code.md](../../skills/design-inspiration/references/design-languages/geist-code.md)
- [skills/design-inspiration/references/design-languages/govuk-procedural.md](../../skills/design-inspiration/references/design-languages/govuk-procedural.md)
- [skills/design-inspiration/references/design-languages/guardian-editorial.md](../../skills/design-inspiration/references/design-languages/guardian-editorial.md)
- [skills/design-inspiration/references/design-languages/ibm-hybrid-illustration.md](../../skills/design-inspiration/references/design-languages/ibm-hybrid-illustration.md)
- [skills/design-inspiration/references/design-languages/index.md](../../skills/design-inspiration/references/design-languages/index.md)
- [skills/design-inspiration/references/design-languages/instances/android-adaptive-continuity.json](../../skills/design-inspiration/references/design-languages/instances/android-adaptive-continuity.json)
- [skills/design-inspiration/references/design-languages/instances/apple-liquid-glass.json](../../skills/design-inspiration/references/design-languages/instances/apple-liquid-glass.json)
- [skills/design-inspiration/references/design-languages/instances/aria-alternative-input.json](../../skills/design-inspiration/references/design-languages/instances/aria-alternative-input.json)
- [skills/design-inspiration/references/design-languages/instances/atlassian-collaboration.json](../../skills/design-inspiration/references/design-languages/instances/atlassian-collaboration.json)
- [skills/design-inspiration/references/design-languages/instances/carbon-operations.json](../../skills/design-inspiration/references/design-languages/instances/carbon-operations.json)
- [skills/design-inspiration/references/design-languages/instances/casa-subject-identity.json](../../skills/design-inspiration/references/design-languages/instances/casa-subject-identity.json)
- [skills/design-inspiration/references/design-languages/instances/duolingo-performance.json](../../skills/design-inspiration/references/design-languages/instances/duolingo-performance.json)
- [skills/design-inspiration/references/design-languages/instances/fiori-complex-workspace.json](../../skills/design-inspiration/references/design-languages/instances/fiori-complex-workspace.json)
- [skills/design-inspiration/references/design-languages/instances/fluent-dimensional-symbols.json](../../skills/design-inspiration/references/design-languages/instances/fluent-dimensional-symbols.json)
- [skills/design-inspiration/references/design-languages/instances/fluent-spatial.json](../../skills/design-inspiration/references/design-languages/instances/fluent-spatial.json)
- [skills/design-inspiration/references/design-languages/instances/ft-relationship-notation.json](../../skills/design-inspiration/references/design-languages/instances/ft-relationship-notation.json)
- [skills/design-inspiration/references/design-languages/instances/geist-code.json](../../skills/design-inspiration/references/design-languages/instances/geist-code.json)
- [skills/design-inspiration/references/design-languages/instances/govuk-procedural.json](../../skills/design-inspiration/references/design-languages/instances/govuk-procedural.json)
- [skills/design-inspiration/references/design-languages/instances/guardian-editorial.json](../../skills/design-inspiration/references/design-languages/instances/guardian-editorial.json)
- [skills/design-inspiration/references/design-languages/instances/ibm-hybrid-illustration.json](../../skills/design-inspiration/references/design-languages/instances/ibm-hybrid-illustration.json)
- [skills/design-inspiration/references/design-languages/instances/material-responsive-shape.json](../../skills/design-inspiration/references/design-languages/instances/material-responsive-shape.json)
- [skills/design-inspiration/references/design-languages/instances/mit-generative-identity.json](../../skills/design-inspiration/references/design-languages/instances/mit-generative-identity.json)
- [skills/design-inspiration/references/design-languages/instances/ons-statistical.json](../../skills/design-inspiration/references/design-languages/instances/ons-statistical.json)
- [skills/design-inspiration/references/design-languages/instances/peeps-humanist.json](../../skills/design-inspiration/references/design-languages/instances/peeps-humanist.json)
- [skills/design-inspiration/references/design-languages/instances/penpot-direct-manipulation.json](../../skills/design-inspiration/references/design-languages/instances/penpot-direct-manipulation.json)
- [skills/design-inspiration/references/design-languages/instances/porsche-restraint.json](../../skills/design-inspiration/references/design-languages/instances/porsche-restraint.json)
- [skills/design-inspiration/references/design-languages/instances/pudding-subject-story.json](../../skills/design-inspiration/references/design-languages/instances/pudding-subject-story.json)
- [skills/design-inspiration/references/design-languages/instances/saleor-commerce.json](../../skills/design-inspiration/references/design-languages/instances/saleor-commerce.json)
- [skills/design-inspiration/references/design-languages/instances/spectrum-bidirectional.json](../../skills/design-inspiration/references/design-languages/instances/spectrum-bidirectional.json)
- [skills/design-inspiration/references/design-languages/instances/vanilla-publishing.json](../../skills/design-inspiration/references/design-languages/instances/vanilla-publishing.json)
- [skills/design-inspiration/references/design-languages/instances/vega-linked-analysis.json](../../skills/design-inspiration/references/design-languages/instances/vega-linked-analysis.json)
- [skills/design-inspiration/references/design-languages/instances/wise-expressive-utility.json](../../skills/design-inspiration/references/design-languages/instances/wise-expressive-utility.json)
- [skills/design-inspiration/references/design-languages/library.json](../../skills/design-inspiration/references/design-languages/library.json)
- [skills/design-inspiration/references/design-languages/material-responsive-shape.md](../../skills/design-inspiration/references/design-languages/material-responsive-shape.md)
- [skills/design-inspiration/references/design-languages/mit-generative-identity.md](../../skills/design-inspiration/references/design-languages/mit-generative-identity.md)
- [skills/design-inspiration/references/design-languages/ons-statistical.md](../../skills/design-inspiration/references/design-languages/ons-statistical.md)
- [skills/design-inspiration/references/design-languages/peeps-humanist.md](../../skills/design-inspiration/references/design-languages/peeps-humanist.md)
- [skills/design-inspiration/references/design-languages/penpot-direct-manipulation.md](../../skills/design-inspiration/references/design-languages/penpot-direct-manipulation.md)
- [skills/design-inspiration/references/design-languages/porsche-restraint.md](../../skills/design-inspiration/references/design-languages/porsche-restraint.md)
- [skills/design-inspiration/references/design-languages/pudding-subject-story.md](../../skills/design-inspiration/references/design-languages/pudding-subject-story.md)
- [skills/design-inspiration/references/design-languages/recovery-feedback.md](../../skills/design-inspiration/references/design-languages/recovery-feedback.md)
- [skills/design-inspiration/references/design-languages/saleor-commerce.md](../../skills/design-inspiration/references/design-languages/saleor-commerce.md)
- [skills/design-inspiration/references/design-languages/spectrum-bidirectional.md](../../skills/design-inspiration/references/design-languages/spectrum-bidirectional.md)
- [skills/design-inspiration/references/design-languages/vanilla-publishing.md](../../skills/design-inspiration/references/design-languages/vanilla-publishing.md)
- [skills/design-inspiration/references/design-languages/vega-linked-analysis.md](../../skills/design-inspiration/references/design-languages/vega-linked-analysis.md)
- [skills/design-inspiration/references/design-languages/wise-expressive-utility.md](../../skills/design-inspiration/references/design-languages/wise-expressive-utility.md)
- [skills/design-inspiration/references/reference-record.md](../../skills/design-inspiration/references/reference-record.md)
- [skills/document-workflow/references/technical-documentation.md](../../skills/document-workflow/references/technical-documentation.md)
- [skills/engineering-judgment/references/boundary-decisions.md](../../skills/engineering-judgment/references/boundary-decisions.md)
- [skills/engineering-judgment/references/service-integration.md](../../skills/engineering-judgment/references/service-integration.md)
- [skills/executive-strategy/references/allocation-decision.md](../../skills/executive-strategy/references/allocation-decision.md)
- [skills/experiment-design/references/assignment-integrity.md](../../skills/experiment-design/references/assignment-integrity.md)
- [skills/financial-analysis/references/cash-and-earnings.md](../../skills/financial-analysis/references/cash-and-earnings.md)
- [skills/frontend-engineering/references/async-state.md](../../skills/frontend-engineering/references/async-state.md)
- [skills/graphic-design/references/art-direction-and-critique.md](../../skills/graphic-design/references/art-direction-and-critique.md)
- [skills/graphic-design/references/constructed-illustration.md](../../skills/graphic-design/references/constructed-illustration.md)
- [skills/graphic-design/references/design-documents.md](../../skills/graphic-design/references/design-documents.md)
- [skills/graphic-design/references/diagram-semantics.md](../../skills/graphic-design/references/diagram-semantics.md)
- [skills/graphic-design/references/image-production.md](../../skills/graphic-design/references/image-production.md)
- [skills/graphic-design/references/production-and-proof.md](../../skills/graphic-design/references/production-and-proof.md)
- [skills/infrastructure-engineering/references/environment-change.md](../../skills/infrastructure-engineering/references/environment-change.md)
- [skills/interface-design/references/application-composition.md](../../skills/interface-design/references/application-composition.md)
- [skills/interface-design/references/art-direction-and-critique.md](../../skills/interface-design/references/art-direction-and-critique.md)
- [skills/interface-design/references/brief-to-proof.md](../../skills/interface-design/references/brief-to-proof.md)
- [skills/interface-design/references/concept-exploration.md](../../skills/interface-design/references/concept-exploration.md)
- [skills/interface-design/references/design-evidence.md](../../skills/interface-design/references/design-evidence.md)
- [skills/interface-design/references/image-to-code.md](../../skills/interface-design/references/image-to-code.md)
- [skills/interface-design/references/public-page-composition.md](../../skills/interface-design/references/public-page-composition.md)
- [skills/interface-design/references/typography-proof.md](../../skills/interface-design/references/typography-proof.md)
- [skills/interface-design/references/workbench.md](../../skills/interface-design/references/workbench.md)
- [skills/it-operations/references/service-path.md](../../skills/it-operations/references/service-path.md)
- [skills/localization/references/message-integrity.md](../../skills/localization/references/message-integrity.md)
- [skills/market-analysis/references/market-boundaries.md](../../skills/market-analysis/references/market-boundaries.md)
- [skills/market-analysis/references/positioning-evidence.md](../../skills/market-analysis/references/positioning-evidence.md)
- [skills/marketing-writing/references/editorial-preservation.md](../../skills/marketing-writing/references/editorial-preservation.md)
- [skills/mcp-integration/references/protocol-completion.md](../../skills/mcp-integration/references/protocol-completion.md)
- [skills/mobile-design/references/materials-journey.md](../../skills/mobile-design/references/materials-journey.md)
- [skills/mobile-engineering/references/interrupted-order.md](../../skills/mobile-engineering/references/interrupted-order.md)
- [skills/motion-design/references/interaction-recipes.md](../../skills/motion-design/references/interaction-recipes.md)
- [skills/motion-design/references/motion-audit.md](../../skills/motion-design/references/motion-audit.md)
- [skills/office-authoring/references/production.md](../../skills/office-authoring/references/production.md)
- [skills/page-transitions/references/navigation-craft.md](../../skills/page-transitions/references/navigation-craft.md)
- [skills/performance-engineering/references/controlled-comparison.md](../../skills/performance-engineering/references/controlled-comparison.md)
- [skills/product-management/references/decision-to-delivery.md](../../skills/product-management/references/decision-to-delivery.md)
- [skills/project-scaffolding/references/staged-generator.md](../../skills/project-scaffolding/references/staged-generator.md)
- [skills/research-and-synthesis/references/claim-verification.md](../../skills/research-and-synthesis/references/claim-verification.md)
- [skills/security-judgment/references/agent-tool-boundaries.md](../../skills/security-judgment/references/agent-tool-boundaries.md)
- [skills/skill-catalog/references/delivery-standards.md](../../skills/skill-catalog/references/delivery-standards.md)
- [skills/skill-catalog/references/task-execution.md](../../skills/skill-catalog/references/task-execution.md)
- [skills/skill-catalog/references/task-routing.md](../../skills/skill-catalog/references/task-routing.md)
- [skills/spreadsheet-analysis/references/recalculation-evidence.md](../../skills/spreadsheet-analysis/references/recalculation-evidence.md)
- [skills/svg-animation/references/svg-motion-craft.md](../../skills/svg-animation/references/svg-motion-craft.md)
- [skills/svg-creation/references/vector-craft.md](../../skills/svg-creation/references/vector-craft.md)
- [skills/systems-engineering/references/reliability-capacity.md](../../skills/systems-engineering/references/reliability-capacity.md)
- [skills/technical-leadership/references/platform-change.md](../../skills/technical-leadership/references/platform-change.md)
- [skills/test-design/references/interface-qa.md](../../skills/test-design/references/interface-qa.md)
- [skills/test-design/references/quality-assurance.md](../../skills/test-design/references/quality-assurance.md)
- [skills/venture-validation/references/pilot-evidence.md](../../skills/venture-validation/references/pilot-evidence.md)
- [skills/work-planning/references/artifact-review.md](../../skills/work-planning/references/artifact-review.md)
- [skills/work-planning/references/delivery-review.md](../../skills/work-planning/references/delivery-review.md)
