# Locked dependency admission

Run `python3 check.py --output /absolute/evidence/audit.json` with Python 3.11+.
The script queries public OSV metadata for all Cargo registry packages, npm lock
entries, pinned Python wheels and every package key in both documents of the
OpenPencil transitive closure in the design npm lock. No package code is imported or installed.

The check rejects untriaged advisories, partial/error responses, expired exceptions
and changes to a reviewed source file. `exceptions.json` records exact coordinates,
advisories, maintainer ownership, expiry and source hashes. Re-review reachability
before updating a hash; never renew an exception just to make CI green.

Run `python3 -m unittest discover -s evals/dependency-audit` from the repository
root. The regressions cover new findings, incomplete/error replies, expiry and
source drift. CI retains its live report and checks weekly as well as on changes.
OpenPencil's CLI, engine, codecs, aliased Yoga package and CanvasKit are captured
in the design npm lock. OSV checks every nested package by its actual package
name/version; native and WASM payloads still need image inventory and manual
coverage review.

## Images and operating-system packages

`image_audit.py` runs a separately selected Trivy executable, recording its
digest, version/database metadata, raw reports and a summary. No containers are
started and no runtime versions or service data are changed. The development
scanner pin in `scanner.json` can be fetched without an installer:

```sh
python3 evals/dependency-audit/fetch_scanner.py --target darwin-arm64 --output /absolute/new-trivy
python3 evals/dependency-audit/image_audit.py --scanner /absolute/new-trivy --remote-only --output /absolute/new-image-evidence
```

CI uses the pinned Linux AMD64 scanner to inspect all ten pinned ARM64
base/service images. Every detected advisory, including an unfixed advisory,
fails that scoped gate. There are no automatic image exceptions. Missing OS
inventory, an unsupported/end-of-life OS, scanner errors or identity mismatches
cannot pass. Language findings remain visible even when OS coverage is missing.
Private per-run scanner caches avoid lock collisions between concurrent scans.

Remote-image checks explicitly exclude locally built layers. To inspect those,
export the installed, verified `design-tools` and `agent-runtime`
images through the candidate CLI:

```sh
mirket dev image-export --engine-home /absolute/engine-home --image design-tools --output /absolute/new-design.tar
```

Use `--candidate` to build an immutable reviewed image without activating design
services. Agent candidates prepare their immutable runtime generation receipt.
Save each JSON receipt and construct an object keyed by those four aliases,
with the complete receipts as values. Then supply it with `--exports` to a full
scan (omit `--remote-only`). Archive hashes and scanned image IDs must match.
For candidate scans, pass `--candidate-source-sha256` with the compiled source
identity to reject old installed exports or exports from another build.
Owned images also require their known language inventories. In particular,
an esbuild npm entry without Go binary metadata cannot claim complete coverage.
`--only ALIAS ...` supports bounded triage; it cannot establish complete coverage.
The script uses a fresh evidence directory and retains failures. Full coverage
requires all remote and built images and a usable inventory for each.

The scanner detects packages it understands; it cannot prove complete language
coverage, vulnerability exploitability or a complete native/WASM vulnerability coverage.
Inspect missing inventories and upstream binaries separately. An audit pass is
never an instruction to upgrade a database image or open retained data with a
new version. Treat source lock coverage, image identity, scanned inventory and
reachability review as separate facts.
