# Exact unbundled pi host candidate

Status: approved for the exact external host in the local pi release candidate
on 2026-10-29, following the earlier disposable-testing approval. See
[`pi-local-candidate.json`](pi-local-candidate.json) for immutable input hashes
and the explicit remaining boundaries.
The user approved the request below; offline installed-package/CLI/RPC/PTY
qualification passed as recorded in `pi-host-repair.qualification.json`. All
122 package archives and 15,251 installed regular files matched byte-for-byte.
Authorized Astra Medium provider, image and provider-native research paths also
passed using the unbundled host with module tracing. Final package host binding passed, including altered lazy Undici code and
unexpected-file refusals. Release qualification remains in progress.
This is a separate batch from P0. The later approval covers the local candidate
only; it does not authorize live installation, host redistribution, browser
production dependencies or publication.

Pi 1.4.0 ships an unbundled `dist/cli.js` in the same npm archive as its normal
bundled CLI. The normal `bin.pi` points at `dist/bundle/cli.js`, which embeds
vulnerable Undici bytes. This candidate selects the existing unbundled CLI and
changes only external `undici` from 8.10.2 to **8.12.2**. It does not rebuild or
modify pi source, bundle a second host into Mirket, or change pi versions.

The exact candidate lock is `pi-host-repair-package-lock.json.proposed` (122
package instances). `pi-host-repair.inventory.json.proposed` records the original
lock, changed entry and exact new archive/hash. Undici 8.12.2 is MIT, declares
Node >=22.19.0 and has no production dependencies. Its `prepare` script invokes
Husky/platform setup: all hooks remain disabled, including in this candidate.
The registry advisory response for this exact package/version set is retained in
`pi-host-repair.advisories.json.proposed`; a clean response is not proof of safety.

Before any fixed-host or networked-release claim, qualify the exact unbundled
CLI/TUI/RPC path, verify which Undici code is actually imported (including lazy
provider paths), prove the bundled entry points are not used, and rerun lifecycle
and performance gates. A package override by itself does not repair the normal
bundled CLI. If unbundled execution reaches the old bundle, this candidate fails.

Requested authorization: use this exact candidate closure in disposable local
host qualification, with hooks/telemetry disabled. No live installation changes,
host redistribution, paid model calls, external account effects or publication.
The host remains an explicitly selected external pi installation. Existing P0
proof and failures retain their original identities.

`pi-host-files.json.proposed` is derived from the frozen archive integrities by
`scripts/freeze-pi-host.py`, not from trusting an arbitrary installed tree. The
new extension checks every unique file, rejects unexpected code and requires the
qualified private Node. It refuses the bundled CLI entry before MCP registration.
The full closure/altered-Undici/unexpected-file tests passed in
`/private/tmp/mirket-pi-host-binding-dWfuVz/report.json`; that initial synchronous
check exceeded the startup budget. The final bounded parallel Rust verifier checks every file. Fresh core
first-tool samples were 1,263.6 / 1,148.8 / 1,140.5 ms, within the user-approved
1,500 ms revised gate. At 30/60/120 seconds aggregate idle was 418.2 MiB,
with no growth; shutdown left no observed owned process. Final negative proof:
`/private/tmp/mirket-pi-host-binding-UVdymp/report.json`. The inventory alone grants no permission; the later explicit local-candidate
admission is recorded separately without rewriting its frozen bytes.
