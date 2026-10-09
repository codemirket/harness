# Worked recipe: debit account credit

Example contract: an authenticated tenant may debit a positive integer number of
minor currency units from its own account. Balance cannot become negative.
A caller supplies an idempotency key for safe replay. This is a design exercise,
not a payment processor or a universal persistence pattern.

1. Parse bounded input; reject booleans masquerading as integers, zero/negative
   amounts, unknown fields and oversized keys. Establish tenant/account access
   from the actual authorization policy, before returning any stored receipt.
2. Bind the key to tenant, account, amount and contract version. A replay with a
   different command is a conflict. Decide whether a failed command reserves its
   key and document that choice; successful-command-only retention is one option.
3. In the transaction, find a matching durable receipt first. Otherwise perform
   a conditional debit and insert its receipt atomically. Return the stored
   original result on replay, even if the account has changed since then.
4. Commit before reporting durable success. If the response disappears after
   commit, the next identical request returns the receipt without a second debit.
   Expiry of retained keys limits the replay guarantee; choose that window from
   callers' retry/delivery behavior and document it.
5. Map validation, authorization, conflict and temporary availability failures
   through the project's stable error contract. Preserve unknown outcomes across
   process or transport boundaries. Do not turn every exception into a retry.

Expected examples with opening balance 1,000:

| Command | Expected visible result | Durable invariant |
| --- | --- | --- |
| Debit 300, key A | Original receipt, balance 700 | One receipt, one debit |
| Replay A, 300 | Same original receipt | Balance stays 700 |
| Reuse A, 301 | Conflict | No additional write |
| Debit 800, key B | Insufficient balance | Balance stays 700 |
| Other tenant accesses account | Policy-defined denial | No receipt or balance leak |
| Crash before receipt/commit | Failure | Neither debit nor receipt survives |
| Two simultaneous debits of 700 | At most one success | Balance never negative |

Test the transaction with real isolated connections, not a mock that serializes
calls automatically. Test the transport's identity mapping separately; passing a
`tenant` argument in a unit test does not verify authentication. Inject failure
between writes and replay after closing/reopening the database. Assert account
balance plus sum of successful unique debits equals opening balance.

For SQLite, explicit `BEGIN IMMEDIATE` can obtain write ownership before this
read-modify-write sequence; another writer can make it return `SQLITE_BUSY`.
Use a bounded wait/retry policy appropriate to the request deadline. This choice
is SQLite-specific; it is not a PostgreSQL locking recipe. See
[SQLite transactions](https://www.sqlite.org/lang_transaction.html) and the
installed `database-systems` SQLite reference.
