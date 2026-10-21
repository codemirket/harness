# Harness critique evidence

These review utilities support the [20 October critique](../../docs/harness-critique-2026-10.md).
They are analysis tools, not production features or runtime dependencies. Use Python 3.11+.

```sh
python3.13 evals/harness-critique/probe.py --binary /absolute/path/to/mirket --output docs/harness-critique/local-probes.json
```

`probe.py` starts one localhost synthetic inference server and uses a disposable
home/project. Native inference targets are explicitly bound to `/usr/bin/false`
(macOS/Linux fixture); provider environment variables are removed from the child.
It performs no external inference or user-project work. Its current assertions cover OpenPencil routing, required-memory refusal, provider-owned
usage and bounded source reads. The dated JSON remains historical evidence; do not
interpret its old Penpot or memory expectations as current defects. Owning test
suites retain the regression checks.

The historical feedback and dependency JSON reports retain their 20 October
scope. Their one-off generators were retired: the feedback probe depended on
superseded function bodies, and the old audit duplicated the maintained
[dependency audit](../dependency-audit/README.md). Use that audit for current
coordinates and advisory policy. Current recall regressions live in
[the agent runtime suite](../../runtime/agent/test_runtime.py) and
[the remediation suite](../harness-remediation). The original generators remain
available in Git history for historical reproduction; do not overwrite the dated
evidence with current runs.
