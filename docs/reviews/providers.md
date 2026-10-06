# Provider-source review

Reviewed 2026-10-06 at the exact revisions in the registry. This is static source assessment: no cloud resource, paid API, upstream helper, plugin runtime or dependency installer was executed.

## OpenAI skills and the official successor

The requested [openai/skills README](https://github.com/openai/skills) marks the repository deprecated and directs users to [openai/plugins](https://github.com/openai/plugins). The registry keeps reviewed older skill snapshots reproducible and indexes the successor as an additional source. It does not pretend every successor plugin is portable or fully audited.

Selected directory bodies were read for ASP.NET Core, WinUI, threat modeling, security guidance, PDF, CLI creation, notebooks, speech, transcription, Playwright and Netlify. Full reads included the ASP.NET substantive reference set, threat-model references, CLI design reference, notebook generator, speech/transcription helpers and Playwright wrapper. Other large reference packs were structurally inventoried with selected inspection; the catalog records that limit per entry. No benchmark or runtime certification is implied.

Concrete integration corrections are prepended and hashed independently: resolve files from the installed project skill instead of hard-coded CODEX_HOME; WinUI bootstrap changes the machine and is explicit environment setup; Netlify production flags require production authorization; native Windows needs appropriate shell/tool equivalents. Notebook helpers need an explicit output path. The speech batch helper accepts parent traversal in output names, so untrusted jobs need validation. Transcription can overwrite outputs and its dry-run can print base64 speaker samples; unique outputs and log hygiene are required. The Playwright wrapper invokes an unpinned npx package, so its runtime needs a separate versioned installation decision.

The bundled GitHub helpers were read and deliberately not selected: failed checks can return nonzero despite usable JSON, while the CI helper treats this as retrieval failure; comments helper uses the head repository for a fork PR and resets completed pagination cursors. The authored `ci-maintenance` workflow covers reliable retrieval, revision identity and diagnosis without importing those defects. Security examples are reference material: unpredictable IDs never replace authorization and HSTS needs deployment-specific reasoning.

Per-skill OpenAI LICENSE files are retained. The successor repository has no blanket root license; several package manifests declare MIT without a bundled license text. Expo includes explicit MIT terms. Native iOS/macOS and account-dependent plugins remain searchable integration options pending package/license/runtime review, not fabricated standalone skills.

## Google skills

The Google snapshot has Apache-2.0 root terms. Canonical inventory uses `skills/` and omits duplicate plugin copies. Selected full entry bodies cover Cloud Run, Cloud SQL, BigQuery basics/optimization, GKE basics/reliability/troubleshooting, Airflow DAG authoring, Developer Knowledge retrieval, Cloud Logging queries and Genkit JS/Python. All Cloud Run references were read; other selected reference trees were inventoried with partial content inspection. Self-contained GKE reliability, workload diagnosis and DAG instructions received full-body review.

Cloud examples contain public IAM, administrative roles, fixed regions and API-enablement commands. The catalog requires actual account/region/permission scope, not copying those as defaults. Cloud SQL password-in-argv/URI examples must use secure credential mechanisms. BigQuery optimization should resolve the actual dataset location; attribution tags are a telemetry policy choice. Optional companion recommendations do not authorize running a suite installer. Autopilot CPU minimums vary with current configuration; the snapshot's fixed example is not universal. Workload diagnosis cannot establish a root cause from offline symptoms, and its automatic branch/commit/PR steps need existing authorization. Genkit provider/global CLI defaults remain subordinate to the project. Public official docs can substitute for Developer Knowledge access when only factual lookup is needed.

## Anthropic and Caveman

The earlier Anthropic source review remains relevant. Document implementations have restrictive per-skill licenses and are not vendored. MCP builder's main workflow was read; templates, scripts and full reference package remain a manual selection. Original office authoring, document parsing and MCP integration fill the portable workflow need, while actual document rendering uses available host tools.

Caveman's main concise-voice skill was fully read and now has an explicit optional project entry. It preserves substantive work, user language, facts and normal persisted artifacts. No persistence hook or compressor is installed. Ultra/wenyan aliases are not supplied implicitly. Other compression/cloud/hook skills remain inventoried. Concise presentation is a preference, not evidence of better engineering or lower total task cost.
