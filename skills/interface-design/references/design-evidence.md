# Reconstruct the system that reaches the screen

Use when documenting an existing design language or checking suspected system
drift. Keep the project's established design document and token format. A narrow
repair needs only the relevant evidence, not a new design-system document.

## Trace intent and observation separately

Start at the selected route and its rendered state. Follow imports, providers,
component variants, theme resolution, CSS inheritance and loaded styles to their
owners. A token file with a matching name does not establish that it governs this
application. Check actual consumers and computed values at the affected viewport
and theme; exclude unused exports, legacy styles and proposed migrations unless
they reach this surface or explicitly govern its intended replacement.

Keep a compact evidence chain for claims worth carrying forward:

**role → value → source → scope → recurrence → confidence**

- **Role/value:** what the value does here; preserve a real token name only when
  its declaration and use are established. A descriptive role is not a new token.
- **Source:** file and symbol/selector at a revision, accepted decision, or URL
  plus observation date and rendered element. Link declarations to consumers.
- **Scope:** application, component/variant, state, theme and viewport; record
  exceptions and overrides instead of flattening modes into one universal rule.
- **Recurrence:** where the same role actually recurs, and the samples checked.
  Repeated literals are observations, not proof of a canonical scale or intent.
- **Confidence:** distinguish source-governed intent, verified rendered behavior,
  reconstruction from limited samples and unresolved ownership. Explain the gap.

With repository access, accepted decisions establish intended behavior while
source and runtime show implementation. Record contradictions between them.
With only a public surface, DOM/computed styles and loaded CSS can establish
observed values and exposed variables, not private token names or design rationale.
Pixels alone cannot prove an exact CSS value or token origin. Label reconstruction
and its sampling limits; do not silently turn it into approved brand direction.

Preserve accepted decisions and alternate themes when updating documentation.
Put provenance in the document or a linked durable evidence record. Use the
existing formatter/validator when available; no particular DESIGN.md schema,
exporter or remote command is required to communicate a supported result.

## Make the kind of finding explicit

A **design-system violation** needs a binding contract, a demonstrated runtime
connection, and a correction supported by both. Reopen the cited sources and try
to disprove it: is the variant intentionally different, the rule obsolete, the
stylesheet unloaded, or the override accepted? Inspect other affected consumers
before repairing a shared owner.

An **observed usability/accessibility defect** can be established through the
actual task, keyboard behavior, rendering or user evidence without a binding
design-system rule. An **improvement proposal** may have several credible fixes;
state its rationale and uncertainty. Do not discard either because a conformance
gate cannot select one mandatory correction. Implement authorized fixes and
reinspect their real behavior; this method does not require a stop-for-plan step.

A handoff must stand alone: affected route/state, source revision and owners,
current versus intended behavior, evidence, exact proposed edit or open choice,
affected consumers and a reproducible check. Use the existing task record rather
than imposing a report format.

## Worked counterexample

Suppose `legacy/tokens.css` defines `--action: violet`, but the app imports
`theme.css`, whose light/dark `--action` values feed `Button.primary`. A local
toolbar repeats violet literals. Do not document violet as the product action
color or call the primary button inconsistent from search results alone.

Trace the import and matching cascade in both themes. Record `Button.primary`'s
governed action role with those theme scopes; record toolbar violet as a local
observation. If an accepted decision requires toolbar actions to use the shared
variant, prove that decision covers this toolbar before correcting it. If instead
the toolbar loses visible focus, fix the demonstrated keyboard defect regardless
of whether the token decision exists. A source-only investigation leaves actual
computed appearance unverified until rendered.

## Technique provenance

Adapted selectively in original wording from ibelick/ui-skills at
[`9a0cb1e610e3a1a4f0f2723bf2e7680ac1b89cf5`](https://github.com/ibelick/ui-skills/tree/9a0cb1e610e3a1a4f0f2723bf2e7680ac1b89cf5):
`skills/create-design-md/SKILL.md` (evidence chain and governing scope) and
`skills/improve-ui/SKILL.md` (ownership, proof and falsification). MIT, Julien
Thibeaut; notice accompanies installed guidance. Remote registry entries are
outside this adaptation. Format mandates, implementation refusal and exclusions
of rendered, functional or accessibility evidence are intentionally not adopted.
