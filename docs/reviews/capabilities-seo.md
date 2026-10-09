# SEO assurance and capability contract review — 2026-10-09

This exercise observes actual local HTTP responses and checks whether the diagnosis
distinguishes crawl permission, indexing directives, canonical signals and unknown
search outcomes. It is a repeatable behavior check, not a live Google indexing test
or proof that an agent will correctly audit any website.

## Method and evidence

Read the local [search-visibility](../../skills/search-visibility/SKILL.md) and
[search-audit](../../skills/search-audit/SKILL.md) guidance, then current Google
primary documentation. The exercise uses the standard-library executable
[SEO fixture](../../examples/assurance/seo.py). It binds an ephemeral port on
`127.0.0.1`, serves fictional HTML/robots/sitemap responses, fetches the actual
responses without ambient HTTP proxies, and closes the server after the checks.
No packages, accounts, production settings or external search submissions are used.

```sh
python3 examples/assurance/seo.py --output build/capability-seo/run-new
```

The output directory must not exist. It receives response bodies, status/headers,
retrieval times and body hashes, plus `report.json` with URL diagnoses, checks,
source hash and limits. Completed evidence is under ignored
`build/capability-seo/run-03/`. These are local review artifacts; a fresh clone can
reproduce them with the command above.

The test separates the HTML fixture from a fixed expected diagnosis table. A
deliberately bad classifier ignores meta/header and canonical signals, relying on
robots permission alone; the expected table rejects it. A second probe removes a
real HTTP response's meta directive and confirms that the diagnosis changes.
This catches a sensor that merely classifies a URL by its name. All 12 checks
pass, including rejection of the known-wrong rule. The exercise is still authored
development evidence, not a held-out model benchmark or independent Google parser.

## Observed URL-level diagnoses

Paths below refer to the run's loopback origin stored in `report.json`. Every
listed page returned HTTP 200. Actual Google indexing and Google-selected canonical
remain **unknown** for all pages; Search Console was unavailable.

| URL path | Actual response/control evidence | Supported diagnosis | Proportionate next action if this were an owned public site |
| --- | --- | --- | --- |
| `/blocked/private` | `Disallow: /blocked/`; fetched HTML contains meta `noindex` | Crawl disallowed. The local inspector can see `noindex`, but a robots-compliant crawler cannot fetch it. Do not claim confirmed exclusion from Google. | First resolve owner intent. For private content use access control; if the goal is search exclusion via `noindex`, let the crawler retrieve the directive. Do not blindly unblock private/training-restricted pages. |
| `/blocked/public` | More-specific `Allow: /blocked/public`; no `noindex` | Allowed by this fixture's rules. No local indexing block detected. | Retain intended exception and verify the actual site's accessibility and other signals. |
| `/meta-noindex` | Robots allowed; meta `robots` = `noindex, follow` | Indexing is prohibited by the page directive once fetched; robots allowance is insufficient. | Preserve intentional exclusion, or remove the directive if the authorized owner wants indexing; verify the resulting response. |
| `/header-noindex` | Robots allowed; `X-Robots-Tag: noindex` | Header directive prohibits indexing once fetched; an HTML-only audit would miss it. | Inspect the header's application/server/CDN owner before changing it and verify the final response. |
| `/canonical-conflict` | HTML canonical points at `/preferred-a`; HTTP `Link` canonical points at `/preferred-b`; both targets return 200 | Conflicting canonical preferences. Neither target can be declared Google's selected canonical from these observations. | Choose the intended duplicate representative, then align canonical sources and internal/sitemap signals. |
| `/preferred-a`, `/preferred-b` | HTTP 200, self-canonical, no observed indexing prohibition | No local indexing block detected. These controls do not establish inclusion or selection. | Check actual duplication, public accessibility and provider observations if relevant. |
| `/clear-controls` | HTTP 200, readable HTML, robots allowed, self-canonical and sitemap inclusion | No local indexing block detected; actual index status remains unknown. Even on a public site, those observations would not prove indexing. | Obtain authorized URL Inspection/indexing evidence if an outcome claim is needed. |

The sitemap lists `/preferred-a` and `/clear-controls`; it is observed context, not
evidence of indexing. HTML is static and server-rendered in this fixture. No browser
render, JavaScript state, verified Googlebot IP, CDN policy, authentication or
Core Web Vitals measurement was tested. Loopback is not publicly reachable, so
the fixture must not be reported as meeting all public Google indexability criteria.

## Primary-source basis

Google distinguishes crawl controls from keeping a URL out of results; a blocked
URL can still be discovered through links. This supports the restricted conclusion
for `/blocked/private`, not a claim that it actually appears in results.
[Google robots introduction](https://developers.google.com/search/docs/crawling-indexing/robots/intro)
(retrieved 2026-10-09; displayed update 2025-12-10).

Google requires crawler access to observe `noindex`, and supports the rule in
both HTML metadata and HTTP response headers. The same documentation does not
support putting a `noindex` rule in robots.txt.
[Google indexing controls](https://developers.google.com/search/docs/crawling-indexing/block-indexing)
(retrieved 2026-10-09; displayed update 2025-12-10).

The more-specific matching robots path wins; on an equal-specificity conflict
Google uses the less restrictive rule. The fixture exercises only one wildcard
user-agent group with literal ASCII prefix paths. Its small helper is explicitly
not a general Google robots parser: multiple agent groups, wildcard paths,
percent-encoding and provider retrieval/error policies are outside coverage.
[Google robots specification](https://developers.google.com/crawling/docs/robots-txt/robots-txt-spec)
(retrieved 2026-10-09; displayed update 2026-08-31).

Google documents canonical methods as signals and warns about inconsistent HTML
and HTTP declarations. The fixture detects disagreement without predicting which
signal Google will choose.
[Canonical documentation](https://developers.google.com/search/docs/crawling-indexing/consolidate-duplicate-urls)
(retrieved 2026-10-09; displayed update 2026-07-10).

Meeting minimum technical requirements does not guarantee indexing. The local
fixture therefore reports a narrow observed control state and leaves actual
indexing unknown.
[Google technical requirements](https://developers.google.com/search/docs/essentials/technical)
(retrieved 2026-10-09; displayed update 2025-12-18).

The exercise follows the distinction between guidance and feedback sensors in
Birgitta Böckeler's article: instructions describe desired reasoning, while the
HTTP observations and known-bad-rule test expose a specific reasoning failure.
This is an application of that distinction, not evidence that the article endorses
this implementation. [Harness engineering for coding agent users](https://martinfowler.com/articles/harness-engineering.html)
(published 2026-04-02; retrieved 2026-10-09).

## Independent review of capability declarations

Read `lib/capabilities.py`, `tests/test_capabilities.py`,
`registry/capabilities.json` and generated `docs/capabilities.md` without editing
them. The declarations explicitly distinguish source integrity from runtime and
quality, which is appropriate. The following concrete issues were sent to the
integrator and their subsequent fixes were rechecked:

1. **P2 — valid global entries fail source checking.** `check()` passed global-link
   entries directly into `catalog.read_local()`, which requires `license_files`.
   Existing `engineering-judgment`, `interface-design` and `marketing-writing`
   entries omit that field. The real `capabilities check` returned invalid entries
   with a `'license_files'` error. The global test fixture always supplied the
   field, so it missed this mismatch. **Resolved:** the integrator normalizes the
   optional field and added `test_real_global_link_schema_needs_no_license_files`.
2. **Scope mismatch — copy work acquires an unconditional experiment.** The
   marketing contract requires a deliverable with a measurable experiment, while
   `marketing-writing` deliberately handles ordinary copy without forcing an
   experiment. Generated docs call contracts required outcomes. Qualify the
   experiment as relevant to the requested task, and select `experiment-design`
   when an actual experiment is needed; do not make a button edit expand into a
   campaign measurement project. **Resolved:** the contract now qualifies the
   experiment with “when the task requires one.”

After integration, `python3 -m unittest tests.test_capabilities -v` passed all
seven tests, and `python3 ai.py capabilities check` returned `valid` for all
23 contracts with no failures. Its JSON report is retained in
`build/capability-seo/capabilities-check.json`. Missing entries and stale hashes
during initial review were in-flight integration work and were resolved before
these checks. No remaining material implementation bug was established in this
bounded review. The 23 contract rows describe intended coverage; they are not
23 completed behavior trials. This SEO exercise adds evidence for the specified
crawl/indexing boundary only.
