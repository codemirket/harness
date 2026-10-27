# Installation-wide resource admission

Mirket shares two heavy-work slots and eight waiting permits across clients using
one explicitly selected private installation state directory. On macOS, a new
heavy admission also observes the resident memory of registered Mirket process
trees and the operating system's memory pressure. Admission is deferred when the
sum is at least **8 GiB (8,589,934,592 bytes)** or pressure is not normal. The
current pressure policy accepts `NORMAL` only; both `WARN` and `CRITICAL` defer.

This is a soft observation, not an 8 GiB reservation or a hard process limit.
Resident set size (RSS) can count shared pages more than once, changes during
sampling, and does not predict a new job's peak allocation. Memory may grow above
the threshold after admission. This policy does not kill active jobs to enforce
the threshold.

## Ownership and measurement

A connection or supervisor's `SessionGuard` retains a `ResourceSession` for its
lifetime. Each registration records the current PID and native process birth
identity in an immutable, privately owned file whose native lock remains held.
Supervisors register independently of their parent server, so a parent exit does
not by itself remove a surviving supervisor and its leaf from measurement.
Registration must outlive every heavy lease and its owned cleanup.

The sampler enumerates registered roots and their descendants, deduplicating
overlapping trees by PID and birth identity. It validates identities and parent
relationships around sampling, then rechecks live registrations and heavy-slot
membership. One complete bounded rescan handles a process disappearing or a tree
changing during observation. If the process tree or registration set still
changes after that rescan, observation returns busy without a snapshot or an
admission. Callers retain their existing queue/deadline behavior; an idle model
can wait for another observation, while dispatch must obtain a fresh result.
An unreadable process, unsupported native response or conflicting registered
birth identity yields unknown accounting. Neither outcome counts missing data as
zero or establishes cleanup.

A short native gate serializes registration, measurement and heavy-slot
acquisition. Registration retries gate contention for at most one second;
admission and snapshots return busy immediately. No daemon or separate polling
service owns this gate. Bounds are 256 registrations, 4,096 observed processes and
512 bytes per registration or measurement binding. Exceeding a bound fails closed.

Two fixed slot bindings associate each occupied heavy slot with an exact
PID/birth identity that has a live lock-backed registration. These bindings are
measurement membership, not authority to kill, reap, reclaim a lease or clear a
recovery marker. In particular:

- A free heavy slot with an unresolved ownership marker makes accounting unknown.
- A held slot without a matching live registration makes accounting unknown.
- If both slots are occupied with valid registration bindings, admission returns
  busy without repeatedly scanning all process trees. This returns no memory
  snapshot and makes no native process-exit or cleanup claim.
- A free slot without a marker does not need its old measurement binding; the next
  admitted owner replaces that fixed binding while holding the gate and slot.

Native lock release and PID absence never prove descendant cleanup. Existing
owned-process cleanup remains responsible for its durable marker. Known valid,
unlocked registration files may be removed under the gate; corrupt, partial,
aliased or otherwise unrecognized files are preserved and refused. Observation
can therefore maintain this private registry and is not advertised as a purely
read-only public diagnostic. Directory checks assume private installation state;
they do not establish containment against hostile ancestor replacement.

## Caller behavior

Text jobs keep their waiting permit while either heavy capacity or known memory
conditions defer admission. Their existing sixty-second total work budget
includes queueing. Unknown accounting fails explicitly instead of waiting as if
capacity were merely busy. No adapter starts while the text job is still waiting.

A cold embedding supervisor waits for occupied heavy slots, but known memory
pressure or the RSS threshold returns a pressure outcome. Warm embedding dispatch
checks a fresh snapshot. Idle monitoring refreshes on its existing nominal
250 ms interval and may retire an idle model for pressure, memory budget or
waiting work; pressure alone does not interrupt active inference. A loaded idle
model continues to own its heavy slot until cleanup is confirmed. See
[the embedding lifecycle](m2-embedding-lifecycle.md) for deadlines, cancellation,
five-minute idle expiry and process ownership.

The implementation owners are [resource admission](../src/resource_admission.rs),
[macOS observation](../src/resource_admission_macos.rs),
[session guards](../src/maintenance.rs), [native leases](../src/leases.rs),
[text workers](../src/worker.rs) and
[embedding resource monitoring](../src/embedding_session_pressure.rs).

## Native evidence and qualification

The macOS implementation uses read-only libproc queries: `PROC_PIDTBSDINFO` for
PID/birth/parent identity, `PROC_PIDTASKINFO` for resident bytes, and
`proc_listpids` with the SDK's `PROC_PPID_ONLY = 6` selector for children. The
selector is defined in Apple's `sys/proc_info.h`. Apple's
[libproc wrapper](https://github.com/apple-oss-distributions/xnu/blob/main/libsyscall/wrappers/libproc/libproc.c)
can convert a failed `proc_listpids` call into zero with `errno` set. The sampler
clears thread-local `errno` first and accepts zero as an empty result only when
`errno` remains zero. Exact-size process-info responses are required. Pressure
comes from the read-only `kern.memorystatus_vm_pressure_level` sysctl; unexpected
values or unavailable queries fail explicitly.

The retained `resource-native-final-2.log` records a macOS 26.5.2 ARM64 run with
the actual private Node adapter enabled: **310 library tests and one deterministic
retrieval fixture passed**, with three manual tests ignored across those suites.
The run includes all ten resource-admission tests and nine maintenance tests.
Strict Clippy and the build also passed. The frozen main executable SHA-256 was
`2ddba4ca99ce4a25e5444fd639f5b524244784e7d0df197184e8270d813d3404`.

Resource tests cover overlapping-root deduplication, injected threshold and
pressure decisions, PID reuse and reparenting, disappearing versus unreadable
processes, lock-backed slot membership, unresolved-marker preservation, state
aliases and bounds, completed-owner marker handoff, bounded transient churn and
an actual owned native child RSS sample. Injected 8 GiB
crossings do not establish a measured 8 GiB workload peak or a hard memory cap.

The same executable passed the actual configured memory MCP journey and manual
CLI transfer/restore in `resource-configured-memory-final.log`, under OS network
denial with `ORT_DISABLE_TELEMETRY=1`. The fixture exercised actual indexing and
semantic recall, explicit lexical fallback for unavailable payloads, canonical
save/backup preservation, forgetting exclusions, transfer admission without
activation, explicit maintenance restore and derived-index rebuilding. Completed
runtime jobs left no scratch directories.

These checks establish the exercised memory paths with resource admission; they
do not establish retrieval-quality targets or a general workload memory ceiling.
The same binary also passed the text MCP journey (`resource-text-final.log`)
and disposable two-release activation (`resource-activation-final.log`). All
fixture processes stopped after verification.
Two-client aggregate memory behavior, performance under sustained process churn,
the oldest supported macOS release and Windows native accounting remain
unqualified. Unsupported platforms return an explicit accounting failure before
resource-registration effects; there is no path-hash or PID-only fallback.
