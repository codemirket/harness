# Locked dependency reporting

Run `python3 check.py --output /absolute/evidence/audit.json` with Python 3.11+.
The script queries public OSV metadata for all Cargo registry packages and npm
lock entries, including the OpenPencil transitive closure in the design lock. No
package code is imported or installed; native agent and model artifacts are
covered by image and package receipts.

Source and image audits default to `--advisory-policy report`: findings stay visible
without blocking ordinary changes. Choose fixes for reachability, long-term
stability and performance; do not add rebuilds, overrides or layers solely to
clear advisory headlines. A reachable defect in Mirket's own tool or permission
boundary still needs a fix.

Incomplete/error responses, invalid inventory, malformed review records and
identity mismatches remain errors. `exceptions.json` retains exact coordinates,
reachability reasons, owners, expiry and reviewed source hashes. Expired or
source-stale reviews are reported and cease to suppress findings; they do not
require ceremonial renewal. `--advisory-policy fail` is available for an explicit
strict audit and also rejects stale reviews. Neither policy declares a package safe.

Run `python3 -m unittest discover -s evals/dependency-audit` from the repository
root. Regressions cover reporting, strict mode, malformed/incomplete replies,
expiry, source drift and image identity. CI retains live reports weekly and on changes.
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

CI uses the pinned Linux AMD64 scanner to inspect all three pinned ARM64
design base/service images. The native agent image is scanned separately when a
verified local candidate export is available; it has no retired Python package
inventory. At the maintainer’s request, CI uses `--advisory-policy report`:
image advisories remain in the evidence as non-blocking warnings rather than
requiring costly upstream rebuilds. This does not declare them fixed or safe.
Both command-line audits use the same report policy by default. Missing OS
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
Save each JSON receipt and construct an object keyed by those two aliases,
with the complete receipts as values. Then supply it with `--exports` to a full
scan (omit `--remote-only`). Archive hashes and scanned image IDs must match.
For candidate scans, pass `--candidate-source-sha256` with the compiled source
identity to reject old installed exports or exports from another build.
Owned images require their retained language inventories: Node packages for
design and native artifact/image coverage for the agent. The removed esbuild
payload no longer creates a Go inventory requirement. Any detected Go findings
still appear.
`--only ALIAS ...` supports bounded triage; it cannot establish complete coverage.
The script uses a fresh evidence directory and retains failures. Full coverage
requires all remote and built images and a usable inventory for each.

The scanner detects packages it understands; it cannot prove complete language
coverage, vulnerability exploitability or a complete native/WASM vulnerability coverage.
Inspect missing inventories and upstream binaries separately. An audit pass is
never an instruction to upgrade a database image or open retained data with a
new version. Treat source lock coverage, image identity, scanned inventory and
reachability review as separate facts.
