# Codex-native motion model and effort controls

Research date: 2026-10-27. This is a bounded control qualification for the
motion-strategy proposal. It does not install a profile, change the user's
Codex configuration, qualify Desktop GUI spawning, or add an agent router.

The subsequent [HyperFrames feedback pilot](hyperframes-feedback-qualification-20261027.md)
records actual author turns configured with Astra/high and user-authorized
automatic approval review. Its successful exports extend execution evidence;
they do not establish provider identity, creative superiority or native child
model enforcement.

The later [seven-case native qualification](native-coordination-qualification-20261028.md)
adds actual role selection, per-turn override, interruption and controlled-denial
evidence on the same bundled CLI. It retains a stalled worker and a command that
continued after interruption. The config-only results below remain the dated
scope of this earlier probe; Desktop GUI and hard enforcement remain unqualified.

## Executed identity and evidence

The test used only the Desktop-bundled executable:

`/Applications/ChatGPT.app/Contents/Resources/codex-cli/CodexCLI.app/Contents/MacOS/codex`

It reported `codex-cli 0.164.0-alpha.11.1` and SHA-256
`a04752e4d46a4d89e53e6f61c59c5349a638c8ce172ffbf391fd482b4d902a4e`. The PATH
CLI (`/Users/nazmiyilmaz/.nvm/versions/node/v24.18.1/bin/codex`) was not used.
The protocol phase used a fresh temporary `CODEX_HOME`, temporary project,
`--strict-config`, and `thread/start` with `ephemeral: true`. It ran no model
turn and copied no authentication material. The separately identified
unavailable-model follow-up below used two bounded authenticated turns.

Machine evidence is retained at:

`/Users/nazmiyilmaz/.codex/task-state/harness-motion-feedback-20261027/model-controls-final/protocol-controls.json`

The reproducible probe is
`tests/codex-motion-model-controls.mjs`.

## Observed controls

| Case | Result | Evidence and boundary |
|---|---|---|
| Root model and effort | Passed | Session flags `model="gpt-6-astra"` and `model_reasoning_effort="high"` appeared in `config/read` origins and the `thread/start` result returned `model: gpt-6-astra`, `reasoningEffort: high`. |
| Custom native role file | Fixture written, unqualified | Disposable `.codex/agents/motion-reviewer.toml` contained both `model = "gpt-6-astra"` and `model_reasoning_effort = "high"`; its SHA-256 is retained. The parent thread also used Astra/high. This does not prove Codex discovery, TOML acceptance, or native spawn selection. |
| Explicit weaker user selection | Passed | A `thread/start` request with `model: gpt-6-luna` selected Luna despite the Astra/high session defaults. This confirms explicit selection wins over written guidance. The protocol response did not expose a turn effort for this thread-start override, so no effort inheritance claim is made. |
| Unavailable model | Provider rejection observed | `thread/start` accepts an opaque model slug (`mirket-model-that-is-not-real`) and reports it with provider `openai`; two bounded authenticated turns then returned provider error `model_not_found` / HTTP 400: the model is not supported with a ChatGPT account. No fallback turn completed. |
| Permission denial | Configured, unqualified | With `[agents] enabled = false`, `config/read` returned `agents.enabled: false`; no spawn was attempted. This establishes the disabled configuration, not a permission-denial result or hard Desktop behavior. |
| Bundled catalog | Advertised | `debug models --bundled` lists `gpt-6-astra` and `low`, `medium`, `high`, `xhigh`, `max`, and `ultra` efforts. This is a local catalog advertisement, not account entitlement or provider execution evidence. |

## Interpretation

The CLI can bind the root motion lead to Astra/high when the host supplies those
session flags. A native role file can carry the same two pins, and the official
configuration precedence says that an explicit spawn request outranks defaults,
which outrank the parent value; a role file's own model/effort then takes
precedence for that role. The probe directly observed the root binding and an
explicit lower model override, but it did not observe a native child spawn.

The official OpenAI documentation reviewed for this qualification is the
[Subagents guide](https://learn.chatgpt.com/docs/agent-configuration/subagents),
[Hooks guide](https://learn.chatgpt.com/docs/hooks), and
[Advanced Configuration guide](https://learn.chatgpt.com/docs/config-file/config-advanced).
Those documents describe custom-agent files under `~/.codex/agents/` or
project `.codex/agents/`, `[agents]` defaults, and hook schemas. Documentation
and an app-server schema do not qualify the Desktop collaboration surface.

This result does not support a Mirket model router. Codex owns model selection,
native spawning, permissions and approvals. Mirket guidance can recommend
Astra/high and an agent role, but cannot enforce the host's chosen model or
intercept a native spawn.

## Bounded authenticated follow-up

Two actual model turns were used only for the unavailable-model discriminator.
Each used the exact Desktop CLI, a mode-600 copy of the existing auth file in a
fresh temporary `CODEX_HOME`, a disposable project, and a 60-second observation
window. The temporary tree and copied credentials were deleted after each
turn. No model output, source write, or provider secret was retained.

`unavailable-model-turn.json` records the first attempt's terminal error event;
`unavailable-model-turn-2.json` retains the second attempt's safe error fields
and the provider message:

`{"type":"error","status":400,"error":{"type":"invalid_request_error","message":"The 'mirket-model-that-is-not-real' model is not supported when using Codex with a ChatGPT account.","code":"model_not_found"}}`

The CLI also emitted a warning that the unknown model would use fallback
metadata. That metadata fallback did not become a provider execution fallback
in these turns: they failed before a model response. This is execution evidence
for this unavailable slug only; it is not a general fallback-prevention claim,
and it does not prove that an unavailable Astra account would behave
identically.

## Unresolved paths

* Provider identity and entitlement remain unverified. The `openai` provider
  field and configured slug are host observations, not a provider-side receipt.
* Provider entitlement for `gpt-6-astra` remains unverified by this test; the
  bundled catalog and the successful protocol binding do not prove account
  access.
* Custom-role precedence and attempted weaker child overrides need a bounded
  native spawn on the actual Desktop path. The current evidence only validates
  the role file and parent protocol.
* `PreToolUse` and `SubagentStart` hooks remain documentation-supported paths;
  no hook was installed in a disposable home and no GUI coverage was claimed.
* Windows behavior and the waived Desktop GUI check remain untested.
