# Selected-source TypeScript graph through a Codex model turn

On 2026-10-26, one actual Codex Desktop-bundled CLI model turn successfully used
the new selected-source `code_navigate` schema against a disposable installed
release. The model resolved a cross-file definition, returned exactly six
references across TypeScript and JavaScript while excluding a shadowed local,
resolved a selected `.d.ts` declaration, and correctly reported a whole-model
missing-binding refusal. The final answer matched independently checked tool
results. This closes the bounded model-use check for this exact schema and
release; it does not establish Desktop GUI activation or general task quality.

## Exact client, installation and invocation

The platform was **macOS 26.5.2 ARM64**. The Desktop-bundled client was
`codex-cli 0.163.0-alpha.11.2`, at
`/Applications/ChatGPT.app/Contents/Resources/codex-cli/CodexCLI.app/Contents/MacOS/codex`,
SHA-256 `248ff9873b6fb35124a86bdffa013a09e1bd17f106385124e3dda73e9eb157ad`.
The client default model and reasoning settings were preserved. Its immutable
backend model identity was not exposed in the saved JSONL, so the evaluator's
own model identity is not attributed to the model under test.

The existing verified disposable installation was reused at
`~/.codex/task-state/harness-rewrite-implementation/typescript-new-schema-installed-20261026/home`.
Its exact active release was
`sha256:888cd610a31b0b433ed29eb96ac7f8d4c7e2fc782d186d2a30fb1b8816d27d8d`.
The normal installed stable launcher served the model; no development server,
replacement installation or runtime fallback was used. `runtime_status`
returned that release, configured navigation, and the selected-source graph
coverage. The fixed backend was `typescript-language-service` **5.9.3**, using
private Node **24.22.1** and the existing admitted design component.

The selected source-payload identities were independently checked:

| Payload | SHA-256 |
| --- | --- |
| `bin/mirket` | `75e557a48ee52559b9811bd6796e1133d5821ed6f0a119e000bc1b6a7f3d4239` |
| `adapters/code-navigation-main.js` | `3615e15a8415a1741e9cd87be80b1dcbbc5c1111435062e74b19c0c531a0e7c0` |
| `adapters/code-navigation.js` | `273ac5617ffa31d2b1ed9769145eb9751d041c22e701e454c6cef8d7055ab73e` |
| `adapters/node_modules/typescript/lib/typescript.js` | `3ae902c92cc44dace175c0e69e13a4b0899f6983c6121d76b9ab8dd5795e7675` |
| `runtime/node/bin/node` | `6f32069480ec14d52b6d7b1f3c7124cfbbc46203abf5ddf5c65b127435f3f329` |

The invocation used `exec --strict-config --ignore-user-config --ignore-rules
--ephemeral --skip-git-repo-check --sandbox read-only --color never --json`,
`project_doc_max_bytes=0`, a fresh **non-Git** project and a task-owned `HOME`.
Command-line MCP overrides selected only this disposable launcher's
`serve --home …`, `startup_timeout_sec=60`, `required=true`, and the enabled
tools `runtime_status`, `file_read`, `code_navigate`. No global catalog-grace
override was supplied. These are one-run configuration overrides, consistent
with the fetched official [Codex advanced configuration documentation](https://learn.chatgpt.com/docs/config-file/config-advanced#one-off-overrides-from-the-cli).
The complete argv, prompt and explicit environment overrides are retained in
`request.json`; there is no shell command-string interpolation in the runner.

Codex consumed the existing real authentication `CODEX_HOME` itself. The evaluator
did not read, copy or print credential bytes. Exact passive guards checked the
real `~/.codex/config.toml` before and after the model process and again during
independent verification: SHA-256
`7cf4d5f11d2a6593a35df91b3fa347638bc614ab3c165c231f1e51e613898317`,
mode **0600**, unchanged throughout. This avoids the disposable Git-project
trust-write observed in the separate
[sampled skill evaluation](m4-project-skill-evaluation.md#trust-write-attribution-restoration-and-continuation).
The guards observe preservation for this run; they are not enforcement against
future client writes. Inherited guidance from the real authentication home was
not proven isolated.

## Frozen task and observed result

The fresh project path contains spaces and `İ`. Four original saved snapshots
were frozen before dispatch; the library has a UTF-8 BOM, emoji, Turkish text
and CRLF, and every snapshot uses CRLF. The explicit graph has three edges:
source 1 `./library` to 0, source 1 `prepared` to 3, and source 2 `./library` to 0.
The TypeScript consumer also contains a same-named `selam` parameter and use
whose lexical scope must remain separate from the import.

| Index | Saved file | SHA-256 revision |
| --- | --- | --- |
| 0 | `library.ts` | `47b60fbcca29e78129ecacb625c9072d3f9b7a82dcc5ccfea0d34779805ec476` |
| 1 | `consumer.ts` | `8de45457372188b6424f2ad156fb3bceecf969c3164cc60dc4b951c3458d1592` |
| 2 | `consumer.js` | `acdcd66e3895e02b244fb502029768442746688c11b0a593408ac6babe53804f` |
| 3 | `prepared.d.ts` | `b0f2d0b0a3f51123c659f3b2950c7685254eaa50353561e1d36a8534f16d2dd1` |

The model received exact paths, revisions, language kinds, query byte offsets
and the explicit import table. It was asked to use the semantic results and
was allowed metadata-only discovery and reads of these sources. It made exactly
**five successful MCP calls**: one `runtime_status` and four `code_navigate`.
There were no shell, file-read or other tool items, source writes or repairs.
One planned trial completed with exit **0** in **23.578 seconds**, within its
120-second deadline; there were no earlier failed trials or retries.

| Query | Actual result, original UTF-8 byte ranges |
| --- | --- |
| Definition of source 1 `selam` call at byte 147 | Source 0 declaration `[38,43)` |
| References from source 0 declaration at byte 38 | Source 0 `[38,43)`, `[101,106)`; source 1 `[9,14)`, `[147,152)`; source 2 `[9,14)`, `[60,65)` |
| Definition of source 1 `prepared` call at byte 188 | Source 3 declaration `[24,32)` |
| Same `selam` definition query with only the `prepared` edge omitted | `refused`, `missing_import_binding`, no locations |

The reference result excluded source 1's same-spelled shadowed parameter/use.
Each returned reference had the service's boolean `is_write`. All three positive
queries had `complete:true` with `limit:100`; that flag means no query-list
truncation. Every navigation result, including the refusal, retained
`model_complete:false`. The refusal did not carry `complete` or partial locations.
The model's final answer accurately preserved these distinctions.

Independent verification compared every requested source and edge with the
frozen inputs, required structured/text result equality, checked release and
source revisions, compared exact locations against the pre-dispatch oracle,
and decoded original saved byte slices as `selam` or `prepared`. It separately
recomputed the ordered graph digests from each recorded model request. The
positive model revision was
`sha256:36d2b9a7f19d2668dfb86beee795f174fda9ee4ca77ffb1756787bbb070c4fce`;
the deliberately incomplete graph was
`sha256:9509cb5f9b657cdfc89f2c77898a3c32bf852619ca020a1a243e998f368baf35`.
All fixture source hashes and the exact project file set remained unchanged.
Post-run `dev doctor-core --json` reported the same installed release healthy,
and no owned Mirket or Node process remained.

## Evidence and limits

Private evidence is retained outside shipped source at
`~/.codex/task-state/harness-rewrite-implementation/typescript-codex-model-qualification-20261026/`:
`qualify.py`, `frozen-inputs.json`, `request.json`, `process.json`, full
`events.jsonl`, `stderr.log`, `final.txt`, `summary.json`, `independent-checks.json`,
`post-doctor.json`, `payload-and-cleanup.json`, `document-checks.json`, the four
original synthetic sources and `evidence-hashes.json`. The full model trace
SHA-256 is `9b485287b2b71849d871c49101102de02958389682dfe3caa86008282818ecaa`;
the independent result check SHA-256 is
`fb64ab71f0d5fa53748a3e7d94f3cc7c9ea8dd6a889d60ab82b070b291e430a2`.
These are retained provenance, not product files. The empty fake home was removed.
Only this new document was authored in the repository by this evaluation.

The client reported 88,819 input tokens, including 71,552 cached input tokens,
and 971 output tokens; monetary cost and immutable backend model revision are
unknown. Its 134-byte stderr contained one rollout state-database discrepancy
warning. The completed tool calls were error-free; the warning is retained
without claiming network silence or a fully isolated real client state.

This is one public developmental case with caller-prepared revisions, positions
and bindings. It does not qualify automatic graph preparation, package identity,
whole-project type correctness, standard-library/framework inference, JSX/TSX
model use, Windows, macOS 15, model-driven approval, Desktop GUI activation,
repeated or held-out reliability, human acceptance or performance ceilings.
The separate [semantic navigation implementation and installed smoke](m3-typescript-semantic-navigation.md)
cover the broader finite schema cases; this turn adds actual model use for its
specific TS/JS/declaration graph. No current user installation, user config,
old harness, dependency set, commit, push or publication was changed.
