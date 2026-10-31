# Private native builds

Development and test profiles retain line-table debug information and disable
incremental compilation to bound disposable build-cache growth. They remain
unoptimized; their timings do not establish release-build performance.

For build-cache cleanup, preserve `target/native`: it contains pinned native inputs,
not disposable Cargo outputs. Removing `target/debug`, `target/release`, old
cross-target outputs and `pi/dist/runtime` is safe after jobs stop; the next build
regenerates them. A blanket `cargo clean` also removes the native input and
requires preparing it again before the build guard can pass. Keep the locked
development dependency tree when an offline rebuild is needed.

Mirket's canonical store links the reviewed official SQLite 3.54.1 source as a private static library. Its store does not use the operating system's SQLite. The exact source archive, two retained source members, compile defines and target options are recorded in `packaging/runtime/sqlite.json`; Cargo uses rusqlite's `modern_sqlite` and `backup` features, never its outdated `bundled` C snapshot.

Prepare the native input before any Cargo compilation. The build script accepts only an already downloaded archive and explicit existing compiler/archiver paths. It does not download, install, invoke upstream build helpers or run the resulting SQLite code.

On macOS ARM64, from the repository:

```sh
python3 scripts/build-sqlite.py \
  --archive /absolute/path/sqlite-amalgamation-3540100.zip \
  --target aarch64-apple-darwin \
  --compiler /usr/bin/clang \
  --archiver /usr/bin/ar
python3 scripts/check-build-inputs.py
cargo test --locked --offline
```

The output is one target at a time:

```text
target/native/sqlite/
  lib/libsqlite3.a          # sqlite3.lib on Windows
  include/sqlite3.h
  build-receipt.json
```

The build verifies archive size/hash, selected member identity and regular-file type, then compiles the unchanged C source with the pinned flags. It checks the produced object's architecture and static archive signature, hashes the library/header, validates the complete receipt and publishes the prepared directory. No source or command path is taken from an archive member. Relative compilation source names and `ZERO_AR_DATE=1` avoid temporary paths and wall-clock archive timestamps in macOS library bytes. Exact compiler and command receipts remain necessary; identical bytes across compiler/SDK versions or all platforms are not promised.

A repeated build replaces only a previous output that passes the same receipt checks. Unknown entries, changed files, links or competing libraries are refused, preserving the existing directory. If interrupted during the narrow directory replacement window, inspect the task-owned `.sqlite-build-*` sibling before recovery; do not describe this build step as a crash-transactional installer. Do not run native rebuilding concurrently with Cargo or another native build. This is a trusted-checkout drift guard, not protection against an attacker who can replace the manifest, receipt and binary together.

Repository-local `.cargo/config.toml` forces `SQLITE3_LIB_DIR` and `SQLITE3_INCLUDE_DIR` to these relative project paths, static linking, and disabled pkg-config discovery. It does not modify shell startup files, global Cargo settings or the parent environment. The explicit-directory branch in the reviewed libsqlite3-sys build script returns before vcpkg lookup; `SQLITE3_NO_PKG_CONFIG=1` stops its pkg-config probe before command execution. No system-library fallback is allowed. Invoke Cargo from this repository; when using `--manifest-path` from another working directory, also supply `--config /absolute/repository/.cargo/config.toml`, because Cargo configuration discovery depends on its working directory.

`scripts/check-build-inputs.py` checks the native receipt/target/header/library and selected dependency features before compilation, and retains the rmcp source-layout check that prevents its upstream Git-hook mutation. Pass `--target` when it differs from the host, matching Cargo's target. `build.rs` independently checks native paths, source identity, target, hashes and directory entries before the final Mirket link, including for direct Cargo invocations. Missing native inputs fail explicitly. These checks do not silently repair or download anything.

On actual Windows, supply `--target aarch64-pc-windows-msvc` or `x86_64-pc-windows-msvc`, and the absolute target-configured MSVC `cl.exe` and `lib.exe` paths from an existing developer environment. The script compiles with `/MT`; repository target configuration pairs that with Rust `crt-static`. The produced COFF machine is checked against the requested target. `build.rs` rejects a Windows build without `crt-static`, including when external `RUSTFLAGS` overrides target configuration. No MSVC installation or SDK configuration is performed. Native Windows build, CRT imports and runtime behavior remain unqualified here; a macOS build cannot establish them. Linux and other targets are currently unsupported.

After preparing a new native input, perform version/source ID, FTS5, extension-loading-denial, transaction and backup checks, plus application tests. Library/hash validation alone cannot establish SQL correctness, memory semantics, supported OS-floor execution or native client acceptance.


## Semantic source checks and private model/runtime assembly

The semantic dependency closure is pinned for the macOS ARM64 private runtime; Windows release acceptance remains open. Mirket stores normalized embeddings as ordinary SQLite blobs and scores eligible rows with checked Rust cosine similarity. No sqlite-vec extension or additional native vector build is required.

Repository `.cargo/config.toml` also forces `RUSTONIG_SYSTEM_LIBONIG=0`, `RUSTONIG_DYNAMIC_LIBONIG=0`, `RUSTONIG_STATIC_LIBONIG=1`. The pinned Oniguruma source is compiled by its reviewed build script instead of discovering a host library. The pre-build checker now verifies exact FastEmbed/ort/ort-sys/tokenizers/onig/onig_sys/esaxx versions and features, hashes the registry archives, and compares every archived source file with its extracted source tree before compilation. In particular, ort-sys must retain `disable-linking`; download/model-hub/image/GPU features are absent. `build.rs` independently validates private SQLite inputs, target/source/header identity and forced Oniguruma settings. The pre-build dependency check must still run: a crate's own build script cannot run before every dependency build script in Cargo's graph.

The private runtime/model package uses another stable authoritative manifest, `packaging/runtime/semantic.json`. Assembly only reads already fetched inputs and does not execute the runtime or model:

```sh
python3 scripts/build-semantic.py assemble \
  --target aarch64-apple-darwin \
  --runtime-archive /absolute/path/onnxruntime-osx-arm64-1.31.0.tgz \
  --model-root /absolute/path/verified-model-directory \
  --output /absolute/empty/semantic-package
python3 scripts/check-build-inputs.py \
  --semantic-package /absolute/empty/semantic-package
```

The model directory must preserve the manifest's `onnx/`, `1_Pooling/`, `sentence_bert_config.json` and `README.md` paths. Output contains only the selected versioned runtime dylib, original runtime licenses/notices, INT8 model/tokenizer/config/card files and a package receipt. No npm/pip/hub, installers, model-fetch helpers, alternate EPs or example libraries are present. The one exact upstream compatibility symlink is recognized but never extracted; unknown links and unsafe/duplicate paths are refused. Input and output paths are explicit absolute paths. Output must be empty, and its parent must already exist.

Package verification checks every actual file against the authoritative source pins, then checks the receipt; changing both bytes and a self-authored receipt is insufficient. The runtime implementation must embed the expected source identities from the authoritative manifest and validate them again before loading an absolute private library/model path. A receipt's own expected hashes are not a trust root. Do not embed a developer's task-state absolute path in a shipped executable. Missing or modified assets fail explicitly; no fallback download or ambient `ORT_DYLIB_PATH` initialization is allowed.

Both Windows variants currently fail assembly completeness before creating output because their exact licensed private Visual C++ runtime payload has not been supplied and verified. Native Windows SQLite and semantic runtime behavior have not run here; they require a Windows host. No host CRT fallback, installation or cross-platform execution is substituted for Windows qualification.

The assembled macOS MiniLM payload is 172,668,906 B excluding its receipt.
Package checks do not establish model lifecycle, retrieval usefulness or other
platform support. The [current qualification](qualification.md)
records installed outcomes and actual resource measurements.

## Locked source and adapter gates

Use the existing verified private Node executable and locked dependency tree.
Do not invoke upstream install hooks or silently obtain another runtime. After
preparing native inputs, run from the repository:

```sh
python3 scripts/check-build-inputs.py
cargo test --locked --offline
cargo clippy --locked --offline --all-targets -- -D warnings
cargo fmt --check
cargo build --locked --offline
python3 -m unittest discover -s tests -p 'test_*.py'
```

Set `MIRKET_NODE` to the absolute verified private Node executable for the candidate
being tested. The adapter build checks required direct production/development package names
and versions against `package.json` and `package-lock.json` before importing the
compiler; missing or mismatched entries fail. This version check is not a
cryptographic audit of the transitive dependency tree.
From `pi/`, use that executable to run the locked compiler and process tests:

```sh
MIRKET_PI_PEERS="/absolute/verified-pi-node_modules" "$MIRKET_NODE" scripts/build.mjs
"$MIRKET_NODE" --test tests/*.test.mjs
```

Then, from the repository root, pass explicit absolute executable paths to the
real stdio fixture:

```sh
"$MIRKET_NODE" tests/mcp-smoke.mjs \
  --mirket "$PWD/target/debug/mirket" --node "$MIRKET_NODE"
```

The fixture creates disposable state and projects. Any CI job should run the
same guard, Rust, Python, locked adapter and MCP gates with explicit verified
executables and isolated state; passing source tests does not qualify Pi TUI,
image delivery, cancellation or another platform. Do not check in a developer's
`.pi/settings.json`, duplicate Mirket's MCP registration or copy its package
skills into `.agents`. CI has no agent-specific configuration in this checkout;
install and initialize Mirket locally for agent-assisted development, not as a
substitute for the build gates.

Installed integration tools under `tests/` require their documented explicit
release/home/tool arguments; never point them at the user's existing `.mirket`
installation. Model trial runners are optional evaluation tooling, not part of
ordinary test discovery or permission to start new paid/model cohorts.

For project initialization and observed-use integration, run the exact release
binary and private Node through `tests/pi-project-setup.mjs` with four absolute
arguments: admitted Pi `node_modules`, release `mirket`, private `node`, and its
`LICENSE`. It prints a new `/private/tmp/mirket-pi-project-*` fixture root and
retains the assembled source candidate there. Run `tests/pi-project-tui.py`
with `--node`, `--peers` and `--state-root` pointing to that fixture root to
exercise literal Pi TUI init, status and reload against a separate private
home/project. These fixtures use an offline provider and leave the user's
installation untouched.

The host peer tree must match `packaging/runtime/host-lock.json` and
`packaging/runtime/host-files.json`. The build leaves pi and TypeBox external.
`assemble-pi-package.py` copies a selected verified native payload and compiled
extension into a local package; it neither downloads nor installs dependencies.
For package assembly, build the native core with
`cargo build --release --locked --offline`, create the native manifest with
`target/release/mirket`, and pass that exact executable to
`assemble-pi-package.py --mirket`. The assembler rejects a core built with
debug assertions; the `target/debug/mirket` command above is for local MCP
smoke only. A release build is a packaging prerequisite, not a startup timing
qualification.
Native payload paths called `adapters/` are manifest storage, not a second source
or build package. The [qualification record](qualification.md) states the exact
local candidate results and remaining release limits.
