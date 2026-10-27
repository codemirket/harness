# Codex startup and model discovery qualification

On 2026-10-25, two fresh model turns through the Codex Desktop bundled executable
called the exact installed Python release's `runtime_status` tool with the
default optional-server startup policy. No global grace override, `required`
setting or `startup_readiness` setting was supplied. Both calls returned the
expected release and completed without a tool error. This qualifies that bounded
model-visible read. It does not establish reliable discovery of the full managed
entry or a model-driven Python navigation journey.

No repeatable per-Mirket configuration fix was established. The proposed
`required = true` plus `startup_readiness = "catalog"` combination succeeded once
and failed once under the earlier restrictive prompt, so it was not adopted.
`src/codex_config.rs` and live Codex configuration remain unchanged. Full release
verification was preserved.

## Exact runtime and scope

The installed release was
`sha256:5d8c2d9d1697a8b353926610bf864d5991b2aff046fa81426cfded23489318ba`,
served through its normal stable launcher in the relocated task-owned home:

```text
/Users/nazmiyilmaz/.codex/task-state/harness-rewrite-implementation/Python navigation İ installed 20261025
```

The client was `codex-cli 0.163.0-alpha.11.2`, executable
`/Applications/ChatGPT.app/Contents/Resources/codex-cli/CodexCLI.app/Contents/MacOS/codex`,
SHA-256 `248ff9873b6fb35124a86bdffa013a09e1bd17f106385124e3dda73e9eb157ad`.
The platform was macOS 26.5.2 ARM64. Model probes used a disposable project,
`--strict-config`, `--ignore-user-config`, `--ephemeral`, `--sandbox read-only`
and command-line MCP overrides. They reused the previously authorized local
authentication route without reading or copying credential files. No provider,
model or reasoning override was introduced.

Each row below used a new client process, without an app-server inventory
preflight. The observer forwarded the original protocol bytes to
`bin/mirket serve --home …`. Its Node executable came from the exact active
release. The second successful default-policy run also used a new server name
(`mirket_cold_discovery_20261025a`). This is a fresh connection/identity check,
not a cold-filesystem performance measurement.

## Observed model results

All observed runs below supplied the existing per-server
`startup_timeout_sec = 60` and exposed only `runtime_status`.

| Per-server policy | Prompt | Initialize reply | Catalog round trip | Completed model MCP calls | Entire turn |
| --- | --- | ---: | ---: | ---: | ---: |
| `required = true`, readiness `connection` | Restrictive | 4,041 ms | 6 ms | 0 | 13.817 s |
| `required = true`, readiness `catalog` | Restrictive | 2,381 ms | 6 ms | 1 | 36.022 s |
| Same catalog candidate, fresh process | Restrictive | 2,380 ms | 5 ms | 0 | 10.218 s |
| Default optional policy | Discovery allowed | 2,387 ms | 6 ms | 1 | 21.325 s |
| Default optional policy, new server name | Discovery allowed | 2,375 ms | 6 ms | 1 | 16.521 s |

The restrictive prompt requested exactly one tool call and prohibited every
other tool. The revised prompt explicitly allowed metadata-only tool discovery
while prohibiting shell, source-file and effect tools. Both successful
default-policy turns made exactly one observed MCP call, `runtime_status({})`,
and no other completed tool item appeared in their JSONL output. The returned
structured content equaled the independently parsed text content, identified
the exact installed release, and reported Python navigation configured.
`runtime_status` does not return the exact Jedi/Parso versions, so the model's
answer cannot qualify those versions or useful navigation.

The successful default-policy calls initialized after the documented one-second
optional-server grace. Therefore, exceeding that grace does not by itself prove
that a model cannot subsequently call the tool. The prompt change and model
selection behavior are possible contributors to the earlier unavailable-tool
answers. This is an inference: these CLI traces do not expose an internal
tool-discovery request or the exact model catalog snapshot. The unsuccessful
runs remain evidence, and this small, non-factorial series cannot identify a
single cause or establish general discovery reliability.

## Supported controls and the rejected candidate

The fetched official [MCP documentation](https://learn.chatgpt.com/docs/extend/mcp#other-configuration-options)
distinguishes the per-server startup timeout from the global
`mcp_optional_startup_grace_ms`, whose documented default is 1000 milliseconds.
It documents `required` as a per-server initialization requirement. No
per-server grace-duration field was found in that page or the fetched
[configuration reference](https://learn.chatgpt.com/docs/config-file/config-reference).

The exact installed binary contains `startup_readiness` and the values
`connection` and `catalog`; strict-config native probes accepted both. The
current upstream [configuration schema](https://github.com/openai/codex/blob/970111f6cb62cc02fe36a82ea7d71b37a2889c97/codex-rs/core/config.schema.json)
and [configuration implementation](https://github.com/openai/codex/blob/970111f6cb62cc02fe36a82ea7d71b37a2889c97/codex-rs/config/src/mcp_types.rs)
describe `catalog` as accepting a valid cached catalog while the live connection
starts; tool execution still requires the current connection. That source pin
is a current upstream snapshot, not a verified source revision of the shipped
alpha binary. It does not justify assuming a cache exists on first startup.

A separate failure probe used an unauthenticated disposable `CODEX_HOME`,
`command = "/usr/bin/false"`, `required = true`, connection readiness and a
two-second timeout. The native client exited 1 in 343 ms with the explicit
`required MCP servers failed to initialize` error before a model turn started.
This establishes that this build applies the required-server failure policy;
it does not prove model-visible catalog readiness. The inconsistent candidate
above is insufficient reason to change registration or make all Mirket startup
failures fatal to a Codex session.

## Full managed-entry attempt and remaining gate

One further model attempt used the normal managed-entry shape: server name
`mirket`, the stable launcher as `command`, only `serve --home …` as arguments,
the generated 60-second timeout and the full unfiltered tool inventory. It used
neither the observer nor additional per-server readiness settings. The client
reached `thread.started` and `turn.started`, then reported model HTTP
`429 Too Many Requests` with a roughly 295-second retry delay. The existing
90-second probe deadline terminated that process group. No completed model item
or MCP call was recorded, and no startup interval was measured in this
unobserved transport. This result qualifies neither discovery success nor
discovery failure. It was not retried to wait out the account limit.

The full managed-entry model gate, useful Python model navigation and actual
Desktop GUI activation remain open. The earlier
[native Python app-server qualification](m3-python-codex-client-qualification.md)
continues to establish useful direct calls, rather than a model journey.
Windows and oldest-platform behavior are outside this macOS evidence.

## Retained evidence and final state

Private evidence is retained outside shipped source at
`~/.codex/task-state/harness-rewrite-implementation/codex-startup-discovery-20261025/`:
exact argv/prompts, complete JSONL and stderr, observer timestamps, the failed
required-server fixture, fetched upstream source/schema pins,
`verification-summary.json`, `final-state.json` and the final doctor result.
The probe script evolved during investigation; each saved request record is
the authoritative argv/prompt for its run.

The verifier checked successful result identity/content, completed-call counts,
timings, client exit status and absence of every observed Mirket process.
A final process inspection found no installed `serve` process remaining after
the bounded full-entry attempt. The actual stable launcher then passed
`doctor --json` with the exact active release healthy. Every authenticated probe
checked the live `~/.codex/config.toml` SHA-256 before and after; it remained
unchanged. No production dependency, installation activation, runtime
implementation change, commit, push or publication was performed.
