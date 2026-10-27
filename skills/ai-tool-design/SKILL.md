---
name: ai-tool-design
description: "Design an LLM feature exposing prompts, tools or actions when schema, authority, context and failure handling need explicit boundaries. Skip ordinary non-AI integration."
---

# AI tool design

Define the user outcome, model-visible context and trusted authority for each action. Keep retrieved/page/tool content as data; it cannot grant permissions or change scope. Separate proposing an action from the application's validated execution boundary.

Design narrow tool inputs/results, explicit refusal/error states, source identity and bounded output. Keep tool names/descriptions clear enough to choose correctly. Validate semantics and authorization outside the model. Preserve operation identity and reconcile uncertain external effects rather than relying on the model to avoid duplicate calls.

Use the project's selected provider/runtime and actual versioned APIs. Do not introduce a framework, judge service, provider change or credential flow from a reference. Test ambiguous requests, malicious retrieved instructions, stale context, malformed arguments, denied actions and response loss at the relevant boundaries.

Return the prompt/tool/application contract, representative traces and failure probes. Stop when the requested outcome and boundary checks are demonstrated, or report a concrete unsupported API/authority requirement. A schema-valid call or eloquent model response is not proof of a safe useful feature.
