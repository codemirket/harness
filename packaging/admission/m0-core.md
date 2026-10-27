# M0a dependency batch — approved

Prepared 2026-10-24. **Approved by the user on 2026-10-24; adopted for implementation and local qualification.** This batch supports the first Rust stdio → owned private child → saved artifact slice. No Cargo source/lock in the application, tool installation, upstream compilation, build script, hook, provider call or client configuration was changed by this review.

## Exact request

Approve the direct dependencies and exact resolved Rust closure in `m0-core.Cargo.toml.proposed`, `m0-core.Cargo.lock.proposed` and `m0-core.inventory.json`, including the bounded build-time behavior below; approve the three development-only npm packages in `m0-core.package-lock.json.proposed`. Use the already installed Rust 1.99.1 toolchain for qualification. Approval permits compiling these dependencies and running their application tests in the fresh rewrite/disposable state. It does not approve later tool engines, shipped Node archives, publication or changes to the old installation.

| Direct dependency | Exact version / selected features | Benefit and tradeoff |
| --- | --- | --- |
| rmcp | 3.7.0; defaults off; `server`, `macros`, `transport-io` | Official MCP implementation and typed schema/handler generation. Avoids a custom stack. Brings futures, chrono/time-zone support, UUID, tracing and procedural macros transitively. |
| tokio | 1.54.1; defaults off; `rt`, `macros`, `process`, `time`, `sync`, `io-util`, `io-std` | Async pipe I/O, child operations, deadlines and bounded tasks. Start with current-thread runtime; blocking filesystem work must not stall that executor. SDK `transport-io` also enables `net` transitively; this is compiled support, not a listener or network permission. |
| serde | 1.0.229; default std + derive | Typed request, adapter and receipt serialization. SDK additionally enables `rc`. |
| serde_json | 1.0.154; default std | Bounded JSON messages/receipts. Application must enforce byte/depth/count budgets and reject unknown/invalid semantics. |
| schemars | 1.2.3; default std + derive | Tool schemas from the same Rust types. SDK also enables `chrono04`. |
| clap | 4.6.7; defaults off; `std`, `help`, `usage`, `error-context` | Actual CLI parsing/help with builder API. No derive, colors, suggestions or env parsing. |
| sha2 | 0.11.1; defaults off | SHA-256 source/artifact/release identity without home-grown cryptography. Hashes identify bytes, not authorization or authenticity. |
| libc | 0.2.191; Unix only | Process groups/signals and narrow native ownership checks. Unsafe calls stay in the platform owner and require PID-reuse/crash tests. |
| windows-sys | 0.61.2; Windows only; `Win32_Foundation`, `Win32_Security`, `Win32_System_JobObjects`, `Win32_System_Threading`, `Win32_Storage_FileSystem` | Job Objects, launch/handle/identity and file finalization primitives. Native Windows execution remains untested. |

No SQLite, separate lock crate, HTTP server/client, OAuth, rmcp client, rmcp child-process transport, shell discovery, TLS stack, tracing subscriber, native C/C++ library or new test framework is added. Rust `std::fs::File` locking can own locks. Bounded JSON operation receipts under native locks serve M0; SQLite canonical memory remains a later exact admission. Process-group membership alone does not prove all descendants stopped, and killing a parent is not successful cleanup.

## Closure and toolchain evidence

Cargo 1.99.1 resolved 96 external crates. Target-filtered graph reachability is 74 for `aarch64-apple-darwin`, 71 for `x86_64-pc-windows-msvc` and 71 for `aarch64-pc-windows-msvc`, including build/proc-macro dependencies. All 96 cached `.crate` archive SHA-256 values were independently compared with the proposed lock checksums. Total archive bytes: 10,340,720; this is neither installed nor executable size. Inventory records every version, checksum, license expression, selected features, declared Rust floor, dependency edges, target membership, build script/proc-macro presence and discovered license/notice file hashes.

Highest declared Rust floor in the closure is 1.89.0 (`uuid`); rmcp/sha2 declare 1.88, Tokio/Clap 1.85. Installed rustc is `1.99.1 (58cf8f9c3 2026-10-13)`, Cargo `1.99.1 (5f94df478 2026-08-27)`. Resolve/metadata success establishes metadata compatibility only. No crate was compiled and no platform ABI/client behavior was tested. Pin toolchain 1.99.1 with minimal profile after approval; do not install absent Windows target toolchains implicitly. The disposable root name `mirket-m0-admission` must be replaced by the actual application root when adopting; preserve all external lock versions/checksums and review any delta.

Selected license alternatives are permissive MIT/Apache-2.0/Unlicense, with Unicode-3.0 and LLVM exception notices where recorded. Preserve actual upstream notices during assembly. `r-efi` has an MIT/Apache/LGPL alternative in metadata but no filename-matched license text in its crate archive; it is outside all three selected target graphs. This is a concrete future source-redistribution gap if its archive is redistributed; it does not establish an LGPL requirement for selected native builds. License expressions and source-file hashes are evidence, not a blanket legal conclusion about the final binary/package.

## Build-time allowlist and meaningful caveat

The selected target union contains these 12 build scripts (versions/hashes in the inventory): `getrandom`, `libc`, `num-traits`, `proc-macro2`, `quote`, `ref-cast`, `rmcp`, `serde`, `serde_core`, `serde_json`, `thiserror`, `zmij`. Typical behavior is Cargo cfg emission, Rust version/feature probes, and generated private modules under OUT_DIR. Rust compiler subprocesses/probes are expected. The full lock also records build scripts for unselected wasm/Haiku packages; they are not in this target build allowlist. Procedural macros are upstream code execution at build time too; the inventory explicitly identifies them. No no-code-execution claim applies after approval.

**rmcp 3.7.0 build.rs is exceptional:** it checks the parent-of-parent of CARGO_MANIFEST_DIR; if BOTH `.git` and `.githooks` exist there, it executes `git config core.hooksPath .githooks` in that ancestor. The crate came from crates.io and that condition is false in the inspected Cargo registry layout. Before every approved build, assert the condition is false for the resolved rmcp source directory, including any vendored location, and refuse the build otherwise. Do not check out/build the upstream workspace, run setup-hooks, or silently patch/bypass the condition. This avoids an incidental Git configuration mutation while using the exact upstream crate. A future source-layout change must re-run this check. The build-script source was read, not executed.

## Advisory assessment

RustSec advisory DB snapshot `21d909e4c8bd4e14c7aea58bf980c881a0053ab1` was downloaded as data. Across all 96 locked package names, 26 advisory records matched names; each proposed stable version satisfies a published patched or unaffected range. The comparison and source hashes are in `m0-core.advisories.json`. This was explicit version-range comparison, not a cargo-audit run, full source audit or proof of absence of vulnerabilities.

Two current examples matter: [Tokio RUSTSEC-2026-0384](https://rustsec.org/advisories/RUSTSEC-2026-0384.html) affects the 1.54.0 multi-thread Interval race and is patched by selected 1.54.1; do not relax to 1.54.0. [rmcp RUSTSEC-2026-0189](https://rustsec.org/advisories/RUSTSEC-2026-0189.html) affects older Streamable HTTP DNS-rebinding handling; selected 3.7.0 is patched and this batch does not enable that transport. Version safety and path reachability are separate observations. Re-evaluate the snapshot before release and after any closure change. Untrusted JSON/resource paths, stderr/stdout limits, macros, unsafe OS calls and process identity still require application tests; a clean advisory comparison does not validate these.

## Adapter development batch

Use exact development dependencies `typescript = 5.9.3`, `@types/node = 24.19.2`; lock closure adds only `undici-types = 7.24.6`. TypeScript is the parent's chosen stable JS compiler for this slice, not the current latest compiler or the future native TypeScript analyzer. `@types/node` follows Node 24. All three tarballs were downloaded as data and checked against lock SHA-512 SRI; license/notices, SHA-256 and manifest scripts are recorded in `m0-core.node-dev-inventory.json`. TypeScript has build/test/setup-hooks scripts but no install lifecycle hook; none was run. Production adapter runtime npm dependency count remains zero.

`npm install --package-lock-only --ignore-scripts --no-audit --no-fund` created only a disposable lock, not node_modules. After approval use the exact lock with `npm ci --ignore-scripts`; invoke only the packaged compiler entry point for compilation. Target ES2022/NodeNext for ordinary Node24 stdlib adapters; validate explicit executable selection and output in runtime tests.

Existing development runtime: Node 24.18.1, npm 11.16.0. It may be used explicitly for local tests; it is not the shipped private payload and must never become a runtime PATH fallback. Current official Node 24 LTS artifact lead is 24.22.1. Official SHASUMS lists: macOS ARM64 tar.gz `89c682593d76cd44233fa53b1563264e7de9cbb189cad169135c5b4fadfbc0a7`; Windows ARM64 zip `a0c807319fede24c4a0d8f84564c82a619bd313b16b10c007c383c2609e99171`; Windows x64 zip `d2046d28c2c3dcd211ac5145c0bf30e2047ef9dfb306693c8c6e2eccd8b95fa5`. These binaries were not downloaded, authenticated, unpacked or license/advisory-qualified. Their private redistribution is deliberately outside this batch. Source: [Node 24.22.1 SHASUMS](https://nodejs.org/dist/v24.22.1/SHASUMS256.txt).

## SDK/API handoff

The official tag `rmcp-v3.7.0` resolves to `5d6a11f2ae022f513aec0f6e0f10f89a57891a7f`, matching the published crate's `.cargo_vcs_info.json`. Use [tagged SDK source](https://github.com/modelcontextprotocol/rust-sdk/tree/rmcp-v3.7.0), [exact feature list](https://docs.rs/crate/rmcp/3.7.0/features) and [ServerHandler source](https://github.com/modelcontextprotocol/rust-sdk/blob/rmcp-v3.7.0/crates/rmcp/src/handler/server.rs). The initially guessed examples/servers/src/counter.rs URL returned 404; it was not evidence. Published crate tests and source supplied the API evidence instead.

3.7 uses `ServerConfig`, `CallToolResponse`, `CallToolResult`, `ContentBlock`, `ServiceExt`, `#[tool_router]`, `#[tool_handler]` and `Parameters<T>`. Schema input structs derive `serde::Deserialize` and `schemars::JsonSchema`; reject unknown fields where the operation contract requires it. API sketch below is source-checked but uncompiled and intentionally does not pretend to implement useful M0 tools:

```rust
use std::borrow::Cow;
use rmcp::{ServerHandler, ServiceExt};
use rmcp::model::{ProtocolVersion, ServerConfig};

#[derive(Clone)]
struct ProbeServer;
impl ServerHandler for ProbeServer {
    fn get_info(&self) -> ServerConfig { ServerConfig::default() }
    fn supported_protocol_versions(&self) -> Cow<'static, [ProtocolVersion]> {
        Cow::Borrowed(ProtocolVersion::known_up_to(&ProtocolVersion::V_2025_11_25))
    }
}
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = ProbeServer.serve(rmcp::transport::stdio()).await?;
    server.waiting().await?;
    Ok(())
}
```

The temporary protocol ceiling avoids advertising the newer no-initialize lifecycle without qualification; actual host experiments determine supported versions. SDK defaults otherwise advertise all known versions through 2026-07-28. Test both client negotiations and do not confuse a protocol enum with actual host support. Add finite task-specific tools through the macros; the source test `tests/test_mrtr_behavior.rs` shows the current router/handler pattern, but this batch does not enable its elicitation/request-state facilities. SHA-256 API: `use sha2::{Digest, Sha256}; let digest = Sha256::digest(bytes);`. CLI API: `clap::Command::new("mirket").subcommand(clap::Command::new("serve"))`. OS lifecycle still needs the application-owned race-safe native implementation and tests.

## Required post-approval validation

1. Verify unchanged manifest/lock/archive hashes and the rmcp build guard; build using the pinned Rust toolchain with locked resolution. Verify no unexpectedly activated features, build scripts or native libraries.
2. Compile the adapter with the exact npm lock and test stdio → child → saved result using explicit runtime path, bounded JSON and stderr, no PATH fallback.
3. Exercise the M0a acceptance/failure contracts: stale release before effects, same operation/changed intent, receipt eviction/expiry, response loss, cancellation and abrupt death, two owners, unrelated-process preservation and synthetic release activation.
4. Qualify the actual Codex surface and native Windows targets separately; local compile/metadata cannot pass those gates. Measure package/staging/runtime bytes later.

All raw evidence and the reproducible inventory generator are under `/Users/nazmiyilmaz/.codex/task-state/harness-rewrite-implementation/dependencies/`. Review artifacts under packaging contain only the concrete candidate, not an admission decision.
