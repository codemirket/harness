# Verify claims and their applicability

Read when a conclusion turns on changing technical behavior, consequential facts,
or conflicting evidence. The aim is a supported answer in the user's setting,
not the largest source collection. Use project-owned evidence and current sources
where needed; retain useful unchanged evidence instead of repeatedly retrieving it.

## Turn a broad question into falsifiable claims

For "does the framework cache this request?", identify which cache, key, lifetime,
execution path, version, and deployment context the caller means. Reusing data
within a render, storing it between requests, browser caching, and CDN caching
are different claims. One successful repeated request does not identify the layer.

Inspect the actual workspace and resolved installation, router/adapter, relevant
configuration, and caller path before applying a guide. An experiment should
observe the affected behavior: count underlying provider calls, compare outputs,
and control which inputs or identities change. Source text and runtime behavior
answer different questions; neither automatically establishes the other.

## Worked comparison: a newer guide disagrees with the installed release

Consider an illustrative project resolved to framework version 2.4.7. A current
version-3 guide says a fetch mode is enabled by default. A version-2 reference
says it is opt-in, and a local development trace appears to reuse a result.
Do not average these into "usually cached" or silently recommend an upgrade.

| Evidence | What it can support | Applicability question |
| --- | --- | --- |
| Resolved package and lock entry | Which installed release is being examined | Is this the running workspace and deployment? |
| Official version-2 reference | Documented contract for that release family | Does the exact API/configuration match, and are there corrections? |
| Version-3 guide or release notes | Current behavior or a documented change | Did the change occur after the installed release? |
| Development trace | Observed behavior in that run | Is reuse from development tooling, request memoization, an SDK, or an upstream cache? |
| Controlled representative build | Observed behavior of the affected path | Are identity, request boundaries, configuration, and cache state controlled? |

Read the relevant passages and change notes. A search snippet, copied summary,
or link to the guide is not verified support. Keep publication/update dates apart
from the version or event date they describe. If the version-specific contract
cannot be located, use the observed behavior with its limits and name the gap.

Suppose two calls within one request produce one provider call, while two
separate requests produce two provider calls. This supports reuse within that
observed request context; it does not establish persistent caching. If the user
needs cross-request behavior, test that boundary. If identity-sensitive data is
involved, inspect keys and exercise distinct identities before making isolation
claims. No result from this hypothetical example establishes a particular
framework's actual default.

## Resolve disagreement without hiding it

Compare what each source measured or defined before choosing a conclusion:
population/workload, version, dates, settings, denominator, and incentives. Two
articles based on the same announcement are one origin. A vendor benchmark may
describe its workload accurately while being inapplicable to the user's workload.

Search for a plausible disconfirming result, correction, limitation, or competing
explanation where it would change the decision. Preserve credible negative
findings. If equally applicable evidence conflicts, explain the disagreement and
the next observation that would resolve it; do not assign numerical confidence
without a method. Absence from a search is usually a coverage limit, not proof
that a behavior, feature, or incident does not exist.

Use a compact claim record when useful: exact claim, source and evidence location,
applicability, counterevidence, conclusion, and unresolved gap. A source may support
only part of a sentence; narrow the claim or add the missing evidence. Label an
inference as an inference, and keep calculated scenarios distinct from measured
results. Check consequential calculations with traceable units and inputs.

## Deliver and stop

Lead with what the evidence supports for the user's question. Cite material
claims near their wording and explain only the uncertainty that changes how the
answer should be used. Distinguish a recommendation from a verified outcome.
State the version/environment, coverage, and condition that would change the
conclusion when those matter.

Complete the requested artifact or decision support, including authorized local
verification where required. Stop once important claims have sufficient support
and the deliverable is complete. If a remaining claim needs inaccessible data,
account access, or a new measurement, report that dependency instead of repeating
equivalent searches or replacing the gap with a confident generalization.
