# Tidepool design acceptance sample — 2026-10-28

The authorized follow-up used two new held-out briefs, each with the same
tools, task inputs and model settings in specialist and no-specialist conditions.
This is a finite diagnostic sample, not evidence that the full skill catalog
passes acceptance or that one condition generally improves design quality.

## Frozen execution and actual outcomes

Four initial turns had 240-second ceilings, one shared 1,500-second author/repair
budget, at most one 180-second repair, and a 1,800-second wall window including
review. Actual author time was **934.254 seconds**; the sample was finalized
after **1,066.364 wall seconds**. No retries or repairs ran. Failed authors were
ineligible for repair; the sole completed author had no actionable findings.
Original D4/D5 budgets remain closed.

| Artifact | Condition | Author result | Independently observed artifact |
| --- | --- | --- | --- |
| `artifact-27559605` / responsive UI | Specialist guidance | Timed out at 240.019s; strict snapshot rejected browser-profile links | Safely recovered required regular files had a coherent illustration and intact text at 1280×800 and 390×844. Sky filtering by Space, unmatched search, error and Enter/Retry recovery worked. Reduced-motion inspection found no animations and no horizontal overflow. Partial diagnostic only. |
| `artifact-ad9ec377` / responsive UI | No specialist guidance | Timed out at 240.038s | Same tested interactions and reduced-motion checks worked, but the hero image was missing and its fallback text clipped. The missing image exists in neither the original output nor its snapshot; it is not a review-pack omission. |
| `artifact-afd6f65a` / captioned motion | No specialist guidance | Completed in 214.180s | Actual Mirket MP4 decoded to 36 frames, 640×360, 12fps, 3s, no audio. Star approaches in frames 12–17, contact holds 18–23, final title holds 24–35. Separate captions, intact descender and retained star; no actionable visual finding. |
| `artifact-e22f4076` / captioned motion | Specialist guidance | Timed out at 240.016s during review | Actual Mirket MP4 has the same required dimensions/frame count/timing and no audio. Decoded star approach reaches the lens around frames 18–19, holds through 23, final copy holds 24–35. No actionable visual finding, but artifact quality does not change the failed author outcome. |

Both motion authors used `video_prepare/start/status/result`; both UI authors
used `design_prepare/start/job_status`. These receipts were checked against
saved output and independent decoded pixels. A capable independent reviewer
received neutral artifact IDs, brief/rubric and saved pixels; the lead owned
browser interactions and reconciled findings. Source/style can reveal guidance,
so masking is not authenticated blindness. The small, timeout-heavy sample
cannot establish specialist benefit, model superiority or broad quality.

## Demonstrated defects and corrections

| Consequence | Cause and smallest correction | Outcome check |
| --- | --- | --- |
| A valid dashed illustration was rejected, leaving the UI hero absent. | `adapters/src/design-operations.ts` omitted `stroke-dasharray` and its paired `stroke-dashoffset` from the closed attribute set. Both now pass the existing value/resource checks. No renderer or dependency was added. | The exact failed SVG succeeds through a fresh disposable MCP optimize operation. Original and optimized SVGs both rasterize through Mirket; all 409,600 RGBA pixels are identical. Saved pixels were viewed. A remote URL in a dash value still fails and writes no output. Regression checks preserve dash styling and reject external values. |
| A browser-profile link prevented a canonical snapshot and its manifest, obscuring otherwise valid partial output. | `tests/motion-feedback-pilot.py` now copies regular files through no-follow descriptors, records unsafe/special entries separately with hashes of retained safe files, and still rejects the snapshot without a success manifest. | Replay of the original failed snapshot retains 13 safe files and records three refused symlinks. Required editable files survive; no links enter the new snapshot. Tests cover external file/directory links, FIFOs, root/destination links, swaps between stat/open, clean success and overwrite refusal. |
| A timed-out UI author left Chrome running. | Host turn/process-group shutdown did not stop a detached host browser. The lead recorded and terminated only the verified process with this disposable profile; its seven observed processes were gone on recheck. | `host-review/browser-cleanup.json` retains exact before/after identity. This cleanup is not a claim that Mirket can cancel arbitrary native host processes. The separate coordination follow-up proves the supported host terminal stop path for an owned command. |

The first UI encountered a local browser connection permission error; automatic
approval review did not ensure it completed inside the frozen ceiling. The
second UI truthfully reported the SVG refusal. Neither failed trial was repaired
or rerun to improve its score. The runtime correction was checked separately on
the retained source after the sample closed. Existing artifact findings remain.

## Verification and limits

Actual platform: macOS 26.5.2 ARM64; Desktop-bundled Codex CLI
`0.164.0-alpha.11.1`, configured Astra/high, private Node 24.22.1. All author homes
and projects were disposable, with automatic approval review. No live Mirket
installation, release or remote branch changed. Auth copies were removed.

Fresh gates: build-input guard; locked/offline Rust tests (506 passed, five
ignored); formatting; locked adapter compilation and process tests (68 passed,
one skipped); 55 affected Python tests; 12 coordination runner tests; catalog
hash/provenance refresh check (no drift); explicit private-Node/binary MCP smoke.
All-target locked/offline Clippy with warnings denied passed.
No skill body or provenance classification changed, so no catalog hash update
was needed. Planned catalog acceptance remains planned.

The private FFmpeg rejects the old `-vsync` option and lacks the tile filter;
full frame decoding succeeded with `-fps_mode passthrough`. Contact sheets were
assembled from decoded frames with the existing host Pillow runtime. Native-size
frames were inspected as well. Browser testing used Codex's in-app browser;
Mirket's private Chrome remained reserved for video rendering. Two initial
manual MCP checks failed before execution because the new state directory was
absent and then had overly broad permissions; creating it with private permissions
resolved this setup issue without weakening the runtime check.

Windows ownership, packaging/capability fidelity, Mac OS-floor qualification,
public recipient distribution and the waived Desktop GUI gate remain unverified.
The original mechanical child stall has no established root cause. The new
coordination success and these visual reviews do not erase earlier failures.

## Retained evidence

Frozen inputs, author events, snapshots, neutral packs, budget ledger and final
study record: `/Users/Shared/mirket-design-acceptance-20261028`.
External seal, original frozen runners, host screenshots/interactions, both
complete independent decodes, reviewer findings, collector replay, real MCP SVG
replay and gate logs:
`/Users/nazmiyilmaz/.codex/task-state/harness-design-acceptance-20261028`.
The old sealed adapter/runner identities intentionally differ from the fixes
made after finalization; do not rerun or reclassify that frozen cohort with new
code. Native coordination evidence is documented in
[the separate follow-up](coordination-followup-20261028.md).

## Subsequent UI handoff diagnostic at `d13b633`

The user authorized continuing. Trace review identified avoidable setup work:
both isolated UI authors attempted to launch a host browser despite having no
configured Codex browser connection, and both motion authors tried unsupported
private FFmpeg options. These observations do not establish the full cause of
the timeouts. The common trial contract now assigns isolated UI source/static
checks to the author and Desktop browser/pixel checks to the lead. This is
observed guidance, not interception of arbitrary host commands. Both comparison
arms receive the same prerequisites. The video reference now supplies the
verified `-fps_mode passthrough -start_number 0` recipe and explains that montage
filters are absent. The recipe produced all 36 PNGs, byte-identical to the prior
independent decode. Catalog resource hashes were refreshed using
`scripts/refresh-skill-catalog.py --write`; reference rights/classifications did
not change.

A new Lantern repair-club pair froze two 240-second turns, 510 shared model
seconds including shutdown allowance, zero repairs/retries and one 1200-second
wall/review window. It closed after **480.060 model seconds / 622.962 wall
seconds**. Both authors explicitly respected the browser handoff and neither
launched a browser, but **both still timed out**. No reliable completion or
causal speedup claim follows. The new brief, SVG runtime fix and contract changes
also prevent interpreting this as a controlled timing comparison with Tidepool.

| Artifact | Author outcome | Independent saved-artifact outcome |
| --- | --- | --- |
| `artifact-6dee41a8` / specialist | 240.030s timeout plus unallowed root `lantern.svg` | Desktop/mobile pixels, loaded illustration, Textiles filtering by Space, unmatched search, refresh error and Enter/Retry restoring all three sessions checked. 390px page bounds and no animations under reduced motion. Reviewer found no actionable visual issue. |
| `artifact-ae6f7470` / no specialist | 240.029s timeout; saved checks and handoff but no native completion event before cutoff | Desktop/mobile pixels, Furniture filtering by Space, unmatched search, refresh error and Enter/Retry restoring all three sessions checked. 390px page bounds and no animations under reduced motion. Reviewer found no actionable visual issue. |

The first neutral pack omitted the out-of-scope SVG even though the actual
snapshot contained it. A labelled diagnostic copy restored that exact hashed
asset for review; the original pack and failed author record remain unchanged.
After closure, the collector was corrected to retain safe visual files from the
snapshot outside `.agents`, including those outside the permitted output scope,
and to carry `author_completed: false` in its review template. This preserves
diagnostic fidelity without admitting unauthorized output or making a failed
cell repair-eligible. New instructions explicitly limit root outputs and place
supporting assets under `.evidence/` or inline. No author was rerun after this
clarification, so its model compliance remains untested.

A separate checker defect was reproduced: a native completed turn with no
required editable output could previously enter the review stage as an
`author_completed` result. Missing required outputs now force failure. Tests
cover missing output, a scope-failed visual asset retained in the review pack,
matched input equality, and refusal to allocate any repair in this pair.

Fresh checks for this continuation: guarded locked/offline Rust build/tests
(506 passed, five ignored), all-target Clippy with warnings denied, formatting,
locked adapter compilation/tests (68 passed, one skipped), explicit private
Node/current-binary MCP smoke, 57 affected Python tests and 14 catalog/assembly
tests. The generic Skill Creator validator could not run because PyYAML was
absent in both available Python runtimes; no dependency was installed to bypass
that. Repository-owned catalog/resource checks passed. `SKILL.md` frontmatter
was unchanged. The last collector/instruction edits affect Python only and were
retested after the closed model pair; runtime gate results precede those edits.

Evidence: `/Users/Shared/mirket-ui-handoff-20261028` holds frozen author inputs,
events, snapshots, original neutral packs, ledger and finalized failed outcomes.
`/Users/nazmiyilmaz/.codex/task-state/harness-ui-handoff-20261028` holds the external
seal, current tested binary identity, host interactions/pixels, independent
reviews, decoder replay and gate logs. Platform/model remain macOS 26.5.2 ARM64,
Astra/high and the same Desktop-bundled CLI/private Node. No full-catalog,
Windows, OS-floor, publication or Desktop GUI gate was promoted. The remaining
completion problem needs a separately scoped timing investigation; repeated
runs under the same ceiling would not establish its cause.

## Timing investigation after the closed handoff pair

The retained chronology narrows the failure without explaining all of it.
The specialist saved its main UI files around 192.6 seconds after phase
reservation; the other author saved `.evidence/HANDOFF.md` at 239.325 seconds.
Neither emitted `turn.completed`, and both exited with SIGTERM at roughly
240.03 seconds. This is evidence against a long post-deadline shutdown wait.
File modification times are only supporting chronology: they cannot partition
the interval into model, service, approval or tool time. No causal timing or
model-ranking conclusion follows from the old buffered event files.

`tests/motion-feedback-pilot.py` now retains raw stdout/stderr during execution
and writes an adjacent `initial-timeline.jsonl` (or repair timeline). It records
monotonic collector observations of stream bytes, native event/item lifecycle,
termination requests and capture completion. The result binds the timeline hash
and distinguishes first output, native turn completion and shutdown time.
This uses Codex's existing [JSONL event stream](https://learn.chatgpt.com/docs/non-interactive-mode#make-output-machine-readable),
not a Mirket agent controller. Observation granularity is 100ms plus scheduling
and I/O delay; stdout/stderr ordering within a poll and provider execution time
are not established. Raw output remains exact, including split UTF-8 and an
unterminated final line. Existing files refuse retry, capture errors reap the
owned process group, and the original absolute deadline and termination grace
remain. A native completion followed by a hung process still fails the trial.

One separately frozen, noncreative native readiness probe requested only
`TIMING_READY`, with Astra/high, automatic approval review, a 45-second ceiling,
no MCP configuration, and zero retries/repairs. It returned the exact requested
text: first stdout at **0.616486s**, native completion at **7.052181s**, process
capture complete/exit 0 at **7.943941s**. No tool or project write was requested;
the project remained empty and the disposable credential copy was removed.
This establishes native capture interoperability, not design latency, Mirket
tool startup performance, motion quality or a fix for the earlier author
timeouts. The frozen native runner precedes a subsequent exceptional-cleanup
strengthening; that error path is covered by real subprocess tests rather than
another native model run.

Fresh verification on macOS 26.5.2 ARM64: 36 motion-runner and 29 matched-study
tests, plus 14 catalog/skill-assembly tests; catalog refresh check reports no
drift. New real subprocess cases cover delayed events/live files before exit,
silent timeout, completed-turn/hung-exit, bidirectional pipe pressure, exact
partial output, interruption, ignored SIGTERM, capture-file collision cleanup
and retry refusal. Independent read-only review caught a scheduling-dependent
interrupt-test assumption; the fixture now interrupts only after observing its
readiness event. Runtime, adapter, skills and dependencies did not change, so
the preceding Rust/Clippy/adapter/MCP gate results are retained, not represented
as fresh. No catalog acceptance or platform gate is promoted.

Evidence and source hashes:
`/Users/nazmiyilmaz/.codex/task-state/harness-timing-investigation-20261028`
(historical chronology, frozen native protocol/runner, result and cleanup,
test logs); native raw streams and timeline:
`/Users/Shared/mirket-timing-probe-20261028`.
Both previous design cohorts remain closed and failed cells remain failed.
The remaining unknown is the distribution of time within a real creative turn;
there is no basis yet to increase the budget, change models or claim a design
completion fix. Future authorized trials can use the new observation trail
without silently resetting a closed cohort's budget.
