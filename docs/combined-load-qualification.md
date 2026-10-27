# Combined-load qualification: two-slot contention and recovery

The revised 2026-10-26 macOS ARM64 journey **passed** semantic retrieval during
large SVG rendering, an explicit availability fallback when three consumers
contended for two heavy-job slots, and a new semantic recall after contention.
It does not establish uninterrupted semantic availability for three simultaneous
heavy consumers. The failed Normal-pressure predecessor and earlier
Warning-pressure admission refusal remain below as separate evidence.

[`combined-load-smoke.mjs`](../tests/combined-load-smoke.mjs) creates three MCP
connections sharing one disposable development state and project. Its intended
journey indexes a sourced Turkish fact, recalls it through English semantic
queries while a 2048 × 2048 SVG raster job runs, then adds selected-source
TypeScript navigation under the same two-slot limit. It samples owned process trees, the actual macOS pressure
value, registration/slot bindings, ownership markers and scratch names. Successful
later stages require matching saved PNG revisions, a correct definition location,
explicit retirement classification, independent semantic recovery after
contention, external-resource refusal, EOF cleanup and available native slot locks.

## Passing bounded-contention run

The saved result is
`~/.codex/task-state/harness-rewrite-implementation/combined-load-successor-20261026/combined-load-xSPtNj/result.json`,
3,955,916 bytes, SHA-256
`df3b9fa1c8f0eba88582900e1715be8f0a8d99ed795252a0dfcb319425719a24`.
It used the same development core
`sha256:981aec966b05f97fc69e7bf27631aa72635fa4e3ad271cd243716ddc2f261728`,
private Node, verified semantic package and 6,024-file Chromium-only Design
selection as the failed predecessor. The corrected runner SHA-256 was
`3edf5c9ba79993a06e0d4cb015345b8f2ba54d5964a0d007fa482ccad1d0d996`.
No product admission limit or runtime implementation was changed for this run.

| Observation | Measured result |
| --- | --- |
| Platform and duration | macOS 26.5.2 ARM64, 16 GiB RAM; 31.233 s elapsed |
| Native pressure and process samples | All 566 samples Normal=1; maximum summed observed server-tree RSS 1,289,504 KiB (shared pages can be counted more than once) |
| First SVG render plus memory | 24 correct bilingual semantic recalls; 141 samples observed actual overlap; saved 2048 × 2048 PNG |
| Three-consumer stage | One semantic success, then one exact typed retirement fallback; Design render succeeded and JS/TS navigation returned the definition at bytes 6–27 |
| Recovery | A new independent semantic recall found the same sourced fact after contention cleared; no interrupted request was replayed |
| Artifacts | Both saved PNGs were 2,987,521 bytes, each SHA-256 `34b356933b854f36e182b059cc4af83cc7266453f5b34d71bdd6b987f0bf71a5`, matching their job receipts; the first was visually inspected for the expected composition and Turkish text |
| Refusal and cleanup | External SVG resource refused without an output; three MCP servers exited zero; no owned processes, jobs or unresolved markers remained; all ten admission locks were available |

The fallback was reported explicitly as semantic unavailable with lexical-only
coverage and no hits. The public warning does not prove the supervisor's exact
stop reason. Normal-pressure samples, slot handoff and later recovery support
the bounded-contention interpretation; they do not turn the fallback into a
semantic hit. This is one disposable development run, not an installed release,
general latency or memory benchmark, browser/video concurrency check, Windows
qualification or Codex Desktop GUI activation.

## Normal-pressure predecessor and corrected test contract

The successor input core SHA-256 was
`981aec966b05f97fc69e7bf27631aa72635fa4e3ad271cd243716ddc2f261728`,
on the same macOS **26.5.2 ARM64 / 16 GiB** reference machine. All **304**
independent pressure samples reported **1/Normal**. The source runner used a
fresh disposable project/state and explicit private runtime paths; no admission
threshold or production implementation changed for this attempt.

| Observation | Measured result |
| --- | --- |
| Canonical save and semantic index | `committed`, revision 1, `indexed`; no receipt warning |
| Baseline bilingual recall | Correct record; zero lexical/literal matches |
| First render | `succeeded`, confirmed effect, complete scratch cleanup |
| Saved first PNG | 2,987,521 bytes; 2048 × 2048; receipt revision matched saved bytes |
| First render's semantic queries | 24 successful recalls of the Turkish source through the English query |
| Independently sampled first-stage overlap | 141 model/raster samples, including 99 with an in-flight semantic request |
| Second stage | One successful recall, then explicit semantic-unavailable response; navigation returned the correct definition at bytes 6–27 |
| Highest independently sampled tree RSS sum | 1,222,016 KiB; shared pages may be counted more than once |
| Maximum observed heavy markers | 2 |
| EOF after the assertion | All three servers exited zero; no fixture processes, registrations, jobs or unresolved heavy markers; all ten admission locks available |
| Full journey | Failed at approximately 18.0 s; second PNG, post-contention recall and external-resource refusal were not verified |

The returned search coverage included `semantic_available: false`,
`lexical_only: true`, no hits, and this warning:

```text
embedding inference may have started; request was not replayed: embedding session stopped during inference
```

The [embedding lifecycle](m2-embedding-lifecycle.md) intentionally retains one
of the installation's two heavy slots for a warm model and retires an idle model
when another owner waits. The supervisor checks waiting demand between native
inferences, before reading the next parent frame. A new search can therefore race
that retirement; the parent conservatively reports possible dispatch and does not
replay it. The observed slot-0 handoff from embedding to navigation under Normal
pressure is consistent with this policy. The public warning does not include the
supervisor's stop reason, so these samples do not independently prove that reason
or that native inference had begun.

Requiring every search to remain semantic while a resident model, raster job and
navigation compete for two slots exceeded that contract. The revised runner
keeps the first render's **24 successful semantic recalls** mandatory. During
the three-consumer stage it retains each actual search response and permits only
exact session-stop warnings with explicit lexical fallback. Other warnings fail,
and the round must have independently sampled Normal pressure throughout. These
generic stop messages do not establish why the supervisor stopped or exclude a
different cause between samples. A fallback is recorded as a fallback, never a
successful semantic recall. It awaits both other
jobs to terminal results, verifies the saved PNG and definition, then requires a
new independent bilingual semantic recall after contention clears. It does not
replay the interrupted call. A later successful run of that contract would
qualify bounded contention and recovery, not uninterrupted three-way semantic
availability.

The failed predecessor remains unchanged in
`~/.codex/task-state/harness-rewrite-implementation/combined-load-successor-20261026/`:

- `combined-load-JTc37r/result.json`, SHA-256
  `d3b6b9ec698a18ecf1165ef82d68214d677573055bc18bf8e6b3d517a38582e0`,
  retains the indexed save, semantic coverage, sampled ownership and cleanup
  evidence.
- `combined-load-JTc37r/render-1.png` preserves the saved first artifact;
  its revision is
  `sha256:34b356933b854f36e182b059cc4af83cc7266453f5b34d71bdd6b987f0bf71a5`.
- `predecessor-runner.mjs`, SHA-256
  `35456fd2ff016c8962078c60d3e6c979c0b135242397b6c5c9842609ee556f07`,
  matches the runner identity in that report. `predecessor-review.json` records
  derived observations without changing the original report.

## Earlier Warning-pressure attempt

The first attempt **did not establish simultaneous semantic retrieval, design
rendering or navigation**. The real macOS memory-pressure gate refused the first
model load. This is a measured admission deferral, not a combined-load pass.

### Inputs and observed boundary

The run used macOS **26.5.2**, ARM64 and **16 GiB physical RAM**. The build guard
passed, followed by `cargo build --locked --offline --bin mirket` in **11.81 s**.
The frozen current core SHA-256 was
`fd0e2c8ceb53be54fe386a858c0801c35163adb0484ba6f767a644eaf8385ecb`.
The probe used that hash as its development release label; it is not a packaged
distribution release ID.

All runtime paths were explicit and local. Before launch, the runner matched
all **6,128 admitted design files / 79,447,423 bytes**, the selected private Node
**24.22.1** binary and Noto Sans font hashes, and the current compiled design and
navigation adapter modules. The old model candidate directory's
`local-evaluation-only` receipt was rejected by the package verifier. A fresh
**172,668,906-byte** semantic package was assembled from the already-pinned local
model files and ONNX Runtime archive, and passed the authoritative package
verifier. No download, dependency addition, upstream helper, user installation or
client setting change occurred.

| Observation | Measured result |
| --- | --- |
| Runner outcome | Failed at the required indexed-save assertion after 2,398.4 ms |
| Canonical save | `committed`, revision 1; no receipt warning |
| Semantic index | `pending`; model inference not started |
| Native admission snapshot | 57,049,088 resident bytes; 4 processes; `Warning` pressure |
| Independently sampled pressure | `kern.memorystatus_vm_pressure_level = 2` before and during the attempt |
| Independently sampled server-tree RSS | 39,712 KiB before use; 45,136 KiB at cold indexing; three server processes |
| Heavy ownership markers observed | 0 |
| Available disk before the run | 45,589,286,912 bytes, above the 25 GiB floor |
| Overlap, PNG publication, navigation and negative SVG case | Not reached |

The actual returned warning was:

```text
embedding inference not started: memory resources defer embedding load: Snapshot { bytes: 57049088, process_count: 4, pressure: Warning }
```

`memory_pressure -Q` subsequently reported **49% system-wide memory free**, while
the pressure sysctl used by the implementation continued to report **2/Warning**
across repeated read-only observations. These are different observations; the
free percentage did not satisfy Mirket's Normal-pressure admission condition.
No threshold was changed, no unrelated process was killed, and no second attempt
was made while Warning persisted. The 8 GiB resident threshold was not approached.

### Cleanup, evidence and remaining limits

After EOF, independent native inspection found no fixture processes, scratch jobs
or unresolved ownership markers. Both heavy-slot locks and all eight waiting-slot
locks were available. One registration row remained unlocked with its registered
PID gone. This is the implementation's permitted stale-row state after Drop
contends on the registration gate; it is not live ownership or an unresolved
heavy marker. Its lock and absent PID were checked explicitly. Automatic pruning
of that row was not exercised by this attempt.

Evidence is retained privately under
`~/.codex/task-state/harness-rewrite-implementation/combined-load-qualification-20261026/`:

- `combined-load-NPsJ14/result.json` contains inputs/source hashes, MCP timings,
  process/admission samples and the actual assertion failure. SHA-256:
  `c6c87ddf831eb7c31811a466e4e76fc6216e208b7fa0ec772717faecffa7b6fd`.
- `first-run-runner.mjs` preserves the exact executed runner, matching its recorded
  SHA-256 `16f943412561d813b6b3e199f4fdceb8364dead1f8386c5a327b209e247a380e`.
- `pending-commit.json`, `first-deferral-cleanup.json` and
  `first-deferral-provenance.json` retain the returned canonical receipt,
  native cleanup checks and relocation/file hashes. `first-deferred-fixture/`
  preserves the stopped fixture's core, state and source bytes; provenance maps
  its original temporary path to this retained location.
- `mirket-current` and `semantic-runtime/` preserve the exact core and verified
  semantic package. `native-probe.log` preserves the original runner output.

The source runner was then improved to retain a full pending commit and native
cleanup evidence directly on failure, and to accept only unlocked stale
registration rows after EOF. That revised runner passed Node syntax validation;
it was not rerun because pressure remained elevated. Its successful overlap and
post-render branches remain unqualified.

The observations establish one real fail-closed pressure boundary and subsequent
owned cleanup. They do not establish combined throughput, latency distributions,
peak or unique physical memory, 10k/100k record scale, 8 GiB deferral, injected
deadline behavior, browser/video concurrency, Windows ownership, installed
packaging, rights admission or actual Codex Desktop activation. Per-process RSS
sums can double-count shared pages. The separate process/file reads are not an
atomic copy of the internal admission snapshot.
