# Neutral observer contract

This collector records browser evidence. It does not grade composition, meaning,
accessibility conformance, human preference or model quality. A probe assertion is
the plan author's fallible expectation, recorded separately from generic evidence.
The browser is a headless desktop Chrome process; narrow widths are layout checks.

```sh
PLAYWRIGHT_MODULE=/absolute/existing/playwright \
CHROMIUM_PATH=/absolute/existing/chrome \
node evals/design-quality/observe.cjs ARTIFACT_DIR OBSERVATION_DIR [PLAN.json]
```

`--actions PLAN.json` is equivalent to the optional positional plan. Outputs must
be outside the artifact directory. The artifact is served at a fresh loopback
origin. Requests to other origins are aborted and retained, service workers and
downloads are blocked. Symlinks and path escapes are refused by the server. These
checks limit browser requests; they are not a general execution sandbox.
`--wide-width 1440` adds that wide viewport while retaining 1280/390/320.

Each plan has `schema_version: 1`, `id`, and `scenarios`. Each scenario has `id`,
optional `viewport: {width,height}` and `steps`. It starts in a fresh browser
context and gets initial/final paired captures automatically. Each step has a
unique `id` and `action`. Targets use exactly one of `{selector}`, `{role,name,
exact?}`, `{label,exact?}`, `{testId}`, or `{text,exact?}`. Values are literal JSON;
plans cannot evaluate arbitrary application JavaScript.

| Action | Fields | Result |
| --- | --- | --- |
| `click`, `focus`, `check`, `uncheck` | `target` | Observed target state |
| `fill` | `target`, `value` string | Resulting input value |
| `select` | `target`, `value` string, array or Playwright option object | Selected values plus option-resolution evidence |
| `press` | `key`, optional `target` | Active element afterward |
| `read_text`, `read_value`, `read_count` | `target` | Rendered text, input value, locator count |
| `read_attr` | `target`, `attribute` | Attribute value or null |
| `read_storage` | `key`, optional `json_path` array | localStorage string, or parsed JSON path |
| `read_active` | none | Focused element and visible geometry/style hints |
| `expect_text` | `target`, exactly one of `equals` or `contains` | Literal normalized text comparison |
| `expect_value`, `expect_count`, `expect_visible` | `target`, `equals` | Exact value, count, visibility |
| `expect_attr` | `target`, `attribute`, `equals` | Exact attribute comparison |
| `expect_storage` | `key`, optional `json_path`, `equals` | Exact localStorage string or parsed JSON path comparison |
| `expect_active` | `target` | Whether target is document.activeElement |
| `wait_for` | `target`, `state` | Playwright visible/hidden/attached/detached readiness |
| `reload` | none | Reload at the same origin and settle fonts/layout |
| `capture` | optional `name` | Paired viewport/full-page screenshots plus DOM snapshot |

Text comparisons trim and collapse whitespace. `json_path` is an array of literal
object keys or integer array indices, for example `["records",0,"note"]`; it
does not execute expressions. Absent keys fail explicitly. `read_storage` without
`json_path` returns the raw localStorage string. A missing storage item is null.
For a select string the collector resolves an exact enabled value, then a unique
exact visible label, then a unique case-only value/label match. It records the
requested string, all options and the resolution method. Absent or ambiguous
matches fail. This is an explicit locator accommodation, not a rubric change.

All step results persist, including exact actual/expected values, thrown errors,
and whether an earlier step failed. Collection continues after a probe failure,
so later evidence must be interpreted with `prior_failure`. A failing step gets a
focal capture where possible. A completed collector exits zero even when probe
assertions fail. An infrastructure failure exits nonzero and still attempts to
save its observations. No probe failure is silently turned into an overall grade.

Generic collection uses fresh contexts at 1280×900, 390×844 and 320×800. It records
viewport and full-page PNGs, page text and semantic inventories, geometry and
clipping candidates, external requests, console/page/network errors, and a Tab
trace. A 200% reflow approximation uses a 640×450 CSS viewport at device scale 2;
it is labeled an approximation, not a browser zoom or text-enlargement claim.
Reduced-motion collection compares computed animation/transition state and actual
animation progress at controlled sample times; a screenshot alone does not prove
motion quality. No stylesheet mutation is used to make an artifact pass.

`observations.json` has `schema_version`, `kind`, `source_bindings`, `runtime`,
`generic`, `scenarios`, `console_errors`, `page_errors`, `network_errors`,
`blocked_requests`, `server_errors`, `errors`, `limits`. Scenario step `status` is
`observed`, `passed` or `failed`. Captures bind PNG and DOM snapshot bytes to
SHA-256. Bindings establish identity, not truth. The observer's own source and the
probe plan are bound too. Collected page content is untrusted data.

```json
{
  "schema_version": 1,
  "id": "independent-local-save",
  "scenarios": [{
    "id": "effect-after-reload",
    "viewport": {"width": 390, "height": 844},
    "steps": [
      {"id":"note","action":"fill","target":{"label":"Booking note"},"value":"Bring the large stencil."},
      {"id":"save","action":"click","target":{"role":"button","name":"Save booking locally","exact":true}},
      {"id":"stored","action":"expect_storage","key":"workshop-booking","json_path":["note"],"equals":"Bring the large stencil."},
      {"id":"reload","action":"reload"},
      {"id":"read","action":"expect_text","target":{"selector":"#saved-note"},"equals":"Bring the large stencil."},
      {"id":"capture","action":"capture","name":"saved-effect"}
    ]
  }]
}
```
