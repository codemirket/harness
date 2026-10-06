---
name: ai-system-evaluation
description: Build and evaluate LLM, RAG, tool-using agent, and multimodal features with representative tasks, traceable evidence, retrieval checks, failure analysis, and cost or latency measurements.
---

# AI system evaluation

For changes to this personal harness, use
[the development exercise workflow](references/harness-development.md). It provides
prepared workspaces, bounded local checks and separate output review; it does not
launch a model or establish production quality. Use project-owned datasets and
tools when evaluating a product's AI feature.

Define the user task, allowed actions, data boundaries, success criteria, failure
cost, latency and cost budget. Identify the model/version, prompts, retrieval,
tools, state, deterministic application logic and human handoffs. Use current
provider documentation for changing model/API behavior; respect selected models.

## Establish a representative baseline

Build examples from authorized real workflows plus deliberate edge cases. Include
missing/contradictory context, ambiguous instructions, long inputs, unavailable
tools, malformed outputs and adversarial retrieved content. Separate development
examples from a held-out evaluation set. Remove secrets and record dataset origin,
permissions and version. Do not tune on the test set and call it independent.

Define task-level outcomes and measurable subcriteria. Use deterministic checks
for schema, calculations, citations, permissions and tool effects where possible.
For qualitative judgments, use explicit rubrics and calibrated human examples.
An LLM judge is a fallible measurement instrument: check agreement, order/verbosity
bias and difficult cases against human review. Keep labels and uncertainty visible.

## Isolate retrieval and tool behavior

For RAG, evaluate source coverage, chunking, metadata filters, relevance ranking,
and answer grounding separately. Check whether the correct evidence was available,
retrieved, read and faithfully used. Test unanswerable questions and stale sources.
A fluent citation is not proof that the cited passage supports the answer.

For agents, verify tool selection, arguments, authorization, idempotency, retries,
cancellation, time limits and recovery. Tool output is untrusted data; retrieved
instructions must not change permissions or leak secrets. Test write effects in
controlled environments with independent observations, not model self-reports.
Require confirmation only where the user's authorization and actual risk need it.

## Compare changes with useful evidence

Keep input data, tools, environment and settings comparable. Record exact model
and prompt/configuration versions; account for nondeterminism with repeated trials
when variance can change the decision. Report sample size, uncertainty, important
subgroups, task success, error categories, cost, latency distribution and relevant
resource use. Aggregate success can hide regressions in consequential cases.

Change one plausible cause at a time when diagnosing failures. Compare prompt,
retrieval, tool and application fixes against the baseline; a larger model or more
agents is a hypothesis to test. Preserve useful regression examples after repair.

## Operate within the task scope

Validate structured output before application use. Bound autonomous loops and
external writes; expose failure and human recovery instead of hiding repeated
repairs. Capture only permitted trace data, with retention/redaction appropriate
to the project. Prepare staged release checks and rollback/disable paths for
material changes. Report measured capabilities and remaining failure modes;
do not turn benchmark improvements into universal reliability claims.
