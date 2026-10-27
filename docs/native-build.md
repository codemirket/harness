# Private native builds

Development and test profiles retain line-table debug information and disable
incremental compilation to bound disposable build-cache growth. They remain
unoptimized; their timings do not establish release-build performance.

Mirket's canonical store links the reviewed official SQLite 3.54.1 source as a private static library. Its store does not use the operating system's SQLite. The exact source archive, two retained source members, compile defines and target options are recorded in `packaging/admission/m2-sqlite.native-build.json`; Cargo uses rusqlite's `modern_sqlite` and `backup` features, never its outdated `bundled` C snapshot.

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

The build verifies archive size/hash, selected member identity and regular-file type, then compiles the unchanged C source with the admitted flags. It checks the produced object's architecture and static archive signature, hashes the library/header, validates the complete receipt and publishes the prepared directory. No source or command path is taken from an archive member. Relative compilation source names and `ZERO_AR_DATE=1` avoid temporary paths and wall-clock archive timestamps in macOS library bytes. Exact compiler and command receipts remain necessary; identical bytes across compiler/SDK versions or all platforms are not promised.

A repeated build replaces only a previous output that passes the same receipt checks. Unknown entries, changed files, links or competing libraries are refused, preserving the existing directory. If interrupted during the narrow directory replacement window, inspect the task-owned `.sqlite-build-*` sibling before recovery; do not describe this build step as a crash-transactional installer. Do not run native rebuilding concurrently with Cargo or another native build. This is a trusted-checkout drift guard, not protection against an attacker who can replace the manifest, receipt and binary together.

Repository-local `.cargo/config.toml` forces `SQLITE3_LIB_DIR` and `SQLITE3_INCLUDE_DIR` to these relative project paths, static linking, and disabled pkg-config discovery. It does not modify shell startup files, global Cargo settings or the parent environment. The explicit-directory branch in the reviewed libsqlite3-sys build script returns before vcpkg lookup; `SQLITE3_NO_PKG_CONFIG=1` stops its pkg-config probe before command execution. No system-library fallback is allowed. Invoke Cargo from this repository; when using `--manifest-path` from another working directory, also supply `--config /absolute/repository/.cargo/config.toml`, because Cargo configuration discovery depends on its working directory.

`scripts/check-build-inputs.py` checks the native receipt/target/header/library and selected dependency features before compilation, and retains the rmcp source-layout check that prevents its upstream Git-hook mutation. Pass `--target` when it differs from the host, matching Cargo's target. `build.rs` independently checks native paths, source admission, target, hashes and directory entries before the final Mirket link, including for direct Cargo invocations. Missing native inputs fail explicitly. These checks do not silently repair or download anything.

On actual Windows, supply `--target aarch64-pc-windows-msvc` or `x86_64-pc-windows-msvc`, and the absolute target-configured MSVC `cl.exe` and `lib.exe` paths from an existing developer environment. The script compiles with `/MT`; repository target configuration pairs that with Rust `crt-static`. The produced COFF machine is checked against the requested target. `build.rs` rejects a Windows build without `crt-static`, including when external `RUSTFLAGS` overrides target configuration. No MSVC installation or SDK configuration is performed. Native Windows build, CRT imports and runtime behavior remain unqualified here; a macOS build cannot establish them. Linux and other targets are currently unsupported.

After preparing a new native input, perform the version/source ID, FTS5, extension-loading-denial, transaction and backup checks described in `packaging/admission/m2-sqlite.md`, plus application tests. Library/hash validation alone cannot establish SQL correctness, memory semantics, supported OS-floor execution or native client acceptance.


The repository route was qualified on macOS ARM64 on 2026-10-24 under the user's authorization for necessary work within the approved rewrite. Two consecutive native builds with `/usr/bin/clang` and `/usr/bin/ar` produced identical library SHA-256 `4c813989b7c239ac57ad45e5cef18dfc90500c2091decf368d9ef150db46e792` (1,884,880 bytes). The full pre-build guard and `cargo check --locked --offline` passed after root adopted the reviewed dependency delta. Direct checks of the compiled Rust guard accepted valid inputs and rejected ten concrete failures, including library/header drift, an added dylib, target/source-manifest mismatch, missing forced settings, wrong private paths, symlinked header, missing receipt and unsupported target. Corresponding Python receipt checks passed. The tests asserted the intended rejection reason; an initial temporary-path alias was corrected to a canonical path before assessing those cases. Evidence is retained in task-state `dependencies/sqlite/{repository-builder-checks.json,rust-native-guard-checks.json}`. The application executable was not rebuilt by these checks.


## Semantic source checks and private model/runtime assembly

The semantic dependency batch is admitted for the macOS ARM64 private runtime; Windows release acceptance remains open. Mirket stores normalized embeddings as ordinary SQLite blobs and scores eligible rows with checked Rust cosine similarity. The removed sqlite-vec build admission is retained only in `docs/historical-admission/` for provenance. No vector extension or additional native build is required.

Repository `.cargo/config.toml` also forces `RUSTONIG_SYSTEM_LIBONIG=0`, `RUSTONIG_DYNAMIC_LIBONIG=0`, `RUSTONIG_STATIC_LIBONIG=1`. The admitted Oniguruma source is compiled by its reviewed build script instead of discovering a host library. The pre-build checker now verifies exact FastEmbed/ort/ort-sys/tokenizers/onig/onig_sys/esaxx versions and features, hashes the registry archives, and compares every archived source file with its extracted source tree before compilation. In particular, ort-sys must retain `disable-linking`; download/model-hub/image/GPU features are absent. `build.rs` independently validates private SQLite inputs, target/source/header identity and forced Oniguruma settings. The pre-build dependency check must still run: a crate's own build script cannot run before every dependency build script in Cargo's graph.

The private runtime/model package uses another stable authoritative manifest, `packaging/admission/m2-semantic-minilm.runtime.json`. Assembly only reads already fetched inputs and does not execute the runtime or model:

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

Both Windows variants currently fail assembly completeness before creating output because their exact licensed private Visual C++ runtime payload is not admitted. Native Windows SQLite and semantic runtime behavior have not run here; they require a Windows host. No host CRT fallback, installation or cross-platform execution is substituted for Windows qualification.

The assembled macOS MiniLM payload is 172,668,906 B excluding its receipt. The original native vector build evidence from 2026-10-24 remains historical; the current Rust cosine implementation passed the frozen 122-query retrieval gate on 2026-10-27. Package checks do not establish model memory lifecycle, oldest-platform compatibility or complete M2 acceptance; see [semantic qualification](m2-semantic-qualification.md).
