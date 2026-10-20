# Harness completion evidence — 20 October 2026

The remaining local implementation and verification work from the
[recovery follow-up](harness-recovery-2026-10.md) is complete. The accepted
macOS ARM64 candidate is installed, and direct calls after the user's reconnect
confirm that Codex and its design connection use the expected generation.
Changes remain uncommitted; no release was published.

The [verification manifest](harness-completion/checks.json) binds the saved
evidence. The earlier reports remain dated records of what was known then.
This report supersedes their pending local acceptance, deployment and Codex
reconnection status; it does not turn historical failures into passes.

## Accepted candidate and active connections

| Identity | SHA-256 |
| --- | --- |
| Compiled source | `231b49a6f81ba6fb291aa37ab68cc45d310291fb2522fd5bd376a8e018f7af49` |
| Optimized executable | `e10197e183c93c07e760b85b05d08aee41d30ad892a46d3e232adb21c5bb1aaa` |
| Live design runtime | `7924f01d8c8406459abf2eeb820263a15f936b52486cb24d7e3109d8514eecbc` |

The version remains 1.0.11, so version text alone cannot distinguish this
candidate. [Contained acceptance](harness-completion/acceptance.json) binds the
exact executable and source. [Local manifest assembly](harness-completion/promotion.json)
accepted that receipt. All four packaged executables passed fresh disposable
setup, doctor, managed update and doctor checks.
[Installer evidence](harness-completion/installers.json).

The local update retained configuration and design data. Doctor and package
status passed afterward. A private pre-update backup and the previous executable
are retained under the user's Mirket backup directory; private state is not
included in this evidence directory.
[Update](harness-completion/local-update.json),
[doctor](harness-completion/live-doctor.json),
[packages](harness-completion/live-packages.json).

After reconnection, the actual Codex MCP process, PID 15507, reported both hashes
above and `installed_binary.matches_process: true`. The direct design token
operation succeeded on Node v22.23.3. All 15 reported design module/lock hashes
match their owning checkout files. Its cached CSS, JavaScript, TypeScript,
Typst and Penpot token outputs were read and checked to preserve the supplied
ink color `#17253c`; the Penpot DTCG JSON matches the source.
[Codex observation](harness-completion/desktop-after-reconnect.json),
[design observation](harness-completion/design-after-reconnect.json),
[output validation](harness-completion/reconnect-validation.json).

That design call reused a compiled result: it establishes a working connection,
runtime identity and correct retained token content, not a fresh compilation or
every design tool. The separate design suite supplies broader operation coverage.
The [pre-reconnect observation](harness-completion/desktop-before-reconnect.json)
retains the stale Codex process and closed design transport. The Claude Desktop
connection was not observed; native Claude adapter execution below is a separate
fact.

## Resume and native vision defects

Live native testing exposed two additional defects after the deterministic
resume fixes:

- On macOS, the system temporary directory can be reached through `/var`, a
  symlink. The strict image reader rejected the harness's own vision specimen
  before inference. `vision_probe` now canonicalizes only the owned system
  temporary parent before creating its private directory. User-supplied image
  aliases remain rejected. The regression exercises both the former failure
  and the retained restriction.
- A conformance probe scoped to one model could overwrite the shared model
  discovery cache with that subset. `record_vision` now holds the discovery
  gate and merges only the selected model's observation into the same registry
  generation with unchanged original metadata. It preserves other models and
  refuses stale observations. The live check retained all 20 models and all
  other model metadata while establishing the selected model's vision support.

The implementation is in [execution](../src/router/execution.rs) and
[discovery](../src/router/discovery.rs).
[Path regression](harness-completion/vision-path-regression.log),
[cache regression](harness-completion/vision-cache-regression.log),
[live cache check](harness-completion/native-cache-live.json).

Probe diagnostics now distinguish wrong pixels, malformed actions and native
contract/transport failures. They retain bounded diagnostics and response
digests, not raw provider errors. An exact static whitelist supplies safe native
failure codes. Regression coverage checks both distinctions and non-disclosure
of an error containing a synthetic secret. The original zero-millisecond
pre-inference failure remains recorded.
[Initial failure](harness-completion/native-first.stderr),
[diagnostic failure](harness-completion/native-claude-diagnostic.stderr).

Actual native journeys exercised a blocked worker, a host readiness change and
resumption of the existing run. Codex `gpt-6.1-sol` and Claude
`claude-sonnet-5-5`, both at high effort, completed with all four image quadrants
correct. Known pause credit was 75 seconds and 26 seconds respectively. The
Claude journey also refreshed discovery to unknown vision and required fresh
selected-model conformance before continuing.
[Native resume evidence](harness-completion/native-resume.json).

These journeys used the candidate generations recorded in that file: the Codex
journey preceded the temporary-path, diagnostic and cache fixes; the Claude
journey included the path/diagnostic fixes and preceded cache merge. The final
source's debug binary separately passed the live cache check, and the final
optimized binary passed complete contained release acceptance. These are
complementary checks, not a claim that every native journey ran on the final
optimized executable.

The existing pause contract remains: only a recorded safe worker pause receives
credit, at most 86,400 cumulative seconds; active-time, turn and repair limits
remain intact. Crashes, pending effects and invalid budget resets do not receive
new allowance. Actual SQLite/LangGraph acceptance also covers rejection followed
by valid retry of the same checkpoint.

## Startup measurements

Executable SHA-256 hashing dominated debug startup. The development profile now
optimizes the existing `sha2` dependency at level 3 while retaining application
debug assertions and identity capture before initialization. No dependency or
timeout was added or relaxed.

The initial 13 samples took 3.75–4.32 seconds. The first improved 13 took
0.28–1.50 seconds. The final reusable probe ran 29 successful samples in groups
of 1, 4, 8 and 16 concurrent processes, with initialization at 0.31–1.07 seconds.
Three optimized samples through the official MCP SDK also passed discovery,
resource and rejection checks.
[Before](harness-completion/startup-before.json),
[first improvement](harness-completion/startup-after.json),
[final debug samples](harness-completion/startup-final-debug.json),
[optimized SDK samples](harness-completion/optimized-startup.json).

The [reusable probe](../evals/harness-remediation/startup_probe.py) retains binary
identity and phase traces, uses disposable homes and preserves the 15-second
initialization deadline. These are bounded local observations, not an SLA.
The original single intermittent 15-second timeout was not reproduced; its
historical cause remains unproven.

## Evaluation evidence and decision

The collector-to-analyzer preflight and immutable failed-attempt records now
have fresh checks. The frozen primary design analyzer is unchanged. This
synthetic round trip remains separate from the model-based pilot below.
[Packet preflight](harness-completion/packet-preflight.json).

A new bounded visual-review pilot compared the existing quality-assurance and
graphic-design roles on four original synthetic dashboard cases: availability
arithmetic, invoice arithmetic, a misleading truncated chart and a healthy
control. Both arms used native `gpt-6.1-sol`, high effort, the same read-only
image/file tools, memory disabled and a 180-second cell budget.

The first eight trials blocked because the host omitted the image filename
while providing no listing tool. All failed returns and traces were retained.
A recorded amendment supplied `screen.png` equally to both arms. All eight
corrected trials completed. Two fresh independent judge calls per case scored
anonymized reviews with randomized labels and counterbalanced order: eight
judge calls, 16 locked review ratings and 64 criterion judgments. The host also
inspected all four images, read all eight reviews and checked the seeded facts.

| Role | Passed cases | Mean score |
| --- | --- | --- |
| Quality assurance | 4/4 | 1.0 |
| Graphic design | 4/4 | 1.0 |

The result is a ceiling tie. **Keep the existing default role; no measured
advantage supports changing it.** Four synthetic cases with one trial per arm
do not establish broad design-production benefit. Workers and judges used the
same model family, so shared bias remains possible. The repaired wave reused
the fixtures; this is not an unseen confirmation after tuning. Native token
usage and cost were unavailable, so latency observations do not establish
cost efficiency.
[Summary](harness-completion/quality-summary.json),
[reviews, judgments and protocol](harness-completion/quality-observations.json),
[corrected-wave record check](harness-completion/quality-check-final.json),
[failed-wave record check](harness-completion/quality-initial-check.json).

The checked raw returns, traces, freezes, score locks and failed serializer
attempt remain in ignored project task evidence under
`.mirket/implementation/harness-completion-2026-10-20`. Public evidence contains
the original synthetic fixtures and review observations. A schema checker
establishes record consistency, not semantic correctness or human approval.

## Final checks and remaining limits

| Check | Result |
| --- | --- |
| Rust, formatting and Clippy | 407 tests passed, zero failed; seven opt-in tests ignored |
| Reviewed Node 22 design runtime, including Chromium | 81 tests passed |
| Candidate-bound contained acceptance | Passed: real agent runtime, desktop contracts, parallel runtime, design output and backup recovery |
| Packaged executables | All four passed isolated setup/update/doctor |
| Python advisory/feedback and packet/tile tests | 5 and 20 passed; packet preflight passed |
| Dependency advisory triage | 679 coordinates; zero untriaged; three existing bounded ZIP exceptions |
| Reconnected Codex and design | Expected identities and token content verified |

See the [Rust log](harness-completion/rust-check-final.log),
[design result](harness-completion/design-check.json),
[Python log](harness-completion/python-checks.log) and
[dependency audit](harness-completion/dependency-audit.json).
Initial failures remain visible, including the unavailable PIL in the default
Python, the noncanonical `/tmp` acceptance output path and rejected initial
evaluation record serialization. Existing bundled Python and a canonical output
path resolved the first two; corrected serialization did not change trial
inputs, judgments or verdicts.

Local native coverage is macOS ARM64; contained runtime coverage is Linux ARM64.
Native Windows/Linux CI and a live Claude Desktop connection remain untested.
The advisory check excludes OS/image packages and the complete upstream Penpot
dependency closure. Native usage accounting, calibrated routing/cost studies and
repository search remain separate product work. The historical timeout cause
and broad quality benefit remain unresolved evidence questions. No required gate,
threshold or timeout was lowered, and no remote CI, commit, push or publication
is claimed.
