# Selected-source Python runtime integration

The development `code_navigate` route connects the exact passive
[Python navigation leaf](m3-python-navigation-implementation.md) to the existing
connection owner and same-binary supervisor. This is a macOS ARM64 development
candidate, not an installed release or a claim of complete Python semantics.
Installed releases leave `AdapterPayload.python_navigation` unset until their
exact runtime, adapter, wheels, source and rights materials are admitted.

Select all four private payload files explicitly; a caller cannot choose these
paths through `code_navigate`:

```sh
mirket dev serve --state-dir /absolute/disposable-private-state \
  --release-id 0.0.1:qualification --node /absolute/private/node \
  --adapter /absolute/private/adapters/main.js \
  --python-navigation-python /absolute/private/python3.13 \
  --python-navigation-script /absolute/private/python_navigation.py \
  --python-navigation-jedi-wheel /absolute/private/jedi-0.20.1-py2.py3-none-any.whl \
  --python-navigation-parso-wheel /absolute/private/parso-0.8.7-py2.py3-none-any.whl
```

All configured paths must be canonical regular files. The leaf rechecks the
exact private CPython 3.13.17 executable, compiled-in adapter bytes and exact
Jedi 0.20.1/Parso 0.8.7 wheel hashes before and after the child lifecycle. It
never searches PATH, imports an ambient analyzer or installs dependencies.
An absent candidate fails explicitly before dispatch.

## Public selection and result

The existing request shape now accepts `python` for `.py` sources and
`python_stub` for `.pyi` stubs. The caller supplies 1–128 distinct absolute saved
paths, every observed SHA-256 revision, a selected `source` index, original UTF-8
`position_byte`, `definitions` or `references`, and a 1–100 `limit`. Python
sources and explicitly selected stubs can share a model. Mixing them with
JS/TS or C/C++, or supplying `c_cpp_model`, is refused. Source, ancestor and
project links are refused by the existing project boundary. `.py`/`.pyi`,
module/package and case-insensitive module collisions are refused by the one
Python model validator.

The public route uses `CapturedSources::capture` and creates deterministic
`source-N` IDs in selection order. The original paths stay in Rust. The private
Python process receives only revision-bound UTF-8 bytes, portable relative
projection paths, source/stub roles, a bound model revision and finite limits.
Every selected original is rechecked after confirmed worker cleanup before
locations become public, including non-target files and edits with restored
mtimes. Initial source drift returns the shared `changed` result with current
revisions; later drift returns `stale`, without locations.

The outcome retains the leaf's `complete`, `incomplete` and `refused` states,
`coverage: "selected_parser_names"`, exact ordered source identities, reasons,
examined-name count and original UTF-8 identifier spans. It adds the exact model
revision, backend identity and verified original path for each location. The
same strict response validator runs at the leaf boundary, across the worker
boundary and before original paths are attached. Unknown fields, source IDs,
revisions, mismatching slices, duplicate spans and contradictory completeness
claims fail explicitly.

`complete` applies only to the selected-parser-name pass. It makes no claim
about unselected modules, runtime behavior or all Python semantics. Unknown or
external bindings and conservatively unsupported constructs are incomplete;
syntax failures and ambiguous queried bindings are refused. Name, location,
analysis-time and output cutoffs are incomplete and discard partial locations.
The public limit narrows the leaf's location limit; it does not truncate a
verified reference set and label that set complete.

Bounds are 256 KiB per file, 2 MiB total source text, a 4 MiB escaped leaf
request, 20,000 examined names, 10 seconds of backend analysis and a 60 KiB
serialized public navigation result including original-path metadata. The
public envelope reserves its metadata before dispatch and returns an explicit
`output_budget` incomplete result if attached locations exceed the remaining
budget. Excessively large path metadata can be refused before dispatch.
JS/TS and C/C++ retain their existing 32 KiB public result bound.

## Lifecycle and ownership

The connection keeps its task alive when the request future disappears, sets
cancellation and awaits the ownership task. The same-binary supervisor acquires
the shared heavy-job lease, persists its unresolved marker and creates a fresh
empty `OwnedScratch` reservation. Only that directory is passed to the leaf.
The Python work budget is the existing 60 seconds including admission; the leaf
caps its complete child lifecycle at 30 seconds. The supervisor's parent pipe
is the lifeline. Cancellation or parent EOF reaches the retained leaf owner,
which terminates and reaps the fresh process group before cleanup. C/C++ keeps
its separate existing 600-second work budget.

The leaf removes only its bounded temporary tree after confirmed reaping; the
outer owner removes only its unchanged empty reservation. Unexpected scratch,
ownership replacement or unconfirmed process cleanup retains scratch and the
unresolved admission marker. No recursive outer removal or PID-based recovery
was added. Queue, transport and lifecycle failures remain explicit errors;
MCP request cancellation follows the host's cancelled-request response contract.

The private interpreter, audit policy and immutable source projection retain
the leaf's passive configuration: isolated startup, no site/project startup,
no source execution, no unsafe extension loading and before-read denial of
outside paths. This is a trusted pinned backend boundary, not an OS sandbox
against a hostile same-user process.

## Local verification

`tests/python_navigation_runtime.rs` exercises public projection/identity
binding, exact BOM/Unicode/CRLF spans, forged responses, mixed backends, source
and stub roles, link/scope/module collisions, initial and post-capture drift,
structured outcomes, 128 selected sources and rejection of an unselected
payload. Its explicitly enabled native groups exercise real definitions,
references, stubs, result cutoffs, a request larger than the old 2 MiB worker
transport bound, cancellation/reaping, released admission and retained unexpected
scratch. The exact private payload JSON is mandatory for those ignored tests.

`tests/python-navigation-runtime-mcp.py` launches the actual development server
and invokes `code_navigate` through stdio MCP. It verifies original paths/spans,
both Python roles, definitions/references, passive project state, syntax refusal,
location cutoff, stale sources, mixed-backend refusal, cancellation cleanup and
an absent private candidate. It uses only disposable project/state directories.

The original isolated Rust and Python suites also run after the narrow identity
contract correction: release/job IDs match `ReleaseId`'s exact ASCII
`[A-Za-z0-9._:-]{1,256}` format, while source IDs retain their stricter 64-byte
bound. Adapter bytes remain part of the model revision.

Run the build guard before Cargo:

```sh
python3 scripts/check-build-inputs.py
MIRKET_PYTHON_NAVIGATION_PAYLOAD=/absolute/private/payload.json \
  cargo test --locked --offline --test python_navigation_runtime \
  --test python_navigation -- --include-ignored --test-threads=1
cargo clippy --locked --offline --lib --bin mirket \
  --test python_navigation_runtime --test python_navigation -- -D warnings
python3 tests/python-navigation-runtime-mcp.py \
  --mirket /absolute/qualified/mirket --node /absolute/qualified/node \
  --adapter /absolute/qualified/adapters/main.js \
  --python /absolute/private/python3.13 \
  --script /absolute/private/python_navigation.py \
  --jedi-wheel /absolute/private/jedi-0.20.1-py2.py3-none-any.whl \
  --parso-wheel /absolute/private/parso-0.8.7-py2.py3-none-any.whl
```

Local evidence is retained under
`~/.codex/task-state/harness-rewrite-implementation/python-route-integration-20261025/`.
The platform is macOS 26.5.2 ARM64. These checks establish this development
MCP/private-worker path, not installed-release availability, Codex GUI activation,
Windows execution, oldest-platform compatibility, release rights admission or
whole-project Python completeness. No commit, push or publication was performed.

The finite native qualification runs serially. A default-parallel combined run
over separate disposable installation states produced one correctly structured
`incomplete/work_deadline` result during payload preparation contention. Its log
is retained; the 10-second analysis and other production bounds were not raised.
The source/protocol tests do not establish a loaded-workstation latency guarantee.
