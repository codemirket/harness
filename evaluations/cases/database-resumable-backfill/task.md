# Expand and backfill a customer lookup key safely

Work only inside this scratch `workspace/`. Use Python standard library and a local
SQLite database made from `schema.sql`. No network, dependencies, real customer
records or production operations are involved. Keep `schema.sql` unchanged.

Implement `migration.py` with two functions accepting a `sqlite3.Connection` that
has no open transaction on entry:

- `expand(conn) -> None`: idempotently add nullable `email_key TEXT` to `customers`
  and install compatibility behavior. Existing rows stay pending (null key).
- `backfill_batch(conn, batch_size=100) -> int`: select at most `batch_size` pending
  rows in ascending `id`, set their key, commit the complete batch, and return the
  count processed. Return 0 when nothing is pending. Reject non-positive, boolean
  or non-integer batch sizes with `ValueError` before writes. A SQLite failure must
  roll back the whole batch and propagate. Every successful return is durable;
  callers may terminate and restart with a new process/connection between batches.

The key is ASCII `lower(trim(email))`, removing space characters at both ends only.
All fixture and test emails are ASCII text. Repeated keys are allowed: do not merge
customers or impose uniqueness. Preserve IDs, original email spelling/whitespace,
display names, notes, existing indexes/triggers and unrelated tables.

Compatibility after expansion is part of the task, including during backfill:
legacy writers still insert/update only `email` and other old fields; their changes
must receive the correct key. New writers may supply both email and its correct
key. An explicitly supplied incorrect non-null key on INSERT, or an explicitly
updated incorrect/null key, must be rejected with a SQLite exception and no row
change. Legacy UPDATE of email alone must remain valid even when its old key no
longer matches the new email. Do not require an application-defined SQL function or
special connection setup: old/new writers use ordinary independent connections.
Keep the old `email` column throughout this exercise; no destructive contract step.

Write `migration.md` with a reproducible local invocation, the invariants and
failure/restart behavior you checked, deployment ordering, and what this exercise
does not establish for a real production database. Include a focused local check
if useful. You may run `python3 -I ../verify.py .`. Automated checks execute your
module, database operations and a fresh-process restart; review is still required.
