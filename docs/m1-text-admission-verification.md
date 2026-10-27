# Text admission qualification

Verified on macOS ARM64 on 2026-10-24 against binary SHA-256
`cd756bd7a63d5740b0931f46de1a029ea18625fb8a4af3015c33dfa90c32ed8e`.
This evidence predates the subsequent receipt-retention integration.

The subsequent native-lock integration passed 234 native-enabled library tests,
one deterministic retrieval fixture, strict all-target Clippy, real text MCP and
two-release maintenance activation against binary
`459d66f1d1f5e2a981a85ddba3290dd3b06e8f84fbda2f20ffc52070aface8b3`.
The model was not configured for that Cargo run. Its separate configured memory
MCP journey passed with OS network denial and telemetry disabled before native
initialization. This baseline predates later unused-intent retirement changes.

The explicit queued-deadline qualification was repeated after native FileLock
ownership and prepared-intent binding changes, against frozen main binary
`f148907a760305ee8620c1d51b3ddc89812d2bb07410216e0f11eb1b1a097404`.
The build-input guard passed, and the exact ignored library test passed in
60.02 seconds on macOS ARM64. Two held heavy slots kept the supervisor queued;
no Running frame, leaf sentinel, stage output or unresolved marker appeared.
Cleanup was confirmed and all eight waiting permits were available afterward.
The test used disposable state and explicit private Node; its sentinel adapter
was deliberately never launched. No main binary rebuild or model run occurred.

Lock owners explicitly unlock before descriptor close. A deterministic Unix
duplicate-descriptor control and an independent fork probe reproduced how an
inherited open file description can otherwise retain a parent's lock until a
child exits. Regressions check both prompt release and preservation of unrelated
owners' exclusion. The original failed parallel test's exact child was not
traced; these checks establish the reproduced mechanism, not that attribution.
SQLite catalog connections close before their scope guard releases exclusion.

Text operations use the existing start/status/cancel flow. Start reports
`queued`; one validated supervisor progress frame records `running` and
`queue_wait_ms`. Two installation-wide native heavy slots and eight native
waiting permits bound admission. Waiting owns no leaf or unresolved ownership
marker. Promotion releases the waiting permit before leaf launch. The total
text work budget is 60 seconds from the first admission attempt, including queue time; owned
cleanup follows deadline cancellation. The parent watchdog remains 70 seconds.
Embedding requests continue to use immediate admission.

The private Node runtime and compiled text adapter were supplied explicitly.
No embedding model was loaded for these checks.

| Check | Result |
| --- | --- |
| Worker suite with actual binary and Node | 11 passed; includes nine competing supervisors yielding eight waiters and one busy response, seven queued cancellations, one promotion, queued and running parent death, unavailable-package recovery, and protocol bounds |
| Progress callback failure | Passed; terminal response drained, supervisor reaped, both heavy slots available |
| Actual queued deadline | Passed in 60.02 seconds; no Running event, leaf, output, or unresolved marker; all eight waiting permits available |
| Adapter process suite with actual Node | 7 passed; includes cancellation, execution deadline, and expired shared deadline refusing launch |
| Engine and state library tests | 18 and 4 passed before receipt-retention integration |
| Strict library/test Clippy | Passed before the final test-only qualification cases were added |

The first saturation test incorrectly acquired all free waiting permits to
observe capacity, interfering with arriving supervisors. It was replaced with
nine real contenders: the excess worker's busy response establishes that the
other eight hold waiting permits without an observer reserving capacity.

Re-run the native worker tests with explicit `MIRKET_TEST_BIN`,
`MIRKET_TEST_NODE`, and `MIRKET_TEST_ADAPTER`. The slow deadline qualification
is intentionally ignored by the ordinary suite and must be selected explicitly:

```sh
python3 scripts/check-build-inputs.py
CARGO_INCREMENTAL=0 cargo test --locked --offline --lib worker::tests
python3 scripts/check-build-inputs.py
CARGO_INCREMENTAL=0 cargo test --locked --offline --lib \
  worker::tests::actual_text_queue_deadline_returns_without_leaf_or_marker \
  -- --ignored --nocapture
```

This is a bounded queue, without a FIFO guarantee. Per-connection registries
remain additional protective limits. These checks do not qualify Windows,
hostile state-directory replacement, process-tree memory pressure accounting,
or desktop client activation.
