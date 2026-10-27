# Native skill project-file path clarification

On 2026-10-26, all 67 authored native skill bodies and their declared operation
boundaries were reviewed after the [sampled Codex design
turn](m4-codex-design-skill-tool-qualification.md) attempted project-relative
`design_prepare` paths. Both relative source and destination paths are supported
by the inspected design implementation; no path defect was established. Three
bodies received scoped factual clarifications, and the saved catalog pins their
revised bytes. This is a source-level guidance clarification; it does not close
the **skill → model → useful design mutation** gate.

The sampled call was refused by Codex with `MCP tool call requires approval, but
approval policy is never`. It returned no Mirket result or operation ID. Mirket's
path validation was not reached. The attempted paths therefore establish neither
a path refusal nor a successful file selection in that model turn. No mutating
model turn was rerun for this repair, and no alternate
host permission route is claimed to work.

## Audited path contracts

The actual public schemas and their scope/publication owners establish the
following distinctions:

| Boundary | Current contract and guidance consequence |
| --- | --- |
| Explicit project selection | `ProjectScope::open` requires an absolute existing project directory. The tool process working directory does not select the project. |
| Ordinary scoped source read | `ProjectScope::resolve_existing_file` accepts a selected relative source as well as an in-project absolute source. Relative paths are resolved against the selected project root. |
| New artifact destination | `ArtifactPlan::scoped_path` joins a relative destination to the selected root and also accepts an in-project absolute destination. Design, D2, browser capture and video prepare operations share this owner. A new-copy output must name a new file. |
| Source expansion and navigation | Source-range expansion and JS/TS/SQL selected-source navigation require bounded absolute source paths, with their existing revision and selection checks. |
| Video dependencies | `assets[].path` and `profile.captions_path` intentionally remain relative to the selected HTML file's directory. Making every path field absolute would violate this contract. |
| Authority and operation identity | Explicit paths do not grant host approval. The existing release/revision checks, prepare/start operation identities, failure reconciliation and host permissions remain applicable. |

The inspection used these local implementation snapshots. Hashes identify the
reviewed bytes; they do not establish authorization or semantic correctness.

| Owner | SHA-256 |
| --- | --- |
| `src/server.rs` | `074bc6e4132e9521c090ec707579a1b9f87ea7603c5d19907075be2dfc6db2ba` |
| `src/scope.rs` | `1bab7d983ebcca0418fcb03ec41117ab6c8c8a89ce5d627ae8bcef69c6ec63a0` |
| `src/artifacts.rs` | `bbe8c6e8bd65e7168d51d19650bed5c5ffcba36d5d710ed63e1eada8002002a2` |
| `src/engine_design.rs` | `c35f3d5f20ca4f64f13a182863f516a73a3b5461ecea7ddb46b13e4e8d7e0f6c` |
| `src/engine_diagram.rs` | `1ed3408142d9ae622ed1fe355dd166570f827296940067ddfd524b297c3c37e2` |
| `src/source_read.rs` | `986d14f081100a152daf8d0f3740f28a365a0bdc0888c324ea5c4dc67968581d` |
| `src/code_navigation.rs` | `2fb1d791e14a602c89dc99dd3a32ad680983d2a0728d2f94e466f36468cb479d` |
| `src/engine_code_navigation.rs` | `e4c094fcec2b9537513def678b4c231b778695c283c94f61e0d0f1e8603c0256` |
| `src/sql_navigation.rs` | `70a84c556ba4a6a575dc1a67f4c1d56efe263525b11e29eb2ff4bdd2429c2742` |
| `src/engine_sql.rs` | `140f5172aadd3608958817cfb04d4d8415df4c3b383f7153b676c8a19b30e2b7` |
| `src/browser.rs` | `11551e4a6bf25cee7ce4ce4cf751c3d2bf7c12561652f5b060171dac22550b9d` |
| `src/browser_session.rs` | `3e91a4fe44a38e63217a42875e269eacb559f65749eeb7fefe11d024d80c43c0` |
| `src/video.rs` | `2c3ae427b1fed55eb021514d284d823733d83edf6fd4d9b9aba4639e97a6ac64` |
| `src/video_session.rs` | `475cac7fd637f2fa11940f497df53d27ec75efebfbc125b30acc79850dc5db51` |

## Guidance and provenance changes

The global `runtime-operations` skill clarifies explicit absolute project
selection where required, resolving supported project-relative file paths
against that root, and following the actual fields' absolute/relative contracts.
It preserves host permission authority and does not require absolute source or
destination paths where relative paths are supported.

`vector-art` identifies SVG authoring as host file work followed by Mirket
optimization/rendering. The public Mirket design schema exposes finite saved-file
operations, rather than a general SVG authoring API. `openpencil` now selects
the project explicitly instead of assuming a reported project workspace.

Names, trigger descriptions, capability selections, all 335 public development
cases and the 9-global/58-project/8-profile release shape are unchanged. The
cases remain planned. The other 64 native skill bodies are byte-identical to
the original authored corpus.

[`skills/catalog.json`](../skills/catalog.json) records the exact revised body
hashes and byte counts and links their local provenance to this record.
[`skills/PROVENANCE.md`](../skills/PROVENANCE.md) records the independently
authored clarification and the continuing model gate. The four original
authoring-basis pins still match and were preserved. No assembler format or
runtime schema changed, and no production dependency was added.

## Verification and remaining gate

Verification records are retained outside shipped source at
`~/.codex/task-state/harness-rewrite-implementation/skill-path-contract-repair-20261026/`.
They contain the previous/current body pins, audited source snapshots, checks
and the disposable assembly inventory. Source and assembly checks establish
catalog/body consistency and refusal behavior; they do not establish that a
model follows the revised guidance.

The following checks passed on macOS 26.5.2 ARM64:

- `python3 scripts/check-build-inputs.py`, before Rust compilation.
- `python3 -m unittest tests/test_assemble_skills.py`: four passed, including changed-body, stale-basis, extra-file and symlink refusal coverage.
- `cargo test --locked --offline native_skills::tests --lib`: five passed, including the actual authored catalog consistency check.
- `cargo test --locked --offline skill_sync::tests --lib`: fourteen passed.
- The assembler CLI wrote 69 files and 422,059 bytes into a new disposable release slot; every assembled file matched its authored bytes.
- Independent comparison against the original frozen skill component confirmed exactly three changed bodies, 64 unchanged bodies, all 335 unchanged development cases, unchanged capability/profile selections and matching original authoring pins. All audited implementation pins above still matched at verification.

The revised catalog SHA-256 is
`b354ddb4d3579ed5fc1b03b81f6891ae4fcf776ffb96c94bab7d8571ae52e6f7`.
The assembler source and test code required no changes.

Coordinator integration exposed one dependent test literal: the full admitted
release capacity had increased by **2,013 bytes** with the three revised skill
bodies. The first full Rust run failed only
`all_admitted_components_compose_with_exact_inventory_capacity` at
`921,548,962` versus the actual `921,550,975` bytes. Its exact expectation in
`src/distribution.rs` was updated. The focused test then passed, followed by a
fresh full locked offline run (**498 library, four CLI and one integration test
passed; five manual tests ignored**) and strict all-target Clippy. The initial
failure remains in the coordinator's task record; no release budget was widened
beyond the measured authored bytes.

The sampled model gate still requires an authorized native-client route that
permits the finite design operations, followed by saved editable SVG, full and
pressure-size renders, independent inspection and the external-reference failure
probe. This repair preserves the earlier observed host refusal and its missing
production evidence. It does not install into the user's existing Mirket home,
change client approval configuration, commit, push or publish anything.
