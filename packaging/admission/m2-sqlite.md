# Private SQLite admission

The active Rust selection is in the root [manifest](../../Cargo.toml) and
[lock](../../Cargo.lock). Mirket uses `rusqlite` with `modern_sqlite` and `backup`,
linked to the reviewed private SQLite **3.54.1** source. It does not use the
operating-system SQLite or rusqlite's bundled C snapshot.

The [native manifest](m2-sqlite.native-build.json) pins the exact archive,
selected members, source ID, hashes, compile defines and target requirements.
[Source receipts](m2-sqlite.sources.json), [inventory](m2-sqlite.inventory.json),
[qualification](m2-sqlite.qualification.json) and
[dated advisory snapshot](m2-sqlite.advisories.json) support the admission.
The original batch review is available at its
[immutable revision](https://github.com/codemirket/harness/blob/b3068a181a674f2aff751de4ff254f8120e7f96e/packaging/admission/m2-sqlite.md);
its preparation sequence is complete, not pending adoption.

Follow [native build instructions](../../docs/native-build.md) with an already
acquired archive and explicit compiler/archiver. The pre-build checker and
`build.rs` validate the receipt, target, original header/library and private paths.
No system-library lookup or silent recovery of mismatched bytes is permitted.

After preparing a changed native input, verify version/source ID, FTS5 behavior,
extension-loading denial, transactions and backups, plus actual application
memory tests. Hash agreement alone does not qualify SQL behavior or platform
support. macOS 26.5.2 ARM64 is the first-release target; Windows remains unqualified.
