# P0 pi host/API admission proposal

Status: **approved for disposable offline P0 testing and executed** on 2026-10-29.
The user explicitly approved this exact batch in the current chat, with hooks
and telemetry disabled. [P0 results](pi-p0.qualification.json) record the scope.
The `.proposed` filenames preserve the reviewed snapshot names; they do not
grant production admission.
This is a narrowly scoped authorization for disposable, offline development/integration
use on macOS 26.5.2 ARM64. It is not production redistribution admission, a live
installation change, permission for provider calls, or a release qualification.

## Exact batch

- Host: `@earendil-works/pi-coding-agent@1.4.0`, source commit
  `4a080252f06caf63a525d7873c5d94d3c23887e0`.
- Host peers: `@earendil-works/pi-agent-core`, `pi-ai`, `pi-tui` at `1.4.0`,
  plus `typebox@1.3.27`. The compiled extension must leave them external.
- Compiler: existing admitted `typescript@5.9.3`, `@types/node@24.19.2`.
- Preserve the upstream root `protobufjs@7.6.6` override explicitly: dependency
  overrides in an installed dependency's own manifest do not control this root.
- [Proposed root](pi-p0-package.json.proposed),
  [exact lock](pi-p0-package-lock.json.proposed), and
  [archive inventory](pi-p0.inventory.json.proposed) freeze **122 package entries**.
  Lock SHA-256: `0d66252383c476049e04e90b2a84d8d21f1ad19909d9e5e132002734ae4e6bec`.
  No floating resolver run is authorized by approving this snapshot.
- Reuse the admitted private Node 24.22.1 ARM64 executable, SHA-256
  `6f32069480ec14d52b6d7b1f3c7124cfbbc46203abf5ddf5c65b127435f3f329`.
  Pi's declared minimum is Node 22.19.0; P0 passed with this private Node.

The npm lock-only operation used explicit existing npm 11.16.0 and the verified
private Node, isolated HOME/cache/config files, `--package-lock-only
--ignore-scripts --no-audit --no-fund`. It created no `node_modules`. Archives
were fetched as data and inspected with Python tar readers, not installed or
executed. All 122 matched their locked registry integrity; total compressed
bytes: 28,776,619. This is not verification of registry signatures or provenance
attestations. Registry metadata and pinned source both identify pi 1.4.0.

## Concrete findings and conditions

**Security: restricted experiment only.** The retained
[audit](pi-p0.advisories.json.proposed) exits 1: `undici@8.10.2` has eight advisory
records (maximum high); the host is also reported affected transitively.
These include response-triggered process crashes and conditional TLS validation
bypass. The advisory fix range begins at 8.12.2. The actual pi CLI bundle includes
Undici implementation code (`dist/bundle/chunks/chunk-MM3LKT73.js`), so changing
only the external lock is not evidence of a repaired CLI. Do not run audit fix,
downgrade pi to npm's suggested 0.75.3, or silently substitute another host.

P0 will exercise only local stdio MCP and a deterministic in-process test
provider if needed to drive tool calls. No external HTTP MCP, provider requests,
web fetching, account login, proxy setup or paid model trials. This restriction
avoids the remote-response paths for the experiment; it does not repair them or
make offline flags an OS security boundary. A networked release candidate needs
a separately reviewed corrected upstream artifact or verified rebuild and a new
exact admission. Keep this blocker visible after P0 passes.

**Hooks.** The lock flags `@google/genai@2.21.0` (preinstall echo/no-op) and
`protobufjs@7.6.6` (postinstall version-scheme warning). The archive inventory also
records every package script, including prepare/build/Husky scripts not normally
run from registry tarballs. All lifecycle hooks remain disabled. Do not build
upstream packages or invoke example installers.

**Network defaults.** Pinned published code has `enableInstallTelemetry: true`
in `dist/core/settings-defaults.js`; the TUI also checks package updates,
refreshes model catalogs, and calls `ensureTool` for fd/rg. The latter can fetch
latest GitHub releases. Every P0 process must receive `PI_OFFLINE=1`,
`PI_SKIP_VERSION_CHECK=1`, `PI_TELEMETRY=0`, and disposable settings with
`enableInstallTelemetry: false`. Do not call managed grep/find where a tool is
missing. Unexpected acquisition/network access is a failed case, not permission
to download. Use an allowlisted environment with no provider/cloud credentials.

**Native/WASM.** Pi TUI contains prebuilt platform addons; only the Darwin ARM64
one may load in this trial. Photon 0.3.4 and QuickJS-WASI 3.6.2 carry WASM;
QuickJS extension `.so` names are inventoried without assuming host ELF loading.
The host archive also contains a Doom example WASM. Examples are not loaded,
executed, adapted or redistributed. No native builds, browser download or new
FFmpeg/Chrome selection is in this batch.

## Rights and distribution limit

Manifest declarations across the lock are MIT (59), Apache-2.0 (39), BSD-3-Clause
(13), ISC (7), BlueOak-1.0.0 (2), Unlicense (1), and 0BSD (1). These counts do not
establish rights for every bundled asset. The inventory records hashes of
LICENSE/COPYING/NOTICE/THIRD-PARTY files actually present. Fourteen package archives
lack a separately named notice file, including all seven pi packages, three AWS
packages, bignumber.js, data-uri-to-buffer, proxy-agent-negotiate and
standardwebhooks. Pi's pinned repository root supplies its MIT license, but this
must not be extended by assumption to its Doom example or third-party bundles.

This approval admits local test use only, retaining original archives and notices.
Before distributing any selected dependency tree, resolve omitted notice texts,
bundled code/assets and native/WASM source obligations individually, preserve
copyright/permission notices and Apache NOTICE/modification obligations where
applicable, and exclude unrelated examples. The eventual Mirket package does
not redistribute a second pi host. No upstream example code is copied by P0.

## Approved verification procedure

1. Materialize this exact lock in disposable state using the selected Node/npm,
   `npm ci --ignore-scripts --no-audit --no-fund`, with isolated HOME/cache/config.
   Compare installed identities against the inventory; never regenerate the lock
   silently. Record any install failure rather than enabling hooks.
2. Build a tiny original compiled ESM extension outside shipped source, with
   external host peers, and two original skill fixtures. No heavy adapter import.
3. Run the cases in the [P0 integration specification](../../tests/pi-p0.md).
   Use the current Rust `dev serve` route only for the temporary experiment.
4. Retain logs, artifact hashes, process exit/PID evidence, requested/effective
   tool exposure, resource origins and measured timing/RSS. Report CLI/TUI/SDK
   results separately. P1 remains gated on the actual P0 exit criteria.

Source references: [pinned package manifest](https://github.com/earendil-works/pi/blob/4a080252f06caf63a525d7873c5d94d3c23887e0/packages/coding-agent/package.json),
[pinned package contract](https://github.com/earendil-works/pi/blob/4a080252f06caf63a525d7873c5d94d3c23887e0/packages/coding-agent/docs/packages.md),
[pinned MCP contract](https://github.com/earendil-works/pi/blob/4a080252f06caf63a525d7873c5d94d3c23887e0/packages/coding-agent/docs/mcp.md),
[pi MIT license](https://github.com/earendil-works/pi/blob/4a080252f06caf63a525d7873c5d94d3c23887e0/LICENSE),
[Undici TLS advisory](https://github.com/nodejs/undici/security/advisories/GHSA-2m5v-pch6-74hx),
[Undici response-crash advisory](https://github.com/nodejs/undici/security/advisories/GHSA-xjj6-f33q-929c).

Raw read-only inspection inputs, source excerpts and resolver logs are retained
outside shipped source in `/Users/nazmiyilmaz/.codex/task-state/pi-p0-20261029/`.

## Additional offline development authorization — 2026-10-29

The user approved using this same exact 122-package closure to compile and test
the production extension offline. Pi remains external to Mirket's shipped runtime;
hooks and telemetry remain disabled. This adds no package versions and does not
authorize networked release, live installation, redistribution of the host or paid
model trials. The original P0 qualification record retains its original scope.
