---
name: api-integration
description: "Implement or revise HTTP, MCP or tool contracts when inputs, results, errors, authorization or retry behavior affect consumers. Skip isolated internal functions with no integration boundary."
---

# API integration

Identify producer/consumer owners and the wire contract actually in use. Define validated inputs, result/error forms, version behavior, authentication/authorization context and side effects. Preserve source and operation identity across boundaries where retries or staleness matter.

Specify what a caller may safely retry and how duplicate/uncertain requests are reconciled. Keep transport failure distinct from application refusal and unknown effect. Bound payloads and timeouts; propagate enough context to recover without logging secrets. An input schema does not establish semantic authorization.

Implement the owning boundary and exercise a real consumer, including malformed input, incompatible version, denied authority, timeout or duplicate request as applicable. Check response framing/serialization rather than only direct function calls.

Return the contract, compatible consumer behavior and observed failure evidence. Stop when the requested integration and project gates pass, or report the exact unavailable endpoint/client path. Do not invent a fallback that bypasses the contract or replay uncertain effects.
