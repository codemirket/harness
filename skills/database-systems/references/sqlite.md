# SQLite decisions

Confirm the library actually linked by the application, not only the command-line version. Check driver transaction defaults and relevant compile options. The official references below were reviewed on 2026-10-06.

## Connections and concurrency

Enable and verify `PRAGMA foreign_keys = ON` on each applicable connection before starting transactions. Changing it inside a transaction does not take effect. Do not rely on a build's default. Exercise foreign-key failures through the same connection lifecycle used in production. [Foreign-key support](https://www.sqlite.org/foreignkeys.html)

SQLite allows one writer at a time. WAL permits readers alongside a writer; it does not allow multiple simultaneous writers. A read transaction whose snapshot is stale may fail to upgrade to a write transaction with `SQLITE_BUSY_SNAPSHOT`; restart the transaction and its reads. For a short operation known to need a write, `BEGIN IMMEDIATE` can acquire the write transaction up front, but may itself encounter contention. A busy timeout is not a cure for an invalid snapshot. [Isolation](https://www.sqlite.org/isolation.html)

Keep write transactions short and use bounded retries consistent with the driver's error handling. Perform slow computation before acquiring the write transaction when doing so preserves correctness. Test with multiple real connections, rather than assuming a single connection fixture reproduces locking.

## Files, WAL, and recovery

Standard WAL operation requires participating processes on the same host; do not treat a shared network filesystem as equivalent to local storage. Long-running readers can prevent checkpoint progress and grow the WAL. Include checkpoint behavior and disk headroom in operations diagnostics. A WAL file is part of the live database state; deleting or separating it from an active database can lose committed data. [WAL documentation](https://www.sqlite.org/wal.html)

Use a supported consistent backup mechanism, such as the online backup API, instead of copying only the main file while writes may be active. Restore into a separate location and verify application queries and constraints before calling the backup usable. [Backup API](https://www.sqlite.org/backup.html)

## Schema and query behavior

Use parameter binding and inspect `EXPLAIN QUERY PLAN` for the actual query. Treat its output as diagnostic information, not a stable application API. Decide deliberately how dates, monetary precision, booleans, and nulls are represented; SQLite's typing differs from a server database. Verify feature availability before introducing STRICT tables, generated columns, or newer ALTER TABLE forms. [Query plan diagnostics](https://www.sqlite.org/eqp.html)

For a schema change requiring a table rebuild, follow the documented procedure for the deployed version and preserve indexes, triggers, constraints, and foreign-key relationships. Test interruption and rollback in a disposable copy. Do not edit the schema catalog directly as a shortcut or disable integrity checks merely to get a migration to pass. [Schema alteration procedures](https://www.sqlite.org/lang_altertable.html)
