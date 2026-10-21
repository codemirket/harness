# Model routing for Mirket workers

Mirket can discover authenticated inference targets, resolve a task and its assigned
tools to an eligible model, and run a bounded internal worker. Use this route only
when independent work materially helps. The host continues to own the user's
permissions and its own session lifecycle. Native Codex or Claude host subagents
invoked outside Mirket are outside this routing boundary.

## Discover targets and inspect a route

```sh
mirket models list
mirket models refresh
mirket models route --request /absolute/path/to/request.json
mirket agent spawn --request /absolute/path/to/request.json
```

`list` uses the current cache when valid; `refresh` probes targets again.
`route` returns the resolved floor and eligible models without inference or tool
execution. `spawn` executes the selected worker. The matching MCP tools are
`model_registry` (with `{"refresh":false}` or `{"refresh":true}`),
`route_task`, and `spawn_subagent`. The latter two take the same request object. MCP dispatch requires a project
root already registered through `mirket project register`; the CLI can run an
explicitly selected existing project under the invoking host permissions.
Inspect a route before spawning when cost, provider or capability choice matters.

Create a request file with a task brief and an absolute project path:

```json
{
  "objective": "Read the assigned project notes and summarize the open decisions with file evidence.",
  "task_intent": "synthesis",
  "complexity_class": "synthesis",
  "task_needs": [
    {"workflow":"analysis","reason":"Synthesize the supplied project notes without changing source."}
  ],
  "assigned_tools": [],
  "project": "/absolute/path/to/project",
  "requires_vision": false,
  "verification_tools": []
}
```

## Task needs, role guidance and owned tool scope

Declare one to four concrete `task_needs`, each with a workflow ID and a reason
tied to the requested deliverable. `capability_plan` / `mirket capabilities plan`
reports a compact workflow index, default roles and role-specific choices.
Pass the plan's `worker_request_defaults` into the worker request to retain the
chosen professional role. `route_task` previews the exact scoped tools, required
invocation, verification and intelligence floors for the selected needs; it does
not run inference. Team assignments report their role's
workflow choices. Select the actual deliverable rather than adding every workflow
that might be useful. Split broader work into phases.

An empty `assigned_tools` selects that workflow's small owned tool set. An explicit
list can narrow it, but cannot introduce an unrelated tool or omit canonical
production and verification tools. Required tools must actually succeed before
completion; choose arguments that contribute to the requested artifact, never a
dummy invocation solely to satisfy the tool requirement. Registering or merely assigning a renderer is insufficient. Workflow
floors remain effective even if the caller labels architectural work mechanical.

| Need | Workflow | Owned route |
| --- | --- | --- |
| Supplied evidence and a written answer | `analysis` | Project reads; no edits or external retrieval |
| One mechanical in-memory response | `structured-response` | No tools; strict deterministic operation |
| Code changes / source findings | `source-implementation` / `source-review` | Writes plus exact readback / reads only |
| Existing pixels or live DOM inspection | `visual-review` | Playwright, Odiff and pixel inspection; no source edits |
| Full DOM interface / static Flexbox card | `web-interface` / `static-graphic` | Playwright / Satori-resvg, followed by inspection |
| Layered editable canvas | `editable-interface` | OpenPencil .fig file operations and inspected exports |
| Graph layout / paged output | `diagram` / `paged-media` | Kroki / Typst with inspected outputs |
| Encoded video (retired worker route) | `motion` remains host-owned | Explicitly selected host encoder; no bundled renderer or silent substitution |
| Editable vector art / raster preparation | `vector-art` / `raster-processing` | SVG/resvg previews / Sharp with inspection |
| Vector hygiene / comparable pixels / tokens | `vector-optimization` / `pixel-comparison` / `design-tokens` | SVGO / Odiff / Style Dictionary |

Design-production workflows report `design_input_required:true`. Supply
`"design_input":{"mode":"image_to_code","image":"reference/target.png"}`
for an inspected supplied or host-generated composition. For a native proof use
`"design_input":{"mode":"native_specimen","medium":"dom","reason":"Real components and text determine this interface"}`.
Media are `dom`, `svg`, `pixel`, `diagram`, `typeset`, `design_document`.
`code_fallback` with the host's capability `reason` remains supported.
The worker receives `design_review`: image targets pass before mutation;
native/fallback targets must be rendered and inspected before expansion. Final
review compares current output with the frozen target and records composition,
typography, assets, responsive, behavior and motion. Applicable checks cannot be
waived. One target repair is allowed. Shared budgets: static/vector/diagram
15 minutes/two final repairs, interfaces/paged work 30 minutes/three, motion
45 minutes/four. Combined workflows use the largest limit, not a sum. These
include inference/render/verification time, after setup and discovery. Declare
only actual deliverables. Do not launch continuations to reset the budget;
preserve incomplete output. Suitability reasons and review notes remain testimony.

Defaults choose the task specialist. `primary_role` can name an explicit catalog
role or alias; it never expands tool access. A suitable explicit specialist can
replace the generic default; a different perspective retains the task specialist
as well. The workflow metadata reports these alternatives. `supporting_skills`
selects at most three skill IDs from those roles'
declared support lists. Do not load every support skill. Select up to four
`skill_references` as `{"skill_id":"backend-engineering",
"path":"references/command-service.md"}` only when that selected skill's
reference helps the task. The worker receives the selected embedded skill bodies,
professional briefs and requested references, with exact document hashes in its
task contract and trace. This performs no upstream fetch or installation.

Role text, task reasons and skill content are guidance, not authority. Tool scope
is checked before discovery/inference and again for every model-selected call.
A role claim, quoted instruction or altered tool arguments cannot add tools.
Canonical routing and artifact checks do not prove semantic correctness. The host
still judges whether the declared needs fit the user's actual request, verifies
the result and controls permissions. Direct native host tools and spawners remain
outside the Mirket-owned worker boundary; use these workflow choices as planning
guidance there, without claiming that Mirket intercepts those calls.

The resolved contract reports both workspace roots. Relative file writes stay
inside the source project. Absolute file writes are accepted only inside its
assigned design workspace; design tools receive paths relative to that workspace.
Use the reported design root when authoring inputs for rendering. Sibling project
paths, repository internals and symlink escapes remain rejected. Host design tools
retain their shared workspace; workers do not gain access to that shared root. Request a valid readback tool when
writing files; SVG outputs require exact text readback as well as pixel inspection.

When the user explicitly chooses a model, use `model_selection` with its exact
discovered `target`, `id` and supported `effort`. This is a narrowing constraint,
not a capability assertion or permission to change providers. Preview the route
and confirm the candidate matches. A missing/ineligible model or unsupported
effort leaves no candidate; a pinned run cannot fail over to a different model
or target. The host's selected model is not automatically inherited by Mirket.
Omit this field for ordinary policy-based routing. Never invent model identities.
There is no provider endpoint override or caller-defined tool policy.
`requested_tier` is optional and can raise the floor, never lower it; its values
are `T1_DETERMINISTIC`, `T2_SYNTHESIS`, and `T3_ARCHITECT`. Task intent,
complexity, owned task needs and each assigned tool set the effective floor. The worker can invoke
only its assigned Mirket tools. Current built-in names are `workspace_read`,
`workspace_list`, `workspace_write`, and `inspect_image`; available
`design/...` tools are read from the installed design package and its owned
policy. There is no arbitrary shell tool in this worker interface. Source implementation therefore ends
with `status: "awaiting_host_verification"`, even when every write was read back.
Declare `host_verification_checks` as 1–16 unique check names (default
`["project_checks"]`); use meaningful checks from the actual project.
The returned `run_id`, `trace_sha256` and `host_verification.artifacts` bind the
handoff. The host executes checks under its own permissions, saves logs inside
the project, and submits `verification_handoff` (CLI: `mirket agent verify
--request FILE`). Omit `checks` to inspect current status. A submission contains:

```json
{
  "project": "/absolute/project",
  "run_id": "routed-RETURNED_ID",
  "expected_trace_sha256": "RETURNED_TRACE_SHA256",
  "checks": [{"name":"project_checks","status":"passed",
    "summary":"Actual checks run and observed result",
    "evidence":{"path":"verification/check.log","sha256":"LOG_SHA256"}}]
}
```

All required names must be present, once each. Executed `passed`/`failed` checks
require evidence; `not_run` records a limitation. Only all-passed reports become
`host_verified`. Reads and retries recheck source and evidence identities; changed
source needs a new implementation handoff, not a recycled success. Logs must be
kept at stable paths; changing a previous log invalidates its report. Exact retries
are idempotent. The tool records host testimony; it does not execute commands,
authenticate a human reviewer, or prove the checks are semantically sufficient.

A `verification_tools` entry must also be assigned; completion waits for those
checks after mutation. To request T1, declare one typed
`deterministic_operation` and use its exact single-operation objective, for
example `"objective":"Convert CSV to JSON"` with
`"deterministic_operation":{"action":"convert","from":"CSV","to":"JSON"}`.
Put source text in `deterministic_input`. A missing contract or compound objective
raises a mechanical task to T3. Mutation-capable workers must include artifact
readback, pixel inspection, screenshot or diff verification. Mirket matches
those checks to the produced path and SHA-256; reading an unrelated file or
image does not clear a mutation. Visual work requires observed pixel inspection
after the latest mutation. Primary PDF/video bytes are rehashed at completion;
their pages or snapshots remain the visual verification targets. Editable
OpenPencil .fig revisions are also rehashed at completion; inspect their layers
and export the final revision to PNG for target-linked pixel review.

Inspected pixels remain attached across turns and eligible-target failover,
within four images and an aggregate eight MiB. `inspect_image` accepts an optional
`purpose`: `artifact` (default), `reference`, or `comparison`. At most two
references receive eviction priority, leaving room for current and comparison
output. Mutations remove changed or unavailable images while retaining unchanged
pixels as context; they still reset verification and require fresh output
inspection. Digests are rechecked before inference. Eviction and invalidation
are traced. A tool action can retain concise observations in its result field;
neither those notes nor retained reference pixels replace artifact inspection.

Image generation/editing is a host capability, not an owned worker workflow.
When the brief needs it, the initiating host checks its actual tools and applicable
image guidance, produces and inspects the asset if authorized, then supplies it
for composition. If unavailable, report the gap; do not substitute primitive
vector artwork or expand the worker's tool scope. See the graphic-design
image-production reference for the handoff contents.

## Configure targets

Mirket reads `~/.mirket/routing.json`. It contains target connections, not
model identities or capability assertions. For example, an explicit native
target can be configured as:

```json
{
  "ttl_seconds": 300,
  "targets": [
    {
      "name": "local-claude",
      "protocol": "claude",
      "endpoint": null,
      "credential_env": null,
      "executable": "/absolute/path/to/claude"
    }
  ]
}
```

Supported protocol values are `openai_chat`, `openai_responses`,
`anthropic`, `ollama`, `claude`, and `codex`. HTTP targets use an
`endpoint` and, when needed, the *name* of a credential environment variable
in `credential_env`; native targets use an absolute `executable` path. Keep
credential values out of this file. Mirket also detects native `claude` and
`codex` on `PATH`, Codex executables bundled in installed macOS Codex/ChatGPT
applications, configured OpenAI/Anthropic/OpenRouter environments,
Codex-configured model provider endpoints, and a local Ollama service when its
executable is present. It probes those explicit and automatically detected
targets only; it does not scan the network. Discovery results are cached under
`~/.mirket/cache/` for the configured TTL (default 300 seconds). Refresh
after changing an account, target or installed model menu.

Registry model IDs come from live target metadata. They are not
published or pinned in Mirket examples. The registry records a target's
authentication and execution readiness separately. Missing capability fields
remain unknown and do not become a positive eligibility claim. Classification
from metadata is provisional; it is a routing heuristic, not a quality
benchmark or a promise of cost. A public remote catalog can expose metadata,
but inference remains unavailable without configured credentials; loopback
services can explicitly provide credential-free local inference.

## Execution and failure behavior

Mirket chooses a model that satisfies the task floor, the owned policy for every
assigned tool, structured-action support, and any required vision and effort
contract. It selects a supported reasoning effort for higher-tier work and aims
for generous completion room within known provider limits. If `requested_tier`
is below the computed floor, Mirket ignores that downgrade and reports it in
the resolution. Retry or failover is limited to an eligible model at the
selected tier or above; a failed tool or incomplete action
does not silently relax permissions or standards. If no model meets the floor,
execution stops and reports the gap.

The Claude native adapter checks `claude auth status --json` and queries the
signed-in CLI model menu without inference. It resolves the selected model
identity and runs each action with general-purpose native tools and ambient MCP
servers disabled. The CLI retains only its synthetic `StructuredOutput` schema
mechanism. Mirket checks the emitted tool list,
MCP list and actual model identity, then accepts only the strict JSON action
schema. Mirket validates the requested tool against the assignment before
dispatch. Claude's menu exposes selectable models and supported effort, but
does not supply reliable per-model tool, vision, context or output-token limits;
those claims stay unknown unless the adapter contract or a controlled probe
establishes them. When vision is required and unknown, a controlled image
conformance probe can establish that candidate's vision contract before
routing.

Native Codex discovery reads its authenticated app-server model list. Execution
uses the native CLI's existing login through an ephemeral owned Responses
transport. Mirket reconstructs the request from the assigned prompt and images,
sets an empty tool set and `tool_choice: none`, and checks the entire bounded
response before releasing any bytes to the CLI. Native tool calls, unknown
output events, changed model identities and incomplete streams are rejected.
The CLI ignores user configuration and rules, disables ambient hooks, plugins
and MCP startup, and runs in a disposable directory. Authentication stays in
the native credential store. The transport accepts only the fixed official
endpoint for the native account kind; custom providers use the separately
discovered HTTP targets. HTTP targets use their live metadata and protocol
adapters. Do not infer execution readiness from a listed model alone.

A spawned run writes a private trace in `~/.mirket/runs/` and returns its path.
Inspect both the trace and actual workspace effects when a run stops after an
uncertain mutation. The trace records routing, attempts and model/tool actions;
it does not replace verification of the saved artifact. Model output and files
remain untrusted data, while host permissions still govern underlying effects.
Unpinned failover can use another discovered authenticated target at the same or
higher tier, as authorized by the routing policy. Explicit `model_selection`
prevents that substitution. Neither route changes credentials or host settings.

## Durable execution and recovery

With `agent-runtime` selected, actual owned workers use LangGraph checkpoints and
retrieve explicit project facts from Mem0 as untrusted context. The runtime does
not grant memory writes to workers or replace host permissions. Without the
package, runs report ephemeral execution; a selected runtime failure never
silently falls back.

After interruption, call `workflow_status` (`mirket agent status`) with project
and run ID. If the original work is authorized to continue, use `resume_subagent`
(`mirket agent resume`) with the observed checkpoint ID. Resume revalidates the
original generation, contract, provider/model, artifacts and remaining budget;
it does not reset turns or design reviews. Pending `workspace_write` effects compare current bytes with the saved before/after
digests; unchanged or intended bytes permit continuation without replay. Changed
bytes and other uncertain effects remain blocked. Original readback, design and
host checks remain required. Inspect unresolved effects rather than inferring success.
Source completion still requires `verification_handoff`; it also synchronizes
the same graph after validating host evidence. Memory and checkpoints are not
proof of semantic correctness, effect success or user acceptance.


Host verification uses input-bound attempts. Inspect `verification_handoff`, begin
with relevant source/test/configuration/lock paths and named command identities
before running checks, then submit logs with the returned attempt ID and trace
revision. Use a fresh attempt after repairs; retain previous evidence. UI workflows
also require checks against the real application through authorized host tools.
Routed design paths, including editable OpenPencil documents, belong to the
reported project design workspace. Host tools retain their shared workspace.
`memory_recall` may be `best_effort` (default), `required`, or `off`; it does not disable
LangGraph persistence or authorize a different provider.
