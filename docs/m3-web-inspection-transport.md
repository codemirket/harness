# Read-only HTML/CSS MCP transport

`web_inspect` connects the original-source inspection leaf to the existing
connection-owned finite worker lifecycle. Requests provide the expected release,
an explicit project root, and one request containing `path`, `language`
(`html` or `css`), ordinal `pointer`, optional `expected_revision`, `start_child`
and `limit`. Every nonempty pointer or nonzero child offset requires the full
observed SHA-256 revision. This exposes structural inspection only; the
[leaf contract](m3-web-source-inspection.md) describes its syntax coverage and
explicit incomplete locations.

The parent reads the saved source through `ProjectScope`, bounded to 1 MiB and
valid UTF-8. It hashes the exact original bytes, reserves the actual serialized
release/project/path envelope from the 32 KiB structured-result budget, and sends
only original UTF-8 text, source revision, release/job identities and selection
to the worker. Project/source paths never reach the Node leaf. A stale expected
revision returns `changed` without dispatching a child or returning locations.

The private Node executable comes from the already configured Design payload.
`web-inspect-main.js` is a fixed sibling of its design adapter and imports the
fixed `web-source-inspect.js` sibling. The existing HTML/css-tree package closure
is reused without additional dependencies. Installed operation has no PATH,
host-Node or project-package fallback. Launch uses a cleared environment and
the canonical private state directory as its working directory. The Node leaf
does not read source/configuration files, resolve URLs, invoke project plugins,
execute source or create output files.

The new Web work variant uses the existing same-binary supervisor, shared heavy
admission, bounded queue, unresolved ownership markers and parent lifeline. The
connection registry retains its task through request cancellation and shutdown.
Dropping a handler signals cancellation; the retained owner cancels and reaps
the Node child before clearing admission. The existing direct-child/process-group
ownership contract remains in effect. Lifecycle locks/markers use the private
state owner; no source bytes or result artifacts are persisted there.

The Node entry accepts exactly one UTF-8 JSON line followed by EOF. Its read
deadline is five seconds, the leaf execution deadline at most 30 seconds, and
the existing finite worker/admission deadline at most 60 seconds. The existing
2 MiB request cap is unchanged. A source below 1 MiB can still refuse if JSON
escaping exceeds that wire quota. There is no truncation or source staging
fallback. Leaf stdout is bounded to 64 KiB including its identity envelope, the
inspection body to the caller's reserved budget, and stderr to 16 KiB counted
without retaining content.

Rust checks protocol/release/job identity, exact backend versions, original source
revision/byte count, finite result fields, node/issue metadata, UTF-8 range
boundaries, immediate-child ordinal identity/source ordering and forward-progress
continuations. It repeats result validation after the supervisor reply. After
confirmed child cleanup, the parent re-reads the scoped saved source. Detected
drift or an unavailable re-read discards the entire page and returns `changed`
without children or locations. This is an observed revision recheck, not an
atomic filesystem snapshot or containment against hostile ancestor replacement.

The response is `{ release_id, inspection: { project_root, path, language,
pointer, outcome } }`. Ordinary outcomes use the leaf result contract. Parent
revision drift uses `{ status: "changed", current_revision }`. Invalid requests,
scope failures, unavailable private runtimes and unconfirmed transport/cleanup
produce tool errors. Source and output files are never created by this operation.

On macOS 26.5.2 ARM64 with explicit Node 24.18.1 and Rust 1.99.1, the build-input
guard, locked TypeScript compilation, six Node protocol tests, six focused Rust
tests and strict library Clippy passed. The Rust run explicitly supplied the
actual development Mirket binary, Node executable and compiled web adapter;
it exercised the real same-binary worker and both admission slots after cleanup.
Hostile incomplete/oversized leaf stdout and request cancellation confirmed
owned-child reaping. Saved-source drift, escaped-wire quota refusal, scope and
symlink rejection, spoofed response identity, fake pagination completion and
surrogate-interior locations are covered. The complete adapter suite passed
65 tests with its one existing live-browser test skipped.

On macOS ARM64, `tests/web-inspect-installed-smoke.mjs` passed against the
disposable four-component release
`sha256:510a84eed465a4cbe35c6287cfbcefba7911ffdef27e338cecf42957cb637346`
with private Node 24.22.1. It exercised actual installed MCP discovery,
private Node/worker dispatch, original HTML/CSS byte spans expanded through
`file_read_ranges`, comments/Raw coverage, malformed input, revision-bound
paging, saved-source changes and scope failures. It checked the project file
list and source bytes unchanged. `doctor --json` reported the same release
healthy. Windows ownership and actual native Codex Desktop activation remain
unqualified.
