# Harness inventory extraction

This standard-library Python 3.11+ utility extracts the repository's declared
expertise, dependency and tool relationships for the [harness critique map](../../docs/harness-map.md).
It is evaluation preparation, not a runtime tool or installer.

From the repository root:

```sh
python3.13 evals/harness-map/extract.py
python3.13 evals/harness-map/extract.py --check
```

Outputs are [inventory.json](../../docs/harness-map/inventory.json),
[inventory.md](../../docs/harness-map/inventory.md),
[methodologies.md](../../docs/harness-map/methodologies.md) and
[dependencies.md](../../docs/harness-map/dependencies.md). Schema 3 retains exact
source/input hashes, complete registry contracts, resolved Cargo/npm records,
Python wheel pins, reference hashes and typed relationship edges.

Output depends on input bytes, not Git HEAD, branch or staging state. A commit or
branch rename cannot invalidate an unchanged inventory, and extraction also works
from a source archive without Git metadata. The content hashes still detect actual
drift. Git history belongs in the implementation record. `--check` compares all four
outputs without writing and runs in CI. This is not an installed SBOM or a
dependency freshness scan.

Review and update the main map separately when execution code changes. Registry
facts cannot establish runtime readiness, permission, semantic correctness or
quality. Extraction never imports the application, reads private task state,
starts services, installs dependencies or performs inference.
