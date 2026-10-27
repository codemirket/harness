# M2 canonical SQLite — reviewed admission batch

Prepared 2026-10-24. **The user authorized further necessary work within the approved rewrite, including reviewed dependencies and disposable qualification. This exact batch is recorded under that authorization; production Cargo files remain unchanged by this worker.**

The selected batch is `rusqlite = 0.41.0` with only `backup` and `modern_sqlite`, its exact eight-package Cargo addition, and the **unchanged official SQLite 3.54.1 amalgamation**, compiled into a private static library using the explicit native build contract below. This implements the selected SQLite/FTS5 direction without a database service, custom database fork, runtime download or dynamic extension loading. It is a concrete alternative to the crate's outdated bundled C snapshot, reviewed before disposable build and requiring root integration before application adoption.

This batch supports canonical facts, source/provenance links, revision transactions, supersession and consistent backup mechanics. Mirket still owns their semantics and failure handling. It is **not complete memory**: vectors, packaged embeddings, multilingual semantic qualification and the rest of M2 remain planned, separately admitted work. Existing Node's embedded SQLite is unrelated to this Rust owner.

## Why this exact source route

| Checked route | Actual native SQLite | Decision |
| --- | --- | --- |
| Latest published rusqlite 0.41.0 / libsqlite3-sys 0.39.0 with `bundled` | 3.54.0 | Reject this bundled snapshot for adoption: known fixes are available. |
| Previous published rusqlite 0.40.2 / libsqlite3-sys 0.38.2 | 3.53.2 | Not a clean patched fallback; it lacks 3.53.3/3.53.4 fixes and later binding corrections. |
| Latest upstream amalgamation history inspected | 3.54.0 commit `1d5d0ecae8b8372c2779caf5ab98847155dba153` | No existing upstream 3.54.1 update found in the bounded current history check. |
| **Proposed: current binding + official static library** | **3.54.1** | Documented explicit-library route; no registry source edits, fork or system-library discovery. |

SQLite 3.54.1 fixes a 32-bit-only heap overflow (our targets are 64-bit) and other defects, including FTS5 separator handling, schema reset, query results and Unix mmap page leakage. The latter cannot all be dismissed by architecture. This proposal selects the fixed source rather than accepting that gap. [SQLite release](https://sqlite.org/releaselog/3_54_1.html), [patch timeline](https://sqlite.org/src/timeline?from=version-3.54.0&to=version-3.54.1&y=ci). The earlier binding release and native version were checked from immutable registry archives, not inferred from release dates. [rusqlite 0.40.2](https://github.com/rusqlite/rusqlite/releases/tag/v0.40.2), [rusqlite 0.41.0](https://github.com/rusqlite/rusqlite/releases/tag/v0.41.0).

## Cargo admission and preserved pins

```toml
rusqlite = { version = "=0.41.0", default-features = false, features = ["modern_sqlite", "backup"] }
```

Use the supplied full [manifest](m2-sqlite.Cargo.toml.proposed) and [lock](m2-sqlite.Cargo.lock.proposed). All **96 existing external M0 versions/checksums are unchanged**, with eight additions and no removals. Total locked external crates: **104**. Resolved target closures including build dependencies: macOS ARM64 **82**, Windows ARM64 **79**, Windows x64 **79**. Target resolution is not compilation.

| Added crate | Pin | License | Purpose |
| --- | --- | --- | --- |
| rusqlite | 0.41.0 | MIT | Rust connection/statement/transaction/backup API |
| libsqlite3-sys | 0.39.0 | MIT; selected SQLite source public domain | SQLite FFI and static link metadata |
| bitflags | 2.13.2 | MIT OR Apache-2.0 | Typed SQLite flags |
| fallible-iterator | 0.3.0 | MIT OR Apache-2.0 | Fallible row iteration |
| fallible-streaming-iterator | 0.1.9 | MIT OR Apache-2.0 | Streaming iteration |
| smallvec | 1.16.4 | MIT OR Apache-2.0 | Small internal collections |
| pkg-config | 0.3.34 | MIT OR Apache-2.0 | Inherited build dependency; discovery disabled |
| vcpkg | 0.2.15 | MIT OR Apache-2.0 | Inherited build dependency; discovery branch bypassed |

[Full closure inventory](m2-sqlite.inventory.json) records every crate archive SHA-256/bytes, license/notice file hashes, feature set, target reachability, build scripts, proc macros and edges. Rust **1.99.1** remains pinned; rusqlite declares MSRV 1.89.0 and libsqlite3-sys 1.88.0. No optional cache/hashlink, wasm implementation, SQLCipher/OpenSSL, time/chrono, CSV, arbitrary user functions, hooks, serialization, loadable extension or build-time bindgen is selected. `modern_sqlite` enables pre-generated bindings, not a native library search or a C build. FTS5 is compiled into the selected C source using its standard define; it needs no separate Rust feature or loadable module. [FTS5 build documentation](https://sqlite.org/fts5.html#building_fts5_as_part_of_sqlite).

## Native source and build contract

The [native manifest](m2-sqlite.native-build.json) pins source selection, build defines, target flags and child build environment. Archive reading occurred as inert data before authorization; the later authorized macOS build is recorded below.

- Archive: [sqlite-amalgamation-3540100.zip](https://sqlite.org/2026/sqlite-amalgamation-3540100.zip), **2,990,966 bytes**, SHA-256 `0a9c8e34352dad46006aa82836d5383bff9dc6bc9ce5a60553c05aa30e6ad1f9`.
- `sqlite3.c`: **9,582,521 bytes**, SHA-256 `0c657666d819f261cc4007526ad57e1669de9949df7a8ea847aa284a2bf0ee52`.
- `sqlite3.h`: **695,729 bytes**, SHA-256 `d6c84557261893439b341d2e6c35187ae3ea24fbfd843ad94616f72d89242b8c`.
- Source ID: `2026-10-16 14:45:20 555e07ef33f69a0957d24d8e555b702464a131d6f1607f4c3c1e4551e8e6ca76`.

The C source SHA3-256 is `e2b869d3003b33c942b7dd7063604fbc53553e1b66609570df39b9b7b558ee81`, matching SQLite's official release record. ZIP SHA-256 is a locally pinned download identity; no detached signature was supplied or verified. Select only `sqlite3.c` and `sqlite3.h` unchanged; omit `shell.c` and `sqlite3ext.h`. The source's public-domain dedication is retained, alongside MIT/Apache notices for Rust dependencies. [SQLite copyright](https://sqlite.org/copyright.html). Archive hashes do not establish defect freedom.

Build the static library **before Cargo**, using explicit installed compiler/archiver paths, no configure/build/download scripts. Required defines are `SQLITE_THREADSAFE=1`, `SQLITE_DEFAULT_FOREIGN_KEYS=1`, `SQLITE_ENABLE_API_ARMOR`, `SQLITE_ENABLE_FTS5`, and `SQLITE_OMIT_LOAD_EXTENSION`. Do not apply unrelated optimization/feature toggles. No arbitrary SQL or path-based extension API becomes a public tool.

On macOS ARM64, the proposed compiler arguments are `-O2 -fPIC -arch arm64 -mmacosx-version-min=15.0`, the five `-D` defines, `-c /ABS/sqlite3.c -o /ABS/sqlite3.o`; archive with `ar rcs /ABS/lib/libsqlite3.a /ABS/sqlite3.o`. Record the actual Apple Clang/SDK/archiver version and final archive hash for each native build. Minimum-OS execution is a separate gate.

On each native Windows target, use that target's explicit MSVC compiler with `/nologo /O2 /MT /c`, the five `/D` defines and explicit `/Fo` output; use matching `lib.exe /OUT:/ABS/sqlite3.lib /ABS/sqlite3.obj`. Pair `/MT` with Rust `-C target-feature=+crt-static` for the Windows qualification build; inspect final PE imports to verify the intended runtime closure. This static CRT choice is part of the proposed build batch, not a change already made. Native Windows tooling/ABI execution is unavailable here and must not be inferred from macOS.

Pass these settings only to the child Cargo build, after checking the target archive and header against recorded native build receipts:

```text
SQLITE3_LIB_DIR=/ABS/VERIFIED/TARGET/lib
SQLITE3_INCLUDE_DIR=/ABS/VERIFIED/sqlite-amalgamation-3540100
SQLITE3_STATIC=1
SQLITE3_NO_PKG_CONFIG=1
LIBSQLITE3_SYS_USE_PKG_CONFIG=0
```

The exact upstream [README](https://github.com/rusqlite/rusqlite/blob/a60e76b5e86ee60442213efe79758a6c2ca8cf3d/README.md) documents explicit library/header paths and static linking. Inspected libsqlite3-sys `build.rs` follows the explicit-directory branch, copies pre-generated bindings, and emits static link metadata. It tries pkg-config first even with an explicit directory; `SQLITE3_NO_PKG_CONFIG=1` makes that probe return before command execution. That branch returns before vcpkg. Reject missing settings, wrong target/archive hashes, competing dynamic libraries and unreviewed flags before build; do not permit a fallback to host SQLite. The pre-generated 3.54.0 bindings are used against the patch-compatible 3.54.1 native ABI; use runtime version/source checks instead of header-derived version constants.

This route trades the convenience of Cargo's outdated bundled snapshot for one explicit native compile step and compiler artifact provenance. It does not build or ship SQLite's CLI or install a system database. The inherited `cc`/`find-msvc-tools`/`shlex` M0 lock entries remain unreachable on the three selected targets. New reachable build-script owner is libsqlite3-sys only; rusqlite has no build script. Existing M0 build-script obligations, including the rmcp hook guard, still apply. Sources and script hashes are in [source evidence](m2-sqlite.sources.json).

## Advisory scope

The RustSec archive was rechecked at commit `21d909e4c8bd4e14c7aea58bf980c881a0053ab1`. There are **34 package-name matches** across the full lock, all outside affected version ranges; [advisory evidence](m2-sqlite.advisories.json) records each range and source hash. This includes the old libsqlite3-sys SQLite printf issue, rusqlite lifetime defects and smallvec issues. It is not a full native audit or a claim of zero vulnerabilities. The released 0.41.0 changelog also contains recent callback safety fixes; optional callbacks are not selected, and the current binding is retained. The crate archive's unused SQLite 3.54.0 source is inventoried but must not be compiled or linked by this proposal.

## API and qualification checks

The source-checked APIs are `Connection::open`, `transaction_with_behavior(TransactionBehavior::Immediate)`, parameterized `execute`/`query_row`, `Transaction::commit`, `Connection::set_db_config`, and `backup::Backup::{new,step}`. No async SQLite wrapper or connection pool is needed for the first slice. Put bounded database work on the existing Rust owner's appropriate blocking boundary; do not hold installation-global locks through long backup loops.

Illustrative qualification calls, not a final memory schema or compiled example:

```rust
use rusqlite::{Connection, TransactionBehavior, backup::{Backup, StepResult}};
let mut db = Connection::open_in_memory()?;
let version: String = db.query_row("SELECT sqlite_version()", [], |r| r.get(0))?;
assert_eq!(version, "3.54.1");
let fts: i64 = db.query_row(
    "SELECT sqlite_compileoption_used('ENABLE_FTS5')", [], |r| r.get(0))?;
assert_eq!(fts, 1);
let tx = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
tx.execute_batch("CREATE VIRTUAL TABLE terms USING fts5(body)")?;
tx.execute("INSERT INTO terms(body) VALUES (?1)", ["source-backed fact"])?;
tx.commit()?;
let mut destination = Connection::open_in_memory()?;
let backup = Backup::new(&db, &mut destination)?;
let step = backup.step(64)?;
// Handle Done/More/Busy/Locked under an explicit deadline and byte reservation.
```

Use bounded `step` calls, not unrestricted `run_to_completion`: its source loops on busy/locked without a deadline. Final snapshot eligibility requires successful completion, integrity checks and current forgetting exclusions; SQLite's consistent backup API alone does not implement that policy. [Backup API](https://sqlite.org/backup.html).

Require native version/source identity, FTS5/OMIT_LOAD_EXTENSION compile-option checks, a failing extension-load attempt, Unicode/FTS separator fixtures, transaction rollback and stale-revision races, current-versus-historical scope filtering, bounded backup contention and consistent restore tests. Exercise source/provenance/supersession in one transaction; logical forget must also update the non-rollback content-free exclusion authority before acknowledgment, and physical/FTS/WAL/backup cleanup must be tracked separately. Verify no resurrection after old backup/import and concurrent backup completion. FTS matches cannot replace canonical scope/currentness checks or exact code/path lookup. These obligations come from the approved memory contracts, not new architecture.

Initial admission preparation performed no compilation or adoption. Subsequent broad authorization covers the disposable native qualification recorded below. No user database, old installation, client configuration, provider/model request or publication is involved. Root owns application dependency adoption and integration; full M2 acceptance still requires its implementation tests.


## Authorized disposable macOS result

The exact batch compiled and ran successfully on macOS 26.5.2 ARM64 after the user's broad implementation authorization. Apple Clang **21.0.0 (clang-2100.1.1.101)** compiled the unchanged 3.54.1 amalgamation with the recorded defines and macOS 15 deployment target. The static library is **1,884,880 bytes**, SHA-256 `d38346b1bbb5d9ea2ac2540228e5ae94080e6b5942766258ef0b74827e974e75`, at `/Users/nazmiyilmaz/.codex/task-state/harness-rewrite-implementation/dependencies/sqlite/native-aarch64-apple-darwin/libsqlite3.a`.

The existing rmcp build-input guard passed before compiling. A repository-owned disposable Rust probe used the exact proposed lock with `cargo +1.99.1 run --locked --offline`. It passed runtime SQLite **3.54.1/source ID** identity, FTS5 and omitted-loader compile-option assertions, actual UTF-8 queries including **U+FFFE/U+FFFF separators**, rejected SQL extension loading, transaction rollback, stale-revision rejection across two connections, a bounded stepped **400-record backup**, snapshot independence after source mutation and reopened integrity/content checks. The resulting executable's `otool -L` list contains OS libraries and no SQLite dylib, supporting the observed private static linkage. No production application files or databases were changed.

[Qualification evidence](m2-sqlite.qualification.json) records exact commands, build environment, source/binary hashes, compiler version and linked libraries. Raw stdout/stderr and the owned probe remain in task-state `dependencies/sqlite/`. Native Windows and macOS 15 execution remain untested; this bounded probe does not establish M2 forgetting/restore interleavings, final schema correctness, complete hybrid recall or desktop acceptance. Root can now adopt the recorded Cargo delta and explicit native build inputs under existing authorization, then run integration checks.
