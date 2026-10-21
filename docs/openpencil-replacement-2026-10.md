# OpenPencil replacement — 20 October 2026

This replaces the Penpot runtime described in the earlier harness map and
security reports. The verified replacement is deployed. The actual Codex Mirket connection
reports the new binary. The separate design connection subsequently responded
successfully during the 21 October review; activation is connection-specific. Earlier Penpot evidence remains historical evidence.

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
| Persistence | No new service volumes. The authorized cleanup removed legacy Penpot volumes, session state and generated token files |

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
The updated agent image is deployed; its inventory contains Certifi 2026.10.19.

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


## Deployment and live verification — 21 October 2026

The [deployment receipt](openpencil-evidence/deployment.json) identifies the
installed macOS ARM64 release binary from the successful hosted run. Its SHA-256
is `186a2311aa928238db6df3227876794a1dfd6b4bd052876eb9e732b2e58f1d3b`.
The runtime update preserved the selected hosts and packages. Post-restart doctor
checks pass for managed files, configuration, package readiness and fresh MCP calls.
The actual Codex `runtime_status` call reports PID 1747, the reviewed source above,
and an installed executable matching that process. Claude activation was not tested.

The [live file verification](openpencil-evidence/live-verification.json) records
creation, edit, save/reopen with persistent node IDs, canonical token variables,
source-bound PNG/SVG export, stale-write rejection and missing-node rejection
without publishing a partial file. The [pixel inspection](openpencil-evidence/live-pixel-inspection.json)
confirms the 480×240 dark-blue rectangle, center RGB `(18, 52, 86)` and PNG digest.
All 15 owned module hashes match the reviewed checkout. After reboot, fresh
installed-CLI calls reopened the same saved `.fig` and produced a byte-identical
PNG; the [post-restart export check](openpencil-evidence/export-after-restart.json)
records its dimensions, pixel value and source digest.

The restart left the private engine stopped. Starting it and the saved core and
diagrams services restored discovery of 14 design tools, including all four
OpenPencil tools and no Penpot tools. Codex had dropped its separate design server
connection at that point. The subsequent 21 October review observed a successful
native design response with matching module identities. The CLI checks and that
connection observation remain separate evidence; Claude activation was not tested.

The contained acceptance gate passed before deployment and again after restart.
The [retained acceptance receipt](openpencil-evidence/contained-acceptance.json) binds
the deployed binary and reviewed source to all five passing checks: contained
runtime, desktop contracts, parallel runtime, design output and backup recovery.
The repeat replaced a final receipt lost from temporary storage during reboot;
its full output is now saved directly to the durable bundle. No required checks
or limits were reduced.

## Penpot cleanup and retained failures

The [removal receipt](openpencil-evidence/penpot-removal.json) records the authorized
purge of the owned database, cache and assets volumes, old containers and session
state. It also records removal of 33 generated Penpot token files, the empty Penpot
scratch directory and 12 unused Penpot/PostgreSQL/Valkey images. Fresh inventory
found no engine volumes or Penpot containers and none of those removed image IDs.
Canonical design sources and historical audit evidence remain; no saved database
conversion was requested or performed. This is deletion evidence, not a secure
erasure claim.

The initial agent-image build exhausted the private VM disk. Recovery removed the
unused Penpot images and task-owned build scratch and reclaimed discarded VM
blocks. A subsequent overlay I/O error required restarting the private VM before
the image rebuilt successfully. Readiness timeouts also recurred under severe host
swap pressure. The private VM was resized from 8192 to the supported 6144 MiB;
an attempted 4096 MiB setting was rejected without changing it. The unrelated VM
was left untouched. Failed probes and recovery receipts remain in the private
bundle alongside successful checks.

## Limits

Schema/tool tests do not establish every OpenPencil feature, font family or file
format round trip. OS/native/WASM inventory and scanner limitations are reported
separately from npm advisory matching. Native Windows CLI checks do not establish
Windows execution of Mirket's Linux ARM64 design container. Advisory-only image
findings remain recorded warnings under the requested policy.
