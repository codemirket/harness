# Harness inventory extraction

This standard-library Python 3.11+ utility extracts the repository's declared
expertise, dependency and tool relationships for the [harness critique map](../../docs/harness-map.md).
It is evaluation preparation, not a runtime tool or installer.

From the repository root:

```sh
python3.13 evals/harness-map/extract.py
```

Outputs are [inventory.json](../../docs/harness-map/inventory.json) and
[inventory.md](../../docs/harness-map/inventory.md). The JSON retains source
revision, selected input hashes, complete registry contracts, resolved Cargo/npm
records, Python wheel pins, reference hashes and typed relationship edges.
Repeated extraction of unchanged inputs is deterministic. The source revision
identifies HEAD; input hashes identify the selected working-tree bytes, including
uncommitted changes. This is not an installed SBOM or a dependency freshness scan.

Review and update the main map separately when execution code changes. Registry
facts cannot establish runtime readiness, permission, semantic correctness or
quality. Extraction never imports the application, reads private task state,
starts services, installs dependencies or performs inference.
