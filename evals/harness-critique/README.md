# Harness critique evidence

These review utilities support the [20 October critique](../../docs/harness-critique-2026-10.md).
They are analysis tools, not production features or runtime dependencies. Use Python 3.11+.

```sh
python3.13 evals/harness-critique/probe.py --binary /absolute/path/to/mirket --output docs/harness-critique/local-probes.json
python3.13 evals/harness-critique/feedback-probe.py
python3.13 evals/harness-critique/dependency-audit.py
```

`probe.py` starts one localhost synthetic inference server and uses a disposable
home/project. Native inference targets are explicitly bound to `/usr/bin/false`
(macOS/Linux fixture); provider environment variables are removed from the child.
It performs no external inference or user-project work. Its current assertions cover OpenPencil routing, required-memory refusal, provider-owned
usage and bounded source reads. The dated JSON remains historical evidence; do not
interpret its old Penpot or memory expectations as current defects. Owning test
suites retain the regression checks.

`feedback-probe.py` executes exact selected owned function bodies with controlled
memory/tokenizer fakes. It demonstrates a test-oracle gap and objective-prefix
collision. It does not test actual embeddings, Mem0 or container behavior. The
existing separate English memory retrieval assertion is not bypassed or assessed
by this isolated baseline probe.

`dependency-audit.py` sends public package names/versions to OSV and queries
crates.io, npm and PyPI for direct dependency metadata. It downloads metadata only,
never installs packages or invokes their code. Outputs replace the dated JSON
snapshot; compare provenance before interpreting a later run. Advisory matches
need separate reachability analysis. This does not inventory OS packages, actual
container images or the complete runtime dependency closure.

The recorded review date is 2026-10-20. Update that field for later investigations.
Current evidence is linked from the critique. Preserve negative results rather
than changing assertions to make an implementation appear correct.
