# Design finding dispositions and repair lineage

This implements D2 of the [remaining design plan](../planning/remaining-design-work.md),
building on the [scope and chronology checks](design-evidence-scope-20261027.md).
It is a repository qualification utility, not a new Mirket tool, approval layer
or agent orchestrator.

## Demonstrated defect and correction

The previous checker accepted a selected candidate with a blocking finding
marked `resolved`, without an after-capture, a repair relationship or a review
of the correction. A new candidate could also omit its parent's unresolved
findings. A shared repair count alone did not bind work to the preserved source
and finding that motivated it.

The checker now requires a one-to-one relationship between each recorded repair
event and a derived candidate. Every derived candidate names its parent, the
parent source-manifest hash, the event and the findings it addresses. Unknown
parents/events, event reuse, cycles and uncharged repairs refuse. Candidate,
specimen and delegate scopes all consume the same ledger. A specimen/delegate
event names `result_candidate_id`; a candidate event's existing `candidate_id`
names the derived candidate.

When a packet records repairs or candidate-specific source manifests, every
candidate needs `source_manifest` and `source_manifest_sha256`. Canonicalization
matches the initial manifest: sort `{path, sha256}` entries by path, then hash
compact, key-sorted UTF-8 JSON. Root candidates retain the frozen initial
manifest. Every source entry has a hash-valid snapshot with `captured_at`;
duplicate snapshot paths refuse. Every capture carries that candidate's
`source_manifest_sha256`. This binds declared source provenance; it cannot
prove that the renderer consumed it without the retained runtime evidence.

A derived snapshot follows its repair event strictly, and a derived capture
follows its latest source snapshot strictly. The repair follows parent captures
and the addressed finding observations. All dates stay inside the one shared
window. Parent artifacts remain on disk and hash-valid; child captures cannot
reuse ancestor capture paths. Overwriting a parent's source invalidates the
packet. For example:

```json
"lineage": {
  "parent_candidate_id": "initial",
  "parent_source_manifest_sha256": "<initial manifest SHA-256>",
  "repair_ref": "repair-1",
  "finding_refs": [
    {"candidate_id": "initial", "finding_id": "contrast24"}
  ]
}
```

Finding IDs remain local to their candidate. Explicit pairs avoid collisions
across multiple repairs. An original finding remains attached to its original
capture, location, observation and consequence. Its status is `open`, `resolved`
or `rejected`. A final status requires a disposition, for example:

```json
"disposition": {
  "kind": "resolved",
  "reason": "After-capture shows readable text and arrow.",
  "brief_sha256": "<frozen brief SHA-256>",
  "rubric_sha256": "<frozen rubric SHA-256>",
  "at": "2026-10-27T20:24:52Z",
  "repair_ref": "repair-1",
  "evidence": {
    "candidate_id": "repaired",
    "capture_ref": "frame24",
    "check_ref": "pixel-review"
  }
}
```

A resolution references a descendant repair that explicitly addresses the
finding, the same state/beat in its after-capture, and a corresponding visual
report or passing automated review. Review follows the capture and precedes the
disposition; automated review records `checked_at`. Check IDs are unique within
a candidate. A file hash check alone cannot resolve a visual finding.

A rejection uses `kind: rejected`, a nonempty reason bound to the frozen brief
and rubric, and a review of the original capture. It does not invent a repair
or erase the original claim. The selected ancestry is checked as a whole:
unresolved ancestor blockers refuse, resolution in another branch does not
clear this branch, and remaining nonblocking findings are returned explicitly.
Selecting the original defective candidate after repairing its child refuses.

This deliberately tightens schema-1 acceptance. Historical bare `closed` or
`resolved` status words, and repair ledgers without lineage, no longer support
a passing current result. Preserve their original historical evidence; do not
invent missing observations to migrate them. Existing zero-repair packets
without resolution claims remain accepted, including the actual D1 packet.

## Actual source/export replay

On macOS 26.5.2 ARM64 with Python 3.15.1, a new deterministic regression replay
froze its initial source, brief, rubric, scope and 30-minute/one-repair ceiling
at `2026-10-27T20:22:49.006915Z`. This was not a new author/model comparison:
zero new model turns were run, and no closed creative-cohort budget was reset.
The repair was explicitly the already retained model-authored `film.js` change.

Both exports used `tests/motion-authoring-runtime.mjs` and the unchanged explicit
private Node/Chrome/FFmpeg/ffprobe paths, compiled adapter and debug Mirket binary
from the preceding qualification. Each initialized the real MCP server, read
`runtime_status`, prepared/started/polled/read the video job, checked its MP4
with ffprobe and decoded saved frames. Each disposable home/state was removed.
No live Mirket installation or Codex configuration changed.

| Stage | Actual outcome |
| --- | --- |
| Initial export, `20:22:58.237660Z` | 640×360, 12 fps, 36 frames; saved frame 24 opened with `view_image` at review time `20:23:38.557901Z`. The label and arrow nearly disappear against the teal button. |
| Repair event, `20:23:38.597893Z` | Copied the preserved input into a separate candidate; replaced only `film.js` with the previously authored correction. The initial source and export remained intact. |
| Repaired export, `20:23:47.349655Z` | Same dimensions/rate/frame count; source and decoded PNG hashes changed. Saved frame 24 opened with `view_image` before review `20:24:51.227650Z`; dark label and arrow are readable on the light button. |
| Complete packet | Accepted: 42 inventoried files, one charged repair, one resolution, no remaining finding. |

The lead inspected both newly saved frame-24 images. The runtime decoded five
samples per export, but the other samples were not visually reviewed in this
run. This replay establishes the targeted readability correction and evidence
relationships, not new playback, audio/caption, overall aesthetic or model-benefit
qualification. Existing broader follow-up evidence remains separate.

Five mutations of this same real packet all passed the pre-D2 checker and
refused after this change:

| Mutation | Current refusal |
| --- | --- |
| Keep `resolved`, delete disposition | Missing disposition |
| Name a nonexistent parent | Unknown/self parent |
| Bind repaired capture to initial source hash | Capture/source mismatch |
| Delete the charged repair event | Unknown/reused repair event |
| Delete the initial finding | Repair finding absent from ancestor |

The valid packet passed again after all controlled probes were removed. Evidence
is retained at `/Users/nazmiyilmaz/.codex/task-state/harness-design-lineage-20261027/`:
`frozen-inputs.json`, snapshots/export/observation records, `repair-event.json`,
`checker-results.json`, `run-summary.json`, replay helpers `journey.py` and
`check-packet.py`, and `packet-root/review.json` with both exports and decoded
pixels. Inventory SHA-256:
`9c287a621603663de9c02e7d0715b70803c3eddb8f480e6be67b5ab0de20ea8c`.
Initial MP4 SHA-256:
`1fd658d04f294d89fa26b1d69c7d5761babbc2750cd469d1fd3aef6e0c161ef4`.
Repaired MP4 SHA-256:
`6d81087091f3e5188764ad2cda7582edc84e822ae1be5b0d49c8c68f00b81de8`.

## Checks and limits

All 87 affected Python tests passed, including 58 checker tests. They cover
two-repair ancestry, candidate-local duplicate finding IDs, branching,
specimen/delegate charges, overwritten source, stale captures, chronology,
rejected claims and remaining defects. Independent read-only review found no
remaining code blocker after stricter snapshot ordering and duplicate-path
checks. Repository catalog refresh reported no drift; diff whitespace checks
passed. The actual D1 zero-repair packet was rechecked successfully.

No Rust, TypeScript adapter, dependency, native skill or catalog input changed;
their baseline compilation gates were not rerun or represented as fresh runs.
The existing actual MCP export route was exercised twice as described above.
The checker remains trust-based: identity, timestamps and judgment are reported,
not authenticated. Rewriting the entire packet cannot be detected; an external
comparison freeze is D3. It cannot determine whether an unnecessary repair was
aesthetic improvement or whether a reviewer missed a defect. The matched
comparison runner and finite UI/illustration/vector/motion cohort remain next.
Desktop GUI stays waived/unverified; Windows and distribution gates stay open.
