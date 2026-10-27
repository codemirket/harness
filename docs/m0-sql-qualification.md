# Finite offline SQL navigation slice

`adapters/python/sql_navigation.py` handles one explicit source set through SQLGlot 30.24.0. It reads one EOF-terminated JSON line and writes at most one reply, then exits. There is no database, project-file discovery, child process, persistent index, template evaluation or automatic download. Rust launches the exact private Python with `-I -S -B` through the existing owned-worker lifecycle, shared admission limits and parent lifeline. The process receives an empty owned working directory and controlled environment; these controls are not a security sandbox.

The only accepted library is `sqlglot-30.24.0-py3-none-any.whl`, SHA-256 `9488828cc018453b7a3ba01e65d2749c9a731314c14583695163ca1f358203fe`, MIT, pure Python, no selected extras or mandatory third-party runtime dependencies. Pass its absolute path through `--wheel` and the exact pin through `--wheel-sha256`. Standard zipimport loads the verified private release archive; before/after hash, identity, module-origin and version checks detect observed drift. They do not establish hostile same-user immutability. The Python distribution is qualified/adopted separately; there is no host-Python fallback in the leaf.

Request keys are exactly `protocol_version` (integer 1), `release_id`, `job_id`, `dialect` (`postgres`, `sqlite`, `mysql`, `tsql` or `duckdb`), `queries` and `schemas`. Each list has 1–8 sources, each exactly `{id,text,revision}`. IDs are globally unique opaque ASCII identifiers 1–64 characters beginning with an alphanumeric character and otherwise containing only alphanumerics, underscore or hyphen. They are not paths. Revisions use `sha256:` followed by 64 lowercase hex digits and must match the exact UTF-8 text. Source order is query sources followed by schema sources. No SQLGlot diagnostic or source text is echoed in a refusal.

Bounds are 2 MiB for the complete input frame including newline, 256 KiB decoded UTF-8 per source, 1 MiB aggregate, 32768 upstream tokens, 8192 AST nodes counted across original and qualified trees, depth 64, and 128 scopes per query. The wire bound is independent of the decoded bound: JSON escaping can exceed it. Malformed framing/JSON, duplicate or unknown fields, wrong types or invalid identities exit nonzero without an unbound reply. Well-shaped source revision/analysis failures return bound static refusal codes. The external supervisor remains necessary for parser/optimizer execution that precedes a post-transformation bound check.

The reply is `{protocol_version,release_id,job_id,result}`. Result fields are exactly:

- `status`: complete, incomplete, refused or budget_exceeded.
- `sources`: `{id,revision,role}` in input order, with role query or schema.
- `queries`: `{source_id,status,code}`; status is complete/incomplete/refused, code is null except static refusal codes.
- `links`: `{reference,definition,kind}`. Each span is `{source_id,start_byte,end_byte}`, UTF-8 bytes with an exclusive end. Kinds are table, column, alias, cte and derived_output.
- `unresolved`: `{source_id,code,start_byte,end_byte}` with a paired span or paired nulls.

Overall complete requires every query complete; overall refused means every query refused; mixed/partial results are incomplete. The complete serialized reply including envelope and newline is at most 32 KiB. Overflow returns budget_exceeded with the source list, empty queries/links and explicit output_budget entries; it never silently truncates a relation. There is no continuation or persistent cursor. References are the inverse of the proven direct links over this explicit source set. Transitive lineage is not returned in this protocol version.

SQLGlot owns dialect parsing, normalization, strict qualification and scope resolution. Supplied schema sources support declarative CREATE TABLE definitions, preserving each file's local offsets independently. Duplicate normalized table/column definitions refuse the catalog. Base targets additionally require MappingSchema proof. Explicit aliases and nearest CTE/derived outputs retain original declaration spans; generated aliases and transformed stars never become invented source locations. Original relation/CTE alias-column lists are refused before qualification in every profile; positional renaming is not inferred. Qualifier links require an actual qualifier occurrence on the original column, not metadata copied onto a generated qualifier. Unsupported DML is refused, while star expansion, JOIN USING and recursive CTE coverage cannot be reported complete. No namespace default or migration-state interpreter is added. Consecutive opening-brace code tokens are refused through SQLGlot's tokenizer; brace text in strings/comments remains data. This is a narrow unsupported-template boundary, not comprehensive recognition of a template language.

The prior standalone qualification passed 23 mandatory cases after independently reviewed provenance/oracle corrections; JOIN USING, recursive CTE output bindings, DML and expanded-star references remained explicitly outside complete coverage. That evidence used a development interpreter and is not evidence of this new leaf's runtime behavior. `tests/sql-navigation-smoke.py` exercises the actual leaf with caller-supplied absolute `--python` and `--wheel` paths; an optional `--qualification-fixture` runs the preserved synthetic gold as well. The new leaf passed 58 actual smoke checks on macOS ARM64 with the relocated private CPython 3.13.17 qualification candidate, including the preserved 27-case fixture and strict frame/type/source/node/depth/output bounds. The 27.42-second run used network-denied finite children; each returned no stderr and left project sentinels unchanged. Tested leaf SHA-256: `2dd4352df38a5a5d1705bc7280be1f7e8a5770f82a79777544a80726e8d89693`. The same 58 unchanged checks also passed on the smaller selected headless private runtime. These are finite behavior fixtures, not proof of complete SQL coverage.


## MCP integration

`sql_navigate` accepts the current release identity, an explicit project root, a dialect, and query/schema files with their expected source revisions. Rust reads project-scoped snapshots, sends only source bytes and opaque IDs to Python, validates every returned span against the original UTF-8 text, and rechecks all source revisions before returning links. A changed source returns an error rather than stale navigation. The post-check does not create an atomic multi-file snapshot. The complete public result, including path mapping, is bounded to 32 KiB; an oversized envelope returns an explicit error.

SQL jobs share the existing two-heavy-job/eight-waiter admission and finite deadlines. MCP cancellation signals the retained job owner and waits for cleanup; connection EOF and abrupt server death use the existing parent lifeline. Cleanup uncertainty retains the ownership marker. There is no separate SQL broker, persistent database or artifact-publication receipt.

Development `serve` requires all three explicit flags together: `--sql-python`, `--sql-script`, and `--sql-wheel`. An unconfigured runtime reports SQL unavailable. The macOS `documents` release component supplies the same paths from its verified installed payload, so installed `serve --home` needs no developer payload flags. Core-only releases still report SQL unavailable.

`tests/sql-mcp-smoke.mjs` passed on macOS 26.5.2 ARM64 with the selected private Python and Node runtimes, under an OS network-deny policy. It exercised real navigation with Unicode and CRLF byte spans, ambiguous bindings, stale release/revision and outside-project refusal, source changes after snapshot, MCP cancellation notifications, connection EOF and abrupt parent death. Owned leaf processes exited and completed scratch directories were removed; a final real SQL request succeeded after cancellation. Existing text/JSON, configured semantic-memory, transfer/restore and release-activation regression checks remain separate shared-runtime checks. Native Windows, oldest supported macOS, desktop-client activation, full toolchain packaging and complete planned SQL coverage remain unqualified.


The separate installed fixture `tests/documents-installed-smoke.mjs` also passed under network denial on the same platform. It assembled the exact selected Python/Node/SQLGlot closure, staged and activated it in a disposable private home with spaces and Unicode in its path, and performed real revision-bound Unicode SQL navigation using only `serve --home`. Full installed inventory, normalized permissions, both interpreter aliases and project sources remained unchanged. Tampered Python bytes and an escaping alias in the candidate were rejected without changing the active release. This establishes the local installed SQL profile; it does not qualify setup into a user's existing installation or native desktop activation.

## Additional bounded dialects — 2026-10-25

The same pinned SQLGlot 30.24.0 wheel supports three additional explicitly
selected profiles. No dependency, database client, service or interpreter was
added. Dialect selection alone does not establish complete dialect or server
compatibility.

| Public identifier | Qualified bounded behavior | Additional limits |
| --- | --- | --- |
| `mysql` | Backtick-quoted and escaped identifiers, qualified database/table declarations, Unicode/CRLF byte spans, aliases, joins, CTE/derived outputs and `LIMIT`. | Exact-case matching is conservative: mismatches refuse instead of guessing server name/collation settings. Versioned executable comment markers `/*!` and `/*M!` refuse even inside literals/comments. |
| `tsql` | Bracket-quoted and escaped identifiers, schema-qualified declarations, original-spelling spans under SQLGlot's case normalization, aliases, joins, CTE/derived outputs and `TOP`. | No batches, dynamic variables, server collation modes or synthesized relation layouts are qualified. |
| `duckdb` | Double-quoted and escaped identifiers, schema-qualified declarations, case normalization with original-spelling spans, aliases, joins, CTE/derived outputs and a column-bound `QUALIFY` case. | No external-file/table functions, dynamic parameters or synthesized relation layouts are qualified. |

For these three profiles, `SELECT INTO`, parameters/placeholders, pivot,
unnest/lateral and non-identifier table-function relations are explicitly
refused. Star expansion, `JOIN USING` and recursive CTEs still cannot be marked
complete. Missing or ambiguous declarations refuse instead of inferring a
plausible base object. These are offline source/catalog contracts; they do not
connect to a server, execute SQL or verify a deployed server's configuration.
`mariadb`, `sqlserver`, uppercase aliases and unqualified dialect names are
rejected. MariaDB and other dialects remain unqualified and are not exposed as
planned support through an implicit alias.

The additional fixtures derive expected links from hand-specified declarations
and original byte positions, independently of rewritten SQL. They compare full
relations for quoted Unicode/CRLF sources, escaped delimiters, CTE and derived
outputs, and keep identically named table/column declarations in separate
namespaces/sources. Discriminating cases cover ambiguity, unknown columns,
unproven statements, executable comments, case mismatch and unsupported
coverage. SQLite/PostgreSQL regression, request/response budgets, identity and
quiet refusal checks remain in the same finite fixture.

Reproduce against explicit private inputs:

```sh
python3 tests/sql-navigation-smoke.py \
  --python /absolute/private/python3.13 \
  --wheel /absolute/release/libraries/sqlglot-30.24.0-py3-none-any.whl

/absolute/private/node tests/sql-mcp-smoke.mjs \
  --mirket /absolute/frozen/mirket \
  --node /absolute/private/node \
  --adapter /absolute/adapters/dist/main.js \
  --python /absolute/private/python3.13 \
  --sql-script /absolute/adapters/python/sql_navigation.py \
  --wheel /absolute/release/libraries/sqlglot-30.24.0-py3-none-any.whl
```

The new-profile evidence is local macOS ARM64 qualification of the finite leaf
and development MCP path. It does not establish installed-release identity,
Codex GUI/model behavior, native Windows, oldest macOS or full dialect coverage.
Any installed distribution must bind the changed leaf bytes in its manifest and
repeat its affected installed checks.

The initial additional-dialect Python fixture passed **84 groups** in 43.76 seconds on macOS
26.5.2 ARM64 with private CPython 3.13.17, each child under OS network denial.
The guarded locked/offline Rust run passed all **nine SQL transport tests**.
The expanded MCP fixture also passed under OS network denial, including the
three new dialects' saved-source spans and refusals followed by the shared
revision/cancellation/EOF/parent-death regression cases. The leaf SHA-256 is
`082c74109c0d51e591fed476af6a33e1581b32133316b0e2ee2731d5e2b652b4`.
The frozen MCP core SHA-256 is
`8083f66b713190b5ec1a6e84aa0e2187094052d0fd6bff651bc922f7941a84be`;
an unrelated CLI source changed during its build, so the integrated final core
still requires the owning lane's repeat. Logs and input/source identities are
retained outside shipped source in the implementation task's
`sql-additional-dialects-*` evidence. These single runs are correctness fixtures,
not performance benchmarks.

## Original alias provenance repair — 2026-10-25

Independent review exposed a false-complete binding for
`SELECT x.id FROM a AS x(name,id)` with declared columns `(id, name)`:
the positional relation rename could be discarded during qualification and
incorrectly resolve `x.id` to the original `id` declaration. The leaf now
refuses any original `TableAlias` with a nonempty column list before
qualification, across all five profiles. This includes base tables, derived
relations, CTE declarations, aliases on CTE references and nested scopes.
The static response is `unsupported_statement` with no links for that query;
independently proven results for other query sources remain available.

A second finding involved `SELECT id FROM a AS x`: a generated qualifier could
copy the alias declaration's metadata and produce an alias self-reference.
The leaf now records each original column's qualifier occurrence before
qualification and emits qualifier links only from that original occurrence.
Returned column references must also match the original query source and
column span. This suppresses synthesized links without inventing a mapping.
CTE/derived projections lacking a proven original output declaration remain
incomplete; an explicit output alias can provide that declaration.

Regression fixtures cover all five profiles, including the reviewer's CTE
relation example, positive ordinary aliases, exact unqualified base/CTE/derived
relations and mixed-query preservation. Controlled fixture-only mutations
restore each reported defect when its corresponding guard is removed,
demonstrating that the checks distinguish the broken behavior. Their synthetic
results and source identities remain outside shipped source in
`sql-original-alias-regression/`.

Final repair verification passed **134 leaf test groups** in 81.42 seconds
with the same pinned private Python/wheel inputs and OS network denial.
The expanded MCP fixture passed against the frozen core above and the changed
leaf, including all five profiles' original alias-column refusals and
unqualified-reference provenance cases. The repaired leaf SHA-256 is
`9861f57a3b409e06ab224d2e77f344f9f26a15cc62d0c5e608bc1b26a034b796`.
The current evidence record is `sql-original-alias-guards-20261025.json`.
After that repair, all five profiles passed the disposable installed-release
smoke in `tests/combined-installed-smoke.mjs` on macOS ARM64. The native Codex
backend fixture passed installed MCP discovery and release transition without
a model turn. Those journeys do not expand the bounded SQL syntax contract or
qualify Windows execution.
