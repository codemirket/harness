---
name: ai-system-evaluation
description: Build and evaluate LLM, RAG, tool-using agent, and multimodal features with representative tasks, traceable evidence, retrieval checks, failure analysis, and cost or latency measurements.
---

# AI system evaluation

## Professional stance

View model behavior as an uncertain component inside a measurable application. Ask
whether failures arise from evidence availability, retrieval, reasoning, tool
contracts or deterministic code before changing the prompt or model.

Choose the smallest supported intervention at the failing layer. Change a
recommendation when held-out outcomes or consequential subgroups contradict
aggregate gains; additional agents or larger models require measured benefit.

Ask product which failures are unacceptable and backend/security which actions
need deterministic enforcement. Challenge fluent outputs, self-reported tool
success and evaluation sets reused for tuning.

Provide the evaluated configuration and dataset provenance, task outcomes, failure
categories, uncertainty, budget observations and regression cases; give operations
concrete disable and recovery conditions.

## Working method

For changes to this personal harness, use
[the development exercise workflow](references/harness-development.md). It provides
prepared workspaces, bounded local checks and separate output review; it does not
launch a model or establish production quality. Use project-owned datasets and
tools when evaluating a product's AI feature.

Define the user task, allowed actions, data boundaries, success criteria, failure
cost, latency and cost budget. Identify the model/version, prompts, retrieval,
tools, state, deterministic application logic and human handoffs. Use current
provider documentation for changing model/API behavior; respect selected models.

## Build the application boundary

Inspect the installed SDK, configured model, credential mechanism and available
retrieval/tool runtime. Verify a minimal authorized call through the application's
adapter before scaling an evaluation; a mocked response cannot establish model
availability, tool support or production latency. Keep provider-specific response
handling in the owning adapter and task rules in deterministic application code.

Define how refusal, empty output, invalid structured output, partial streams and
exhausted token/time budgets reach the caller. Parse and validate completed output
before using it as a command or durable result. Bound repair attempts within the
original budget and retain the failed outcome when repair cannot establish the
contract. Exercise the fallback/disable path without silently selecting another
model, provider or billable service.

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
