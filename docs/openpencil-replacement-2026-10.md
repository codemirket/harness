# OpenPencil replacement — 20 October 2026

This replaces the Penpot runtime described in the earlier harness map and
security reports. The change is prepared for review; the live installation has
not been changed. Earlier Penpot evidence remains historical evidence.

## Runtime and routing

| Boundary | Replacement |
| --- | --- |
| Package | Existing `design` package; no additional package or provider |
| Editable design engine | Official OpenPencil CLI 0.16.0, source `ffe2fc5798468988a5943c5f5737905c3c7a28b5`; npm integrity pins cover the resolved closure |
| Topology | One core design-tools container; optional Kroki and Excalidraw containers |
| Removed | Penpot frontend, backend, exporter, MCP and browser-session services; PostgreSQL/Valkey services; backend rebuild and Penpot lock inventory |
| Files | `.fig` source and PNG/SVG exports in the reported workspace; `.pen` input supported |
| Worker path | `editable-interface` routes to the assigned project workspace with explicit production and pixel-verification requirements |
| Host path | Same owned tools in the host's shared design workspace |
| Operations | `openpencil_inspect`, `openpencil_document`, `openpencil_edit`, `openpencil_export`; inspected upstream schemas govern arguments |
| Tokens | Style Dictionary produces DTCG; new/imported documents receive variables/styles; edits compile tokens and can explicitly refresh them through `import_design_tokens` |
| Persistence | No new service volumes. Stopped legacy state is archived byte-for-byte; old volumes and workspace files remain untouched |

OpenPencil's current MCP server requires an attached editor. Mirket uses the
official headless file CLI, so workers do not depend on a desktop session or
shared editor objects. Open resulting `.fig` files manually in OpenPencil.
Remote asset/font retrieval, arbitrary evaluation and app-only controls are not
exposed by this integration. The child has an explicit credential-free environment
and Node file permissions limited to installed dependencies/fonts and scratch.

## Defects reproduced during integration

- The prepared build context gave the new schema manifest mode 0600; the non-root
  image could not read it. The Dockerfile now sets its intended read permissions.
- OpenPencil `.fig` serialization remaps node IDs. The adapter reopens edited files
  and returns persisted IDs; it avoids an intermediate token save before an edit.
- Upstream tool errors can be JSON with process exit zero. The adapter rejects
  errors and partial batch failures before publishing an output.
- The upstream headless PDF exporter fails while loading its browser-oriented
  svg2pdf module. Only verified PNG/SVG export is exposed; use the existing Typst
  workflow for paged PDF production. The failed probe is preserved.
- Editable binary verification must follow the actual source revision. Consuming
  edits/exports return source digests; unrelated or changed source cannot clear a
  pending artifact. Exported pixels still require independent inspection.

The dependency audit identified image-size 1.2.1 through OpenPencil's PptxGenJS
closure. Its lock override is 2.0.4. PowerPoint export is not exposed by Mirket.
The existing Rust TLS closure also received compatible patches:
rustls-webpki 0.103.16 and webpki-roots 1.0.10. Certifi is pinned to 2026.10.19 after the first hosted audit reported
[GHSA-6q48-6c7w-9gw2](https://github.com/advisories/GHSA-6q48-6c7w-9gw2).
The wheel hash and parsed CA bundle were verified; the affected root is absent.
This is a source-lock change, with no live agent image deployment.

## Verification status

The OpenPencil implementation was initially verified at compiled source
`edb6137c600e09e62e46b9ac1f1e7b165effcb23cbca1522c40aa99d24fcf224`.
[Local receipts](openpencil-evidence/local-checks.json) record:

- macOS ARM64 full native gate: format, Clippy and 413 passing tests; 7 existing
  opt-in tests ignored, none failed.
- Linux ARM64 isolated design image: all 73 tests passed, including actual MCP
  discovery/calls, file creation/edit/reopen, canonical variables, persisted node
  IDs, partial-failure rejection, stale-write rejection, source-bound PNG/SVG
  export, path/symlink rejection and scratch cleanup.
- The generated 480×240 PNG was inspected; its center pixel is exactly `#123456`.
  Its [receipt](openpencil-evidence/proof-receipt.json) binds it to the edited `.fig`.
- Catalog validation and setup/doctor for both hosts passed in a disposable home.
- [Source dependency audit](openpencil-evidence/dependency-audit.json): 808 queried
  coordinates, no untriaged findings; three existing reviewed `zip` exceptions.

The verified image is
`sha256:27fa82c56b00d54d2d7e44634db42cdb0e9cb814a3859ee9fc0a59d568364d98`.
Its [raw-scan summary](openpencil-evidence/initial-built-image-audit.json) records
351 Debian packages, 442 npm packages and 3 Go inventory entries. All required
inventories are present. It reports 1,116 advisory/package occurrences (441
unique advisory IDs), including 8 critical occurrences; these are retained as
warnings, not claimed fixed. The OpenPencil npm closure has no untriaged findings.

The maintainer explicitly requested avoiding costly advisory-only remediation.
CI therefore uses `--advisory-policy report` for image scans: advisory findings
remain visible and non-blocking; failed scans, wrong identities, missing/unsupported
inventories, missing targets and functional test failures still fail. The default
standalone scanner policy remains `fail`, and source dependency auditing remains
unchanged. Twelve audit regression tests pass, including proof that report mode
retains critical findings and still rejects missing inventories and wrong targets.
The [policy assessment](openpencil-evidence/advisory-policy-assessment.json) evaluates
the preserved built-image and four remote-image reports under this policy.

An optional Go dependency patch was not retained after its candidate test run
exceeded the existing 300-second deadline. The engine status check succeeded;
the timeout's root cause was not established. The final design recipe is the
same one that passed both 73-test runs and was scanned. Failed probes, the stopped
optional export, and later results are preserved rather than described as passes.

The [initial hosted run](openpencil-evidence/initial-hosted-ci.json) passed JavaScript
and all three native platforms. The [final hosted run](openpencil-evidence/hosted-ci.json)
for code commit `15de6d1aa41646133392fa4538234f89d406cf7f` passed **all six jobs**:
JavaScript, source dependencies, image inventory/advisory reporting, and native
Windows, Linux and macOS checks, packaging, installation and MCP validation.
The [reviewed runtime identity](openpencil-evidence/reviewed-runtime.json) records
source `a0f2d924a1512ae2c7a037d88fb257b30760cd0ab2e20b327dff3df85ae6b41a`.
Subsequent evidence-only documentation updates do not change that runtime source.
Raw evidence, test
logs, exported `.fig`/PNG proof and the verified image archive are retained in
`~/.mirket/evidence/harness-openpencil-replacement-2026-10-20/`.


## Limits and activation

No production deployment, desktop reconnect or saved Penpot database conversion
has occurred. Validation used an isolated review checkout; the original workspace
sync receipts and OS-offloading read limitations are retained in the private bundle. Stop the prior generation using its installed CLI before installing
a replacement generation. Export needed legacy designs first. The migration
retains legacy data; deleting it is a separate decision.

Schema/tool tests do not establish every OpenPencil feature, font family or file
format round trip. OS/native/WASM inventory and scanner limitations are reported
separately from npm advisory matching. Native Windows CLI checks do not establish
Windows execution of Mirket's Linux ARM64 design container.
