# Local browser qualification candidate

Status: exact three-package batch approved by the user on 2026-10-29 for
**disposable local-only browser testing** with hooks and downloads disabled.
This does not approve production use, redistribution, public browsing or account
access. Research uses the separately tested provider-native route; this browser
candidate is being qualified for local live-UI interaction.

The exact three-package candidate is `@playwright/mcp` 0.0.83 and its declared
`playwright` / `playwright-core` 1.64.0-alpha-1790635538000 dependencies. Archive
SHA-512 integrity and SHA-256 identities, scripts, notices and the full closure
are in `pi-browser.inventory.json.proposed` and the proposed npm lock. Archives
were downloaded and inspected as data only. No upstream code or hooks ran.
The packages are Apache-2.0; retain LICENSE, NOTICE, ThirdPartyNotices and all
bundle license sidecars. Bundled third-party identities must not be omitted
from security review: the three top-level packages alone return no advisories.

The bundle notices identify 163 unique npm name/version pairs. The registry
response, filtered using admitted semver 7.8.5 against the actual versions,
finds 26 advisory matches across Hono, MCP SDK, brace-expansion, fast-uri,
ip-address and Zod (maximum high). These are not 26 proven reachable exploits.
The current stable Playwright 1.64.1 archives still match 19 advisories, so a
blind override to stable does not solve this gate. Keep the original declared
closure for a bounded compatibility experiment; do not claim release safety.

Candidate execution uses explicit private Node 24.22.1, an explicit local
`@playwright/mcp/cli.js`, stdio only (no HTTP/SSE server, OAuth or bearer auth),
`--isolated`, `--no-webmcp`, explicit bounded output directory and timeouts.
Filter out browser installation, arbitrary code execution, upload and network
mutation capabilities for research. Local UI QA needs a separately scoped
allowed tool set. Default file restrictions stay enabled; no unrestricted file
access, personal profile, extension connection, CDP attachment or secret file.
Explicit navigation and interaction still need outcome tests: these settings
are not a sandbox and do not establish that the affected bundle code is absent.

Use a separately owned disposable browser instance of the already admitted
Chrome for Testing 156.0.8078.73 macOS ARM64 bytes from
`m3-browser-cft-macos-arm64.json`, verifying its manifest. Never attach to the
HyperFrames render process or profile. This does not add a browser download or
redistribution permission. No install-browser tool or PATH/npx fallback.

Approved scope: install the exact three-package closure offline without hooks
or browser downloads, then test local fixtures, tool filtering, screenshots,
interaction and shutdown in disposable state. No public search or source browsing
is included. Production integration, redistribution and release admission remain
separate until affected-code reachability and outcome evidence are reviewed.
No live pi/Mirket/client configuration changes or publication.

The approved archives were extracted with verified SHA-256/SHA-512 identities
into `/private/tmp/mirket-pi-browser-csmce8ra`; installation evidence and per-file
hashes are retained there. No upstream lifecycle hook ran.

Evidence inspection directory:
`/Users/nazmiyilmaz/.codex/task-state/pi-browser-review-20261029/`.

Local qualification passed through pi's built-in MCP client on macOS 26.5.2
ARM64/private Node 24.22.1. Verified all three package file inventories plus
344 Chrome regular files and five links. Actual keyboard activation, wide/narrow
layout, reduced-motion preference, screenshot saving, forbidden tool calls,
nonlocal navigation and redirect refusal passed. The local proxy never opens
outbound sockets and denied Chrome's attempted background requests. This is not
an OS sandbox or evidence that known vulnerable code is unreachable in general.
All observed owned descendants stopped; temporary Chrome profile removal was
checked. Both saved screenshots were inspected: unclipped text/glyphs and visible
keyboard focus. Evidence: `/private/tmp/mirket-pi-browser-csmce8ra/run-ElxTZg/report.json`;
reproducer: `tests/pi-browser-local.mjs`. Earlier fixture-contract failures remain
in sibling run directories. This does not qualify production browser support.
