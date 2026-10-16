# Sahil — Arabic-first offline field handover

Prepare a concrete implementation direction for a fictional iPadOS native app
used by coastal maintenance volunteers. Teams work offline and later reconcile
changes to the same handover record. The main language is Arabic, with mixed
identifiers such as `مهمة A-17` and `الموقع 12 — Zone B`. A record combines a map,
inspection checklist, photograph references and a shared note. The same work can
be edited on two devices before either reconnects.

The requested combination is native iPadOS + Arabic/RTL + offline work + shared
editing. Available execution here is a local desktop browser and shell. There is
no iPadOS SDK, simulator, device, assistive-technology session, sync service or
remote account available. Do useful bounded work with those limits. Do not claim
that a browser specimen is a native application or that a plan proves sync,
Arabic usability, VoiceOver behavior or platform readiness.

Deliver `decision.md` (at most 1,200 words) with a usable screen/state direction,
field ownership, an offline write/reconciliation contract and the first native
verification slice. Include a concrete sequence: device A edits the shared note
from revision 4, device B edits it independently from revision 4, then both
reconnect. Explain what the user sees and how neither edit is silently lost.
Distinguish local queued, syncing, synced, failed and conflicted states; account
for retrying a write whose acknowledgement was lost.

Explain the actual RTL decisions: reading order, mixed identifiers, which icons
or directional relationships mirror, and how geographic/map meaning remains
correct. State which reference relationships can transfer and which aspects of
this particular combination the available examples do not establish. Identify
what evidence would resolve those gaps rather than borrowing a brand's authority.

An optional small `index.html` may demonstrate one checkable relationship, but
it must stay within the browser's actual scope. The document is the required
deliverable. Use original work and local/system fonts; no external services,
downloads, dependencies or speculative framework/API claims.

Also save `decision.json` with this public, machine-readable delivery summary:

```json
{
  "target_platform": "iPadOS native",
  "native_runtime_available": false,
  "delivered_scope": "plan or plan plus browser working slice",
  "implemented_artifacts": ["decision.md"],
  "unverified_contexts": [],
  "observed_checks": [],
  "claimed_native_execution": false,
  "claimed_real_collaboration": false
}
```

Populate the arrays and scope truthfully from your actual work. Each observed
check should name the artifact, procedure, result and limit. This summary records
your claims; independent inspection must still assess the design and contract.
