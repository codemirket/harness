# M0a private Node runtime — approved qualification subset

Prepared and approved 2026-10-24. **The user approved this exact subset for disposable M0 qualification. All three payloads were assembled; only the macOS ARM64 runtime was executed. No host installation or publication occurred.**

Approval covers packaging and using the exact official **Node.js 24.22.1 LTS executable plus its complete root LICENSE** for each target, in a disposable Mirket release/home for M0a qualification. Preserve executable and notice bytes unchanged. The exact six source/destination paths, sizes and SHA-256 values are in [m0-node.selection.json](m0-node.selection.json). This is deterministic selection from authenticated official archives, not a custom Node build. It does not authorize publication, old-installation changes or additional engines.

The compiled adapter needs Node's standard library only. Package no npm, npx, Corepack, package-manager modules, headers, installers, shell helpers or symlinks. This removes unused package-management code and its closure without modifying Node. Resolve `runtime/node/bin/node` on macOS or `runtime/node/node.exe` on Windows beneath the selected release; never fall back to PATH or the existing development Node. Preserve `runtime/node/LICENSE` in every target payload.

| Target | Official archive | Download bytes | Selected runtime + LICENSE bytes |
| --- | --- | ---: | ---: |
| macOS ARM64 | node-v24.22.1-darwin-arm64.tar.gz | 53,062,778 | 122,703,465 |
| Windows x64 | node-v24.22.1-win-x64.zip | 37,748,064 | 94,076,019 |
| Windows ARM64 | node-v24.22.1-win-arm64.zip | 33,786,263 | 82,311,283 |

These are exact file-byte totals, not filesystem allocation or measured resident memory. Assembly needs space for the verified download, selected staging files and recovery state; measure total release reservation separately. Full archives would contain 197,423,496 / 107,345,970 / 95,581,234 regular-file bytes respectively. Their 145 package manifests and 150 notice-file records per archive remain evidence, not the runtime package manifest.

## Identity and authenticity

All three downloaded archive SHA-256 values match the official [24.22.1 SHASUMS](https://nodejs.org/dist/v24.22.1/SHASUMS256.txt):

- macOS ARM64: `89c682593d76cd44233fa53b1563264e7de9cbb189cad169135c5b4fadfbc0a7`
- Windows x64: `d2046d28c2c3dcd211ac5145c0bf30e2047ef9dfb306693c8c6e2eccd8b95fa5`
- Windows ARM64: `a0c807319fede24c4a0d8f84564c82a619bd313b16b10c007c383c2609e99171`

Existing `gpgv` 2.5.24 successfully verified the detached signature over that exact checksum file. Signer fingerprint: `5BE8A3F6C8A5C01D106C0AD820B1A390B168D356`, Antoine du Hamel, 2026-10-21. The keyring came from Node's official release-keys repository at commit `481637f813e912c4aa3622d7964ab426c97b8e8d`; pinned bytes were checked against that commit. The fingerprint is also listed in the [tagged Node README](https://github.com/nodejs/node/blob/v24.22.1/README.md#release-keys). Verification used a task-local GPG home, with no key imports or changes to the user's keyring.

Node tag `v24.22.1` resolves to `ce7aaf1cee78b8cf0ce17364260535c418d3ed26`. Checksums/signatures establish selected artifact identity and the stated signer trust chain; they do not prove behavior or absence of defects.

## Native and license evidence

macOS `otool` identifies a thin ARM64 Mach-O, minimum macOS 13.5 and SDK 15.0. Its four linked libraries are macOS CoreFoundation, Security, libc++ and libSystem. `codesign --verify --strict` succeeded: Node.js Foundation, team `HX7739G8FX`. Notarization/Gatekeeper acceptance and launch under the proposed macOS 15 floor were not tested. The inspected machine was macOS 26.5.2 ARM64.

Both Windows PE files have the expected native machine type: x64 `0x8664`, ARM64 `0xaa64`. The statically parsed import tables contain only Windows OS DLLs: CRYPT32, WS2_32, USER32, dbghelp, ADVAPI32, IPHLPAPI, USERENV, SHELL32, ole32, WINMM and KERNEL32; no delay-import entries. No extra redistributable DLL was found in these tables or the official archives. Both contain Authenticode certificate tables, but Windows trust verification was unavailable and is not claimed. Imports alone do not rule out runtime library loading or establish actual target behavior.

The [exact upstream platform document](https://github.com/nodejs/node/blob/v24.22.1/BUILDING.md) lists macOS ARM64 as Tier 1 and Windows ARM64 as Tier 2. Its OS floors are compatible with the proposed Mirket floors; native runtime and child-lifecycle tests remain required on both Windows architectures. No WSL or emulation is proposed for this runtime.

Retain the entire 157,609-byte macOS LICENSE or 160,555-byte Windows LICENSE unchanged. Their text is equivalent after CRLF normalization; their exact per-platform hashes are in the selection. Node's MIT permission and the bundled third-party notices cover its upstream distribution, including V8, ICU, libuv, OpenSSL, c-ares, compression libraries and other named components. The license bundle also contains notices for build/test/omitted tooling; preserving them does not imply those files are shipped. Do not replace this bundle with “Node is MIT.” Source-read macOS config confirms bundled rather than shared third-party libraries; it also enables built-in SQLite and other Node facilities. Accepting this executable includes those bytes, even though M0's adapter does not use them; it does not adopt Node SQLite as Mirket's canonical memory owner.

## Security scope and exclusions

[Node 24.22.1](https://nodejs.org/en/blog/release/v24.22.1) is the 2026-10-21 security release, addressing listed TLS/DNS/brotli and other defects; it updates embedded OpenSSL to 3.5.9 and undici to 7.30.1. The existing local development Node 24.18.1 is a different artifact and is not promoted as the private runtime.

A registry advisory query over the full archive's package manifests returned 15 records, including five high-severity records, for npm transitive modules (`brace-expansion`, `undici` 6, `http-cache-semantics`, and other packages). **Every affected package tree is omitted from this two-file runtime selection.** Those findings are retained in [m0-node.advisories.json](m0-node.advisories.json) as archive-only evidence; they are neither hidden nor attributed to Node's different embedded dependencies. Corepack/npm bundles are not rebuilt or independently patched. The query is not a full native/embedded dependency audit, and this proposal does not assert zero vulnerabilities.

For the qualification launch, select the fixed executable and compiled adapter path, sanitize inherited Node preload/module-path configuration, and permit no package-manager execution or runtime downloads. The adapter remains dependency-free and runs in a child owned by Mirket. These conditions concern the intended path; Node is a capable general runtime and is not an OS sandbox.

## Assembly and verification after approval

1. Recheck signed archive identity and select only the two exact regular-file members per target in `m0-node.selection.json`; reject duplicates or changed bytes. Copy content unchanged, preserving platform-appropriate executable access. Do not run installer/helper/package-manager scripts.
2. Verify the assembled private file hashes and invoke that private executable for the stdio → child → saved artifact fixture. Record actual `process.versions`, executable path and architecture; test clean environment, Unicode/spaced paths, cancellation, abrupt disconnect, process ownership and release rebinding.
3. Verify native Windows signatures/launch and both architecture lifecycle paths on actual targets. Test native trust/launch on the supported macOS floor. Missing access stays an explicit gate.
4. Reassess embedded dependency advisories and applicable notices before broader tool exposure/distribution. The current approval is bounded M0 runtime qualification, not public release acceptance.

Full archive file/notice hashes are in [m0-node.inventory.json](m0-node.inventory.json); the smaller [selection](m0-node.selection.json) is the authoritative approved package content. Raw archives, source captures, GPG/codesign/load-command outputs and static inspection scripts are under `/Users/nazmiyilmaz/.codex/task-state/harness-rewrite-implementation/dependencies/node/`. They were treated as inert data during admission review. After approval, only the selected macOS executable was launched for the checks below. No npm script, installer or upstream build helper was executed, and no runtime was installed into a host.


## Disposable assembly command (only after exact subset approval)

The repository-owned `scripts/assemble-m0-runtime.py` is a qualification artifact assembler, not an installer or update path. It downloads nothing, runs no runtime/scripts and changes no host configuration. Supply an already existing empty absolute output release directory. The default manifest is this repository's `packaging/admission/m0-node.selection.json`; `--manifest` exists for synthetic tests only and must not bypass the reviewed selection. A manifest's status field is not authorization.

After approval, an example macOS command is:

```sh
mkdir /Users/nazmiyilmaz/.codex/task-state/harness-rewrite-implementation/m0-node-release
python3 /Users/nazmiyilmaz/Projects/harness-rewrite/scripts/assemble-m0-runtime.py \
  --archive-root /Users/nazmiyilmaz/.codex/task-state/harness-rewrite-implementation/dependencies/node/archives \
  --target aarch64-apple-darwin \
  --output-release /Users/nazmiyilmaz/.codex/task-state/harness-rewrite-implementation/m0-node-release
```

Windows selection targets are `x86_64-pc-windows-msvc` and `aarch64-pc-windows-msvc`; choose a separate empty absolute output directory for each. This only copies the selected runtime; launching it remains the subsequent approved qualification step. Each run verifies archive size/SHA-256 before and after reading, checks every selected member's exact size/hash, and stages both files before publication. It rejects duplicate/noncanonical/escaping member names, special members, selected links, archive symlinks, and nonempty/symlinked output paths. Unselected official npm symlinks are ignored and never materialized. It never recursively extracts an archive. A second run refuses an existing payload. Do not concurrently mutate the output directory or its ancestors; this local qualification utility does not claim an adversarial filesystem sandbox or crash-transactional installation.

Synthetic-only verification, with no official runtime execution or assembly:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover \
  -s /Users/nazmiyilmaz/Projects/harness-rewrite/tests \
  -p test_assemble_runtime.py -v
```

The 15 tests cover tar/zip exact bytes and modes, archive/member identity failures, missing/duplicate members, path traversal, selected links/hardlinks, ignored unselected links, protected existing output and repeat refusal. Actual native payload assembly/launch was not performed while preparing this utility; the later approved qualification is recorded below.


## Approved assembly and macOS qualification result

All three exact subsets were assembled under `/Users/nazmiyilmaz/.codex/task-state/harness-rewrite-implementation/releases/` on 2026-10-24. Each directory contains exactly the selected executable and complete LICENSE; no npm, npx, Corepack, modules, scripts, headers or symlinks. Independent post-assembly hashes and sizes match all six selection entries. The archive SHA-256 and member checks ran through the repository assembler before publication.

| Target directory | Executable relative to directory | Executable SHA-256 |
| --- | --- | --- |
| `m0-aarch64-apple-darwin` | `runtime/node/bin/node` | `6f32069480ec14d52b6d7b1f3c7124cfbbc46203abf5ddf5c65b127435f3f329` |
| `m0-aarch64-pc-windows-msvc` | `runtime/node/node.exe` | `8bea91dc437fe01e7d92ef0efc6c9eaf654369aaba9b2d3e9841552ea7b95a00` |
| `m0-x86_64-pc-windows-msvc` | `runtime/node/node.exe` | `d702daa815eabbff763c7d3808e235cf031cf2ca70211b56847bc72f51e240e5` |

The macOS private executable reported `v24.22.1`, `darwin`, `arm64` and its exact private `process.execPath`. Its observed `process.versions` includes OpenSSL 3.5.9, undici 7.30.1, libuv 1.52.1, V8 13.6.233.17-node.58 and ICU 78.3. Running that executable with `--test /Users/nazmiyilmaz/Projects/harness-rewrite/adapters/tests/text-adapter.test.mjs` passed all **17 tests**, with zero failures/skips. The tests spawn adapters through `process.execPath`, so their children also used the private executable. Child environments excluded inherited `NODE_*` and `DYLD_*`; the host environment was unchanged. No package manager or compilation was used in these checks.

This establishes macOS adapter behavior on the inspected macOS 26.5.2 ARM64 host. It does not qualify Windows ABI/launch, the macOS minimum supported version, native desktop activation, or the complete Rust-owned lifecycle. Both Windows executables remain unexecuted. Their successful byte assembly is not native runtime qualification.

Exact commands, all six output paths/hashes/modes, complete version output, adapter artifact hash and test output are in task-local `dependencies/node/evidence/assembly-results.json`, `private-node-qualification.json`, `private-node-version.txt`, `private-node-versions.txt`, and `private-node-adapter-tests.txt` beneath the implementation task-state root. Those records are evidence for this run, not shipped package files.
