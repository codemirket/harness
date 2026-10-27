# Python navigation qualification — Pyright 1.1.414

Observed **2026-10-25** on **macOS 26.5.2 ARM64, Apple M1 Pro, 16 GiB**, using
the existing private **Node v24.22.1** executable explicitly. This is a bounded
backend experiment for the Codex-only release. It adds no analyzer to the
production manifest, adapter, installed release or client configuration.

**Result: the unmodified npm server passes the prepared, passive fixture after
relocation. Release admission is blocked.** Same-file and cross-file definitions,
an exact four-location reference set, document symbols, a declared dependency
stub and saved-source invalidation worked. Static Python version/platform and
venv metadata avoided interpreter execution. Default or incomplete metadata did
attempt interpreter discovery; upstream did not turn that into a refusal.
Bundled parser advisories, complete embedded-code rights/source accounting and
the integration preflight remain unresolved.

## Identity and acquisition

Primary inputs were retrieved as bytes, not installed with npm/npx or a Python
wrapper. Archive extraction accepted only relative regular files/directories;
no links, special files, installers, hooks, builds or upstream helpers ran.

| Input | Exact observed identity |
| --- | --- |
| [npm metadata](https://registry.npmjs.org/pyright/1.1.414) | `pyright` 1.1.414; `node >=14.0.0`; npm bin `pyright-langserver` → `langserver.index.js` |
| [npm archive](https://registry.npmjs.org/pyright/-/pyright-1.1.414.tgz) | SHA-256 `bf5f473f6167c0d14175492c3263d783b4489a6956e1c06c18e15228e3a3fa42` |
| npm integrity | `sha512-FPZZb51jepDX4eP7TEYDeNFtmE3WgwkkEcJpvH3/QmUSsj0EAy3LXu+xB4T/FWDejtsXlCfFr/rbWFjwuwuXww==`; SHA-512 and registry SHA-1 both matched |
| [source tag](https://github.com/microsoft/pyright/tree/1.1.414) | Annotated tag `7f78dd6e06ae7fab6f0c740b85b7436b2738316d`, commit `a28cce4077d87b80bead12de809f79845e39c8ba`; API reports unsigned |
| [source archive](https://codeload.github.com/microsoft/pyright/tar.gz/refs/tags/1.1.414) | SHA-256 `7a57e948445b2d61a3aa16872b2a8026951df3186f11b8d6d7357e94924569ac` |
| Existing private Node | `releases/m0-aarch64-apple-darwin/runtime/node/bin/node` under the task-state root; SHA-256 `6f32069480ec14d52b6d7b1f3c7124cfbbc46203abf5ddf5c65b127435f3f329` |

The complete unchanged npm payload was copied to
`relocated ç private tools/pyright/`, including its adjacent `dist/` and stubs.
Every copied file matched the extracted archive. The server banner independently
reported `Pyright language server 1.1.414 starting`. The package and tag identities
were compared; this experiment did not rebuild or establish reproducible-build
correspondence between the tag and published JavaScript.

| Measured payload | Bytes / count |
| --- | --- |
| Compressed npm archive | 4,226,827 bytes |
| Extracted payload | 19,457,120 logical bytes; 5,423 files |
| Included typeshed | 12,728,007 logical bytes; 5,410 files |
| Relocated filesystem allocation | 34,700 KiB from macOS `du -sk` |
| Separate native add-on/shared-library/WASM files | None; embedded WASM still exists inside bundled JavaScript |

These figures exclude the already selected Node runtime, source and rights
archives, temporary duplicate copies and future adapter/notice material. They
are not final installation or staging reservations.

## Closure, hooks and rights

The published package has no `preinstall`, `install` or `postinstall` script.
Its `build`, `clean` and `webpack` scripts are build-time tools and were not run.
Its only external dependency is optional **fsevents `~2.3.3`**; registry metadata
also mirrors it in `dependencies`. **fsevents was omitted.** Its exact 2.3.3
[registry metadata](https://registry.npmjs.org/fsevents/2.3.3) declares the
`node-gyp rebuild` install hook and a Darwin-only native package. No fsevents
archive or native code was needed by the passing fixtures. This does not qualify
its use or watchers on another platform.

The [build configuration](https://github.com/microsoft/pyright/blob/1.1.414/packages/pyright/rspack.config.js)
externalizes fsevents and bundles other imports. Published source maps name 37
exact bundled npm modules. Their registry metadata and tarballs were fetched
read-only; all 37 registry integrity values matched. No additional npm module was
installed or loaded from another project during the observer fixture.

| Bundled group | Exact versions from the published source maps |
| --- | --- |
| Filesystem/archive | `@yarnpkg/fslib` 2.10.4; `@yarnpkg/libzip` 2.3.0; `tmp` 0.2.7; `tslib` 1.14.1 |
| Language protocol | `vscode-jsonrpc` 9.0.1; `vscode-languageserver` 10.1.0; `vscode-languageserver-protocol` 3.18.2; `vscode-languageserver-types` 3.18.0; `vscode-languageserver-textdocument` 1.0.12; `vscode-uri` 3.1.0 |
| Configuration/maps | `jsonc-parser` 3.3.1; `smol-toml` 1.7.0; `source-map` 0.6.1; `source-map-support` 0.5.21; `buffer-from` 1.1.2 |
| Watch/glob | `chokidar` 3.6.0; `anymatch` 3.1.3; `binary-extensions` 2.3.0; `braces` 3.0.3; `fill-range` 7.1.1; `glob-parent` 5.1.2; `is-binary-path` 2.1.0; `is-extglob` 2.1.1; `is-glob` 4.0.3; `is-number` 7.0.0; `normalize-path` 3.0.0; `picomatch` 2.3.2; `readdirp` 3.6.0; `to-regex-range` 5.0.1 |
| CLI/color | `ansi-styles` 4.3.0; `chalk` 4.1.2; `color-convert` 2.0.1; `color-name` 1.1.4; `command-line-args` 5.2.1; `has-flag` 4.0.0; `lodash.camelcase` 4.3.0; `supports-color` 7.2.0 |

Source-map identities describe bundled modules, not a complete reproducible build
or all embedded C/runtime content. The 37 packages' declared licenses span MIT,
ISC, BSD-2-Clause, BSD-3-Clause and 0BSD. None declares an install lifecycle hook.
Their other authored scripts remain unexecuted evidence. The published payload
contains the [Pyright MIT license](https://github.com/microsoft/pyright/blob/1.1.414/LICENSE.txt)
and [typeshed's Apache-2.0 and additional MIT terms](https://github.com/microsoft/pyright/blob/1.1.414/packages/pyright-internal/typeshed-fallback/LICENSE).
Preserve the full stubs, metadata, licenses and original attribution; stub package
names do not imply their Python implementations are installed or executed.

The published Pyright payload supplies no comprehensive third-party notice file.
License/notice material was recovered from 35 bundled npm archives. The two Yarn
archives contain no license file; the root BSD-2-Clause license was retrieved at
each package metadata's exact Git revision. The pinned
[libzip build script](https://github.com/yarnpkg/berry/blob/f4af6c7cf1588f80011ec13060099d96be67127c/packages/yarnpkg-libzip/artifacts/build.sh)
names **Emscripten 2.0.22**, **zlib 1.2.11** and the libzip fork commit
**`664462465d2730d51f04437c90ed7ebcbe19a36f`**. It generates single-file embedded
WASM; absence of a separate native binary does not eliminate this closure.
That script was inspected only. Exact embedded correspondence, complete
libzip/zlib/Emscripten source/notice obligations and their advisory review are
**open release gates**. No production notice bundle was added by this task.

An OSV exact-version batch query for Pyright plus the 37 bundled npm modules
returned two advisories for the bundled **smol-toml 1.7.0**:

- [GHSA-7w5x-hrqm-74c2](https://github.com/squirrelchat/smol-toml/security/advisories/GHSA-7w5x-hrqm-74c2): malformed TOML can loop indefinitely; fixed in 1.7.1.
- [GHSA-r4xh-jqrq-34v2](https://github.com/squirrelchat/smol-toml/security/advisories/GHSA-r4xh-jqrq-34v2): quadratic parsing of ordinary TOML; fixed in 1.9.0.

Pyright's project service can parse `pyproject.toml`, so this is a relevant input
boundary. The adversarial TOML payloads were not executed in this experiment.
The API query is dated evidence, not a proof that other components have no
vulnerabilities. A fixed, exactly pinned upstream payload or a separately
qualified rebuilt closure is required before release; editing a minified vendor
file would not close that gate.

## Passive-loading experiment

The [upstream server](https://github.com/microsoft/pyright/blob/1.1.414/packages/pyright-internal/src/server.ts)
creates `FullAccessHost`. Its
[interpreter host](https://github.com/microsoft/pyright/blob/1.1.414/packages/pyright-internal/src/common/fullAccessHost.ts)
uses `execFileSync`: the version probe supplies `-I`; the search-path probe
supplies neither `-I` nor `-S`. Removing the current directory from `sys.path`
inside `-c` occurs after Python startup, so it cannot establish absence of
startup-hook execution. Without an explicit path it tries `python3` then
`python`. Merely selecting an interpreter path still executes it.

The [static venv resolver](https://github.com/microsoft/pyright/blob/1.1.414/packages/pyright-internal/src/analyzer/pythonPathUtils.ts)
returns prepared site-packages when it finds them. It reads `.pth` path entries
as data and ignores `import` lines. An explicit `pythonPath` additionally invokes
the interpreter even when static site-packages exist. A configured Python
version avoids the separate version probe.

Two hand-authored `.py` files use CRLF, Turkish filenames/identifiers and emoji
before call sites. A declared dependency `.pyi` is reachable through an ordinary
`.pth` path. Project modules, `sitecustomize.py`, `usercustomize.py`, a startup
file, a setup file and an executable `.pth` line write distinct markers if run.
A fake build backend/requirement is present in `pyproject.toml`; executable
sentinels cover Python, npm/npx/pip/node-gyp, shells and Cargo. Only disposable
fixture state is used.

Prepared configuration supplies `pythonVersion: "3.13"`,
`pythonPlatform: "Darwin"`, an absolute private metadata `venvPath`,
`venv: "prepared"` and an existing `lib/python3.13/site-packages` directory.
It supplies **no interpreter path**. Client settings disable automatic source
search paths, library implementation typing and auto-import completions. No
command, code action, formatter, package installer or build request is sent.

The observer run preloads a task-owned logger that blocks and records Node child
process/network APIs and module loading outside the pinned payload; macOS also
denies network access. This is test instrumentation, not a production sandbox
or a complete process-security claim. The **independent unmodified-server run**
has no observer and runs under macOS network denial, fork denial and exec denial
except the explicitly selected Node executable. A control returned `EPERM` for
both a child `touch` and a localhost connection, and produced no marker.

| Scenario | Observed result | Passive-policy conclusion |
| --- | --- | --- |
| Prepared metadata, observer | Seven assertions pass; zero recorded process, network or outside-module attempts; no marker | Bounded passive path passes |
| Same prepared form, unmodified server with OS denial | Same seven assertions pass; clean exit | Does not require observer replacement of APIs or successful subprocess/network use |
| Missing metadata | Local definitions/references still work; dependency definition is `null`; ten blocked Python discovery attempts; defaults to Python 3.14 in logs | Must refuse before launch; plausible local output is insufficient |
| Missing prepared venv | Local operations still work; dependency definition is `null`; eight blocked Python discovery attempts | Must refuse before launch |
| Explicit interpreter in LSP settings, valid static venv | Seven assertions pass but six blocked attempts to the exact interpreter sentinel | Must refuse this execution route |

All final runs shut down via LSP `shutdown`/`exit` with exit status zero. No
project/startup/`.pth`/executable sentinel fired, and no task-owned language-server
Node process remained. Negative-case assertion passes mean the expected unsafe
attempt/degraded output was observed; they do not qualify those configurations.

An additional Node `--permission` experiment failed before initialization:
bundled Yarn filesystem code attempts to change a read-only `path.resolve`
property. Broadening filesystem read permission did not fix it. The exact
diagnostic is retained; do not propose that flag as a tested portable enforcement
mechanism for this payload. The passing OS-denial run uses ordinary Node.

## Exact navigation and measurements

Both prepared runs passed these seven assertions: same-file definition after an
emoji, cross-file definition through a Unicode module name, exact cross-file
references, prepared dependency stub definition, unresolved-name `null`, source
document symbols, and disappearance of the former target after a saved
declaration rename plus explicit change notifications. No result was accepted
merely because a capability was advertised during initialization.

The client offers UTF-16 positions; upstream omits `positionEncoding`, using the
LSP default. The exact reference set converted against saved original bytes is:

| Original file / symbol | LSP line / UTF-16 columns, zero-based | Original UTF-8 byte interval |
| --- | --- | --- |
| `kütüphane.py`, declaration | 3 / 4–9 | `[112,117)` |
| `kütüphane.py`, same-file call | 7 / 26–31 | `[220,225)` |
| `istemci.py`, import | 2 / 22–27 | `[133,138)` |
| `istemci.py`, call after emoji | 5 / 23–28 | `[209,214)` |

Each interval slices the original bytes to `selam`. The fixture includes CRLF
bytes in the offsets. Full URIs, request/response bodies, dependency Unicode
spans, expected sets and source SHA-256 values are saved. The backend returns
`null` for an unresolved definition; it does not supply Mirket's missing-metadata
or source-revision result contract.

| One measured process per prepared mode | Observer | Unmodified / OS denial |
| --- | --- | --- |
| Initialize response | 117.807 ms | 117.347 ms |
| First definition, including deferred analysis | 196.314 ms | 175.092 ms |
| Later cross-file definition | 1.802 ms | 1.316 ms |
| Exact references | 2.034 ms | 1.727 ms |
| Prepared dependency definition | 37.485 ms | 37.126 ms |
| macOS `time -l` maximum RSS | 198,705,152 bytes | 195,166,208 bytes |
| Swaps reported | 0 | 0 |

These are local single-run observations with filesystem caches potentially warm
and concurrent desktop workload uncharacterized. They include the fixture,
protocol and launcher costs as described, not a cold-install benchmark, p95,
large-project scale result or aggregate multi-client memory measurement.

## Minimal integration boundary and preflight

Keep the existing Rust project/revision/job owner and compiled Node adapter.
Use one connection-owned private Pyright process through ordinary stdio LSP;
do not add a Python package wrapper, executable discovery or a second resolver.
The candidate suggests this narrow contract, **not implemented by this task**:

```text
python_navigation(project_id, project_model_revision,
                  operation: document_symbols | definition | references,
                  source: {path, sha256}, position_utf8?,
                  include_declaration?, max_results, deadline)
  -> backend/version/payload_digest, project_model_revision,
     participating_source_revisions, original_utf8_locations,
     status: complete | incomplete | missing_metadata | stale |
             unsupported | truncated | failed,
     coverage_and_limits
```

Before starting/reusing the analyzer, the owner must:

1. Verify manifest-bound Node/server/stub identities, project scope, source
   revisions and a finite prepared model containing Python version/platform,
   import/stub roots, accepted configuration identity and canonical allowed
   dependency roots. Missing or changed metadata refuses before any server spawn.
2. Require valid static site-packages metadata and **no `pythonPath`** in supplied
   LSP settings. Reject missing venvs, unreviewed `extends`, environment-dependent
   paths, out-of-scope `.pth` path entries and configuration that changes the
   execution/read boundary. An interpreter selection is host preparation; its
   executable is not an input to the passive analyzer route.
3. Supply only validated configuration as data. Upstream discovers root
   `pyrightconfig.json`/`pyproject.toml`; arbitrary project configuration cannot
   override policy. Do not silently write a generated config into the user's
   project. Use explicitly host-prepared metadata or a qualified owned immutable
   source/model snapshot with exact original-source mapping. That production
   configuration transport and its revision-race handling remain open.
4. Allow only the required LSP methods; answer configuration requests from the
   validated model. Bound diagnostics/output/requests, watch and idle lifetime;
   never expose upstream execute-command or stub-generation routes as navigation.
   A suppressed interpreter error is a policy failure, not permission to report
   a complete result. Reuse the existing two-analyzer admission and cancellation
   lifecycle rather than create another process supervisor.
5. Convert UTF-16 positions against each exact original document, reject invalid
   surrogate boundaries and escaped/out-of-scope URIs, recheck participating source
   and model revisions, and return explicit missing/incomplete coverage. The
   successful saved-change fixture does not replace these owner checks.

A task-local illustrative preflight record rejects the missing, broken and
explicit-interpreter fixtures without spawning. It is not a production
implementation or comprehensive validation test. The source/URI/cache races,
tampered-payload refusals, output limits, cancellation and concurrent-client
lifecycle still require the common LSP integration and its discriminating tests.

## Evidence and remaining gates

Task-owned evidence is retained outside shipped source at:

```text
~/.codex/task-state/harness-rewrite-implementation/
  python-navigation-qualification-20261025/
```

`fetch-evidence.json`, `platform.json`, tag/registry/source captures,
`payload-manifest.json`, `bundled-modules.json`, `rights-inventory.json` and
`rights/` retain identity, scripts and notice provenance. `probe.py` authors the
fixture and records exact `runs/*/transcript.jsonl`, launch settings, sentinels,
stderr and expected/actual assertions. `probe-results.json`,
`original-byte-results.json`, `measurement-summary.json`, `size-summary.json`,
`os-denial-control.json` and `descendant-cleanup.json` summarize final evidence.
Initial null-versus-empty expectation mistakes and the failed Node-permission
hypothesis are separately retained, without replacing the final results.
`evidence-manifest.json` binds saved evidence bytes; hashes alone do not establish
semantic correctness, rights or user acceptance.

Freeze this candidate at **prepared passive backend pass / release blocked**.
Resolve the bundled TOML advisories and embedded archive-code/source/notices
before adding a package to the release. Then implement and test the preflight,
revision/scope/coverage contract and shared process lifecycle. Native Windows
x64/ARM64, oldest-supported macOS, installed-payload ownership, broad Python
project/import forms, native Codex GUI/model behavior and multi-client resource
qualification remain unverified. No Windows or Codex activation claim follows
from this experiment.
