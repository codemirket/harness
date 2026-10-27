# Bounded LSP navigation transport

`src/lsp_client.rs` implements a backend-neutral, finite stdio LSP client for
**document symbols, definitions and references**. It uses the existing locked
Rust crates, `ProjectScope` and `OwnedChild`; it adds no dependency, tool discovery,
plugin mechanism or persistent daemon. The module is currently compiled **only
under `cfg(test)`**. No production backend, release payload, MCP tool, cache,
analyzer admission slot or Codex GUI journey is wired by this change.

The [rust-analyzer experiment](m3-rust-navigation-qualification.md) establishes a
candidate's bounded semantics independently of this transport. The [ty
experiment](m3-python-ty-qualification.md) is another candidate. The fake-server
checks here establish transport behavior, not either candidate's production
capability or passive loading policy.

## Ownership and API

The API is crate-internal and has no request deserialization that could authorize
a raw command or LSP method:

- `PreparedProject::capture(PreparedInputs)` reads one explicit JSON model file
  and a finite manifest of original UTF-8 documents, each with an expected
  `sha256:<64 lowercase hex>` revision. Project and optional dependency roots
  are explicit. There is no filesystem/project/dependency discovery. Capture
  rejects missing, changed, duplicate, oversized, invalid UTF-8 or escaped inputs.
- `LaunchSpec::trusted(...)` is for the backend integration owner. It binds an
  absolute selected executable and its SHA-256, a canonical private job directory,
  backend identity and fixed initialization options. Invocation is either no
  arguments or the literal `server` argument required by ty. No arbitrary argv
  or environment map is accepted. The executable is streamed and rehashed at
  launch, without PATH lookup.
- `navigate(spec, project, request, cancel, absolute_deadline)` starts a fresh
  owned analyzer for one operation. `Operation` has exactly `DocumentSymbols`,
  `Definition { byte_offset }` and `References { byte_offset,
  include_declaration }`. The request document must be a captured source and
  match its exact expected revision. There is no generic send/command interface.

The backend owner must derive effective analyzer configuration from verified
prepared metadata and bind all additional immutable release inputs. Capturing a
JSON file does not prove that arbitrary initialization options describe it, that
its dependency graph is complete, or that the analyzer reads only the declared
files. `Current` records both the model revision and the exact initialization
revision so those identities cannot be conflated. Before activation, the owner
must validate that correspondence and establish analyzer-specific configuration
precedence, execution and offline policies. In particular, rust-analyzer project
configuration, compiler/sysroot selection, proc macros, build scripts, discovery,
checking and runnable metadata require the qualified policy in its report.

The caller owns admission and its private job/scratch lifecycle. The client creates
new private `home`, `cargo`, `rustup`, `config` and `tmp` directories inside that
job and refuses existing ones. The child starts with an empty environment plus
those explicit paths, an empty PATH and disabled rustup auto-install. This is
configuration isolation, **not network denial or an OS sandbox**. An offline
backend still needs its own qualified fixed policy and supplied prerequisites.

## Original positions and outcomes

The client negotiates UTF-16 only; an omitted server encoding uses the LSP UTF-16
default, while an explicit different encoding returns `Unsupported`. Conversion
uses exact original text without newline normalization or reserialization. It
handles Turkish characters, astral surrogate pairs, LF, CR and CRLF. It rejects
positions inside UTF-8 scalars, inside UTF-16 surrogate pairs, between CR and LF,
outside line contents or in reversed ranges.

Server locations must match a URI generated from a captured canonical source
path **exactly**. No network URI, file authority, relative URI, percent-decoded
alias or implicit outside dependency is opened from a response. Definition
`Location`/`LocationLink` and reference `Location` results become original UTF-8
byte ranges with target revisions. Link selection/origin ranges are validated.
Hierarchical and flat document symbols retain original ranges; hierarchical
children retain parent indices and must fit their parent's source range.

`NavigationOutcome` distinguishes:

| Status | Meaning |
| --- | --- |
| `Current` | Validated positions in the declared corpus. Includes backend identity, executable/configuration/model revisions and a stable manifest revision incorporating source paths, revisions and language IDs. Coverage is explicitly `declared_sources_only`. |
| `Incomplete` | Cancelled, deadline reached, backend not ready/error, or output budget exceeded. Oversized results are not silently truncated into a complete result. |
| `Unsupported` | Operation or negotiated position encoding unavailable; unsupported process platforms are refused before launch. |
| `Stale` | A source/model/request revision changed, became unavailable, or the backend reported content modification. No current locations are returned. |

`ClientError` distinguishes invalid input, unavailable sources, stale executable
identity, scope/position violations, malformed protocol, budget, launch, cleanup
and owner failures. It retains `cleanup_confirmed`; false requires keeping
admission/ownership for reconciliation. Untrusted server diagnostics are not
returned as error text.

Every captured source and the model are rechecked before execution, before the
semantic request, and after shutdown/reaping before a current result escapes.
This includes sources not mentioned by the returned locations. These are bounded
consistency checks, **not an atomic multi-file snapshot**, and inherit
`ProjectScope`'s documented concurrent ancestor-replacement limitations. The
backend integration must identify the full analyzed corpus or separately pin
immutable dependencies; this module cannot infer all semantic inputs from a
successful LSP response. Empty results do not prove missing metadata is complete.

## Protocol, budgets and shutdown

Strict `Content-Length` framing rejects duplicate length headers, unsupported
content encodings, ambiguous/unknown headers, invalid JSON-RPC envelopes,
truncated messages and mismatched response IDs. Only one client request is pending
at a time. An independently draining reader prevents stdout/stdin backpressure
from blocking reception during document writes. Its queue and cumulative byte
budget are bounded. A separate bounded stderr drain discards content.

All server-initiated requests receive JSON-RPC method-unavailable errors. This
includes edits, execution, document opening, registration and configuration
lookup. Notifications are bounded data; they never trigger host effects.
Readiness can use the initialize reply or rust-analyzer's quiescent status. The
latter's warning/error health yields incomplete analysis. Neither policy by
itself proves semantic completeness.

| Bound | Value |
| --- | ---: |
| Explicit sources | 1–128, plus one static model |
| Explicit additional roots | At most 8 |
| Per-source/model bytes | 1 MiB |
| Aggregate source bytes | 8 MiB |
| Selected executable bytes hashed | At most 256 MiB, streamed |
| Header bytes / header fields | 8 KiB / 16 |
| Single framed body | 2 MiB |
| Total wire bytes | 16 MiB separately in each direction |
| Incoming messages / server requests | 512 / 16 |
| Returned items / symbol depth | 1,000 / 32 |
| Serialized successful result | 32 KiB |
| Maximum selected execution deadline | 10 minutes |
| Cancellation notification and acknowledgement grace | At most 100 ms each |
| stderr | Existing process owner's 16 KiB limit |

Escaped document/initialization messages still have to fit the wire limits; a
raw-source allowance does not waive framing/aggregate bounds. The selected
absolute deadline includes initialization and request processing; queue wait is
the integration owner's separate responsibility. Bounded synchronous filesystem
reads/hashing/conversion are not a hard real-time OS scheduling guarantee.

Cancellation sends `$/cancelRequest` for a pending RPC, allows bounded response
grace, and then terminates/reaps the owned group. Success also shuts down the
fresh analyzer and confirms cleanup. The task owning the child survives a dropped
calling future and observes a cancellation guard, so dropping a requester does
not merely abandon the analyzer. Callers must await confirmed cancellation before
releasing admission. Runtime/host death, escaped descendants and unconfirmed
cleanup remain the shared process owner's limits; the client does not invent a
sandbox or kill by process name. Native Windows launch remains refused until the
existing ownership boundary supports it.

## Verification and activation gates

Inline tests use a controlled local fake LSP server generated in disposable
private directories. Process tests require an explicitly selected private Node
via `MIRKET_TEST_NODE`; they never search PATH or install a runtime. The recorded
macOS ARM64 run supplies that variable, so these process cases execute rather
than taking their documented unavailable-runtime branch.

The corpus exercises exact Unicode/CRLF positions, same/cross-file definitions,
references, nested document symbols, fragmented frames, hostile server requests,
malformed/oversized/truncated frames, ID/scope/position errors, stderr/message/
result budgets, unsupported encoding/capabilities, backend errors, source/model
changes, executable drift, expired requests, explicit cancellation, deadlines and
dropped caller futures. Source changes preserve no falsely current locations.
The process tests also exposed a macOS exited-leader group-cleanup race, repaired
by the shared process owner with its own discriminating test.

Actual command results are retained under
`~/.codex/task-state/harness-rewrite-implementation/rust-navigation-qualification-20261025/`
as `lsp-client-tests-*.log`, `lsp-client-cleanup-diagnostic.log` and the final
Clippy/process-owner records. Earlier failures remain evidence, not passing
results. Final recorded checks on macOS **26.5.2 ARM64**, Rust/Cargo **1.99.1**, and
explicit private Node **24.22.1**:

- Build-input guard passed before compilation.
- `cargo test --locked --offline --lib lsp_client:: -- --nocapture`: **14 passed,
  0 failed, 0 ignored**; the explicit Node variable was set and the fake process
  cases executed. Final log: `lsp-client-tests-03.log`.
- `process::tests::already_exited_owned_child_can_be_reaped_after_group_cleanup`:
  **1 passed**, recorded in `lsp-client-process-regression.log`.
- Strict `cargo clippy --locked --offline --all-targets -- -D warnings` was
  attempted. It stopped at the concurrently added `web_inspect.rs` large-enum
  finding, with no LSP diagnostic. This is **not a passing whole-crate Clippy
  result**; the integration owner owns the rerun after the web fix.

The source and docs were checked for formatting/whitespace. These results do not
qualify native Windows, the oldest macOS target or an actual analyzer through
this newly authored transport.

Before production activation, remove the test-only module gate **as part of a
real backend integration**, bind private admitted payloads and prepared metadata,
retain existing two-analyzer/heavy-job admission and finite connection ownership,
exercise installed MCP and source/refusal journeys with actual analyzers, and
qualify platforms and release rights. No latency, whole-project coverage,
installed Rust/Python navigation or Codex model-turn readiness is claimed here.
