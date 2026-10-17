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
completion; registering or merely assigning a renderer is insufficient. Workflow
floors remain effective even if the caller labels architectural work mechanical.

| Need | Workflow | Owned route |
| --- | --- | --- |
| Supplied evidence and a written answer | `analysis` | Project reads; no edits or external retrieval |
| One mechanical in-memory response | `structured-response` | No tools; strict deterministic operation |
| Code changes / source findings | `source-implementation` / `source-review` | Writes plus exact readback / reads only |
| Existing pixels or live DOM inspection | `visual-review` | Browserless, Odiff and pixel inspection; no source edits |
| Full DOM interface / static Flexbox card | `web-interface` / `static-graphic` | Browserless / Satori-resvg, followed by inspection |
| Layered editable canvas | `editable-interface` | Official connected Penpot MCP and exported evidence |
| Graph layout / encoded motion / paged output | `diagram` / `motion` / `paged-media` | Kroki / HyperFrames / Typst with inspected outputs |
| Synthesis / preparation of rasters | `raster-generation` / `raster-processing` | ComfyUI / imgproxy with inspection |
| Vector hygiene / comparable pixels / tokens | `vector-optimization` / `pixel-comparison` / `design-tokens` | SVGO / Odiff / Style Dictionary |

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

The resolved contract reports both workspace roots. File writes stay inside the
assigned project; design tools receive paths relative to the fixed design
workspace. When authoring design input files, use that workspace or a registered
child as the worker's project so writing and rendering share paths. Source-repo
workers do not gain an extra write root. Request a valid readback tool when
writing files; SVG outputs require exact text readback as well as pixel inspection.

The request has no model ID, provider override or caller-defined tool policy.
`requested_tier` is optional and can raise the floor, never lower it; its values
are `T1_DETERMINISTIC`, `T2_SYNTHESIS`, and `T3_ARCHITECT`. Task intent,
complexity, owned task needs and each assigned tool set the effective floor. The worker can invoke
only its assigned Mirket tools. Current built-in names are `workspace_read`,
`workspace_list`, `workspace_write`, and `inspect_image`; available
`design/...` tools are read from the installed design package and its owned
policy. There is no arbitrary shell tool in this worker interface. A
`verification_tools` entry must also be assigned; completion waits for those
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
their pages or snapshots remain the visual verification targets. Penpot edits
without a target-linked artifact receipt stop incomplete and require external
inspection.

Inspected pixels remain attached across read-only turns and eligible-target
failover, within four images and an aggregate eight MiB. Mirket rechecks their
digests before inference, drops pixel context after a mutation, and records
bounded-context eviction in the trace. A tool action can retain concise visual
observations in its result field; those notes do not replace artifact inspection.

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
Failover can use another discovered authenticated target at the same or higher
tier, as authorized by the routing policy. It does not change credentials or
host settings.
