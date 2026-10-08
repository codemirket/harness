# Completion evidence by task

Read the affected domain for substantial work. Use the project's accepted
requirements and checks first; these standards help choose evidence, not create
new product documents or force every task through every workflow.

## Context and tools

Establish the requested result, affected consumers, source owners and consequential
failure cases. Inspect actual stack versions and available tools. A registered
skill does not supply a browser, document renderer, API client, test database or
authenticated account. Exercise the needed tool within scope before relying on
it; report a material missing capability and continue with a workable path.

Use one lead workflow, with supporting expertise for actual needs. Read applicable
references and preserve useful operational detail. For a long or delegated task,
retain a compact note of decisions, affected inputs, checks and unresolved gaps.
A routine edit needs no new task record, review panel or specialist installation.

Before editing, choose the observable result that will establish completion; use
the existing project workflow to obtain it after editing. In the final report,
state what was exercised or inspected, the result, and a material gap if one
remains. "Configured", "captured" and "passed review" describe different evidence.
An unavailable check stays unverified; it is not replaced by a self-awarded score.

## Frontend and motion

Name the user's task, primary action, content hierarchy and intended visual
improvement. Select application or public-page composition guidance in
`interface-design`; select `motion-design` for control interactions,
`page-transitions` for navigation or `advanced-motion` for complex choreography.
Read the relevant craft reference. Existing brand, components and behavior remain
authoritative.

For visual refinement, choose a concrete direction from the project's design
owners or supplied references and name the baseline defects. Use comparable
captures to assess those defects, then correct unresolved ones before claiming
improvement. Keep this brief in the working task; no new design document is needed.

Inspect actual renders with realistic content at affected widths, states and
themes. For an improvement, compare equivalent before/after views and refine
visible defects. Exercise changed controls, keyboard/focus behavior, asynchronous
states and interrupted/reduced motion where applicable. A successful build,
screenshot capture or automated accessibility scan is supporting evidence;
visual acceptance requires judgment against the requested goal.

For navigation motion, exercise actual route commitment, late data, failures,
rapid requests, history, direct entry, focus and scroll on native and fallback
paths. Visual capture failure must not duplicate application updates. For advanced
choreography, inspect timing, origins, intermediate beats, responsive geometry,
interruption and disposal; verify scroll/3D support and readable static content.
Use the actual reduced-motion path at load and after a live preference change.
Frame samples do not establish performance or physical device readiness.

For vector assets, use `svg-creation` for deliberate composition and geometry;
inspect silhouette, curves, optical weight, effect bounds and the minimum displayed
size. Deliver editable source and verify real embedding plus repeated inline IDs.
For vector motion, use `svg-animation`; inspect normal playback and intermediate
beats, actual playback controls, reduced motion at load/live changes and lifecycle.
Pure-vector delivery cannot be satisfied by a raster wrapped in SVG. Structural
audits and sampled frames do not certify aesthetics, smoothness or untested exports.

## Engineering

Define observable behavior and the affected interface/data invariants. Use
`engineering-judgment` and the project's implementation method, adding appropriate
architecture/debugging/test/stack expertise. Distinguish request-owned data,
durable state and local interaction state; test relevant failure and concurrency
cases rather than mirroring implementation details.

Complete affected project gates and inspect actual consumers when mocks cannot
establish integration, authorization, cache ownership or browser behavior. Support
performance claims with measurements and environment assumptions. Explain material
tradeoffs, compatibility changes and remaining verification gaps.

## Documentation and artifacts

Identify the reader, owning source, requested change and final format. Technical
guides must match the implementation: verify examples, commands, API signatures,
defaults, errors and links. Record decisions in existing owners; an ADR is useful
for a consequential choice, not every edit. Use the technical-documentation
reference in `document-workflow` or the project's selected documentation specialist.

For office artifacts, verify affected content, numbers, units, formulas, links and
revision semantics against sources. Reopen/render/recalculate using the available
format tool and inspect affected pages, sheets or slides. A successful save does
not prove content fidelity or usable presentation.

## Integrations

Choose general API/service expertise for connectors; use `mcp-integration` for MCP
protocol/client work. Read `engineering-judgment`'s service-integration reference
for the underlying provider boundary. Keep the existing contract authoritative
across consumers and providers; do not manufacture a second integration pattern.

Verify affected schemas, auth/scope, pagination, rate limits, timeout/cancellation,
retry and idempotency behavior with a controlled service or authorized bounded
calls. A write timeout is an uncertain outcome: use provider reconciliation before
replay when the operation may have committed. Check mapping/error behavior and
relevant webhook duplication/order cases. Record separately what was implemented,
registered, authenticated and actually exercised; synthetic success cannot prove
a live provider or tenant boundary.

## Research

Frame the decision and break it into checkable claims. Use appropriate current
sources and installed-version documentation; treat snippets as discovery leads.
Link each material claim to its supporting source and explain applicability,
counterevidence and uncertainty. Supplied-source synthesis must retain the source
set's limits. Do not describe a vendor claim or several copies of one announcement
as independently established evidence.

Use the claim-verification reference in `research-and-synthesis` for substantial
comparisons. Stop when the decision has adequate support or a concrete gap needs
new data/access. Keep sources, inferences and recommendations distinguishable.

## Evaluate the harness separately

Registration, declared selection, observed execution and accepted output are
different evidence. Development exercises can expose defects, but are not a
held-out benchmark. Use `ai-system-evaluation` for representative comparisons;
preserve exact input/configuration versions, actual artifacts and reviewer basis.
An agent's reported selection or self-awarded score does not certify application
or quality. Retain human and agent review as distinct kinds of evidence.
