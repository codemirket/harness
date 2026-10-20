# Critique remediation regressions

These tests complement the owned Rust and contained runtime suites. Run with
Python 3.11+: `python3 -m unittest discover -s evals/harness-remediation`.

`test_feedback.py` executes selected unchanged function bodies from the owning
runtime files with controlled fakes. It proves that an all-empty retriever fails
the frozen baseline, and that late requirements and explicit recall queries are
preserved. It does not measure embedding quality. The separate contained runtime
suite runs actual Mem0/FastEmbed retrieval; the standard Rust suite exercises
synthetic provider calls and actual worker file/verification behavior.

The original dated critique probes intentionally reproduce the old defects.
They remain historical evidence and should not be relabeled as passing regression
checks after a fix.

## Bounded startup sampling

Run against an immutable executable outside Cargo's build output:

```sh
python3 evals/harness-remediation/startup_probe.py /absolute/candidate --output /absolute/new-result.json --concurrency 1 4 8 16
```

The standard-library probe uses disposable homes, initializes MCP with the
unchanged 15-second deadline, then sends an initialized notification and ping
with a five-second deadline. It captures phase timings and executable hashes
before and after sampling, refuses to overwrite an existing result, and exits
nonzero for a failed sample. Each group is bounded to 1–16 processes, with at
most four groups. It does not run model inference. Results describe these
local samples, not an SLA or the cause of an earlier intermittent failure.

## Native platform receipts

`native_validate.py --dist /absolute/dist --output /absolute/new-evidence`
checks the current host target, executable/checksum/source binding, all four
installer variants, managed updates, doctor and actual MCP SDK exchanges.
It keeps stdout/stderr plus a hashed receipt on both success and failure.
CI runs it separately on Windows, Linux and macOS; one platform's receipt
cannot establish another platform's behavior.

For native Linux ARM64 process checks on the already selected macOS Podman
engine, `mirket dev linux-check --project . --engine-home /absolute/engine-home
--output /absolute/new-evidence` copies only public source inputs into an
isolated compiler container. It uses a digest-pinned Rust 1.99.1 image and the
repository toolchain, builds a separate bootstrap executable, and runs the full
Rust gate, catalog, disposable setup/doctor and MCP checks. The source mount is
read-only and excludes the user home, credentials, engine socket and private
task state. Debug symbols and incremental caches are disabled for this disposable
build to limit shared VM disk use; debug assertions and every required test remain
enabled. The receipt records those settings, and the log records disk capacity.
It does not establish Windows, x86 Linux, GUI activation or support
for a Linux package controller.
