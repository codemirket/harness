# Security maintenance and promotion

Mirket separates advisory inventory from release admission. A successful report
means the inventory was collected, not that the candidate is safe to promote.
Published versions are immutable; remediation candidates use a new version.

## Runtime boundaries

The design service retains the project workspace. Each browser job copies bounded
inputs into a read-only snapshot, then runs Node and sandboxed Chromium inside
private user, mount, PID, network, IPC and UTS namespaces. Only that job's files
and explicitly referenced compiled tokens are served inside the worker. Scratch
and output are bounded; the process guard owns cancellation and reaping. The
shared service's HTTP listener serves health only. It does not serve project
files to diagram companions.

The outer design container retains read-only rootfs, dropped capabilities,
resource limits and no-new-privileges. Its reviewed seccomp profile permits
chroot for Chromium's nested user namespace. On SELinux hosts it uses the confined
container_engine_t domain and unmasks proc for nested proc mounts. This increases
the trusted parent process's proc visibility; untrusted browser jobs receive their
own private PID/proc view. A failed nested sandbox is an error, never a reason to
launch Chromium without its sandbox.

Kroki and Excalidraw receive no project mounts or host-published ports. The internal
Kroki gateway accepts only the six supported diagram types and health requests,
with bounded request/response bodies and deadlines. The Java backend listens on
container loopback. Secure mode and disabled companion discovery are explicit.
The internal bridge has no external egress. These boundaries do not establish
absence of parser defects; artifact scans and adversarial render checks remain
required.

Owned recipes use digest-pinned bases and dated Debian snapshots. D2 is built
from checksum-verified upstream source with a pinned Go toolchain and reviewed
module lock, retaining upstream licenses. The runtime image contains no Go
compiler. Excalidraw retains upstream parser code and uses reviewed dependency
locks. Temporary scoped overrides of lodash-es and nanoid repair exact pins in
upstream Excalidraw/Chevrotain dependencies; remove them when the parent releases
adopt compatible fixed versions. DOMPurify is updated through its parent range.
Excalidraw starts on the first CPU in its container's allowed affinity set,
matching its existing one-CPU quota and avoiding Chromium thread-pool expansion
against the whole VM. Its 768 MiB and 256-task limits accommodate Chrome 156
and the offline application bundle; the former 350 MiB/128-task envelope
caused page-cache thrashing and thread-creation failures during real renders.
The server has its own locked runtime dependency tree; browser build packages
are retained only in the build stage. The browser bundle records its input
modules for advisory reachability review. Kroki imports patched Jackson 2.22.3
and 3.1.7 BOMs without modifying upstream parser source.
No runtime package manager, compiler, development headers or build cache is
required in the final renderer images.

## Evidence and release gate

Run the native checks and dependency audits through the documented development
workflow. Export owned OCI candidates with `mirket dev image-export --candidate`;
this builds and exports without selecting or activating a generation. Native
rootless Linux ARM64 may run these development checks with an explicitly
registered system Podman binary (preserving its distribution AppArmor profile).
Production managed-controller activation remains limited to macOS ARM64. Retain
export receipts and archives together. The image audit checks archive hashes,
image identities, package inventory and native/browser payload coverage. Image
header packages are not evidence about the running VM kernel.

`mirket dev dist` emits a native SBOM and source-bound provenance predicate.
Trusted GitHub verification runs sign native and OCI artifacts and SBOMs. Local
promotion independently verifies the Sigstore bundles with GitHub CLI against
`codemirket/harness/.github/workflows/check.yml`. Checksums alone do not replace
signature identity verification. Distroless's upstream keyless signature uses
issuer `https://accounts.google.com` and identity
`keyless@distroless.iam.gserviceaccount.com`.

`mirket dev manifest` requires both `--acceptance` and `--security`. The security
bundle has schema 1 and binds the same source SHA-256 as native build receipts.
It records `created_unix`, hashed references to `image_summary`, `image_reports`,
`source_report` and `runtime_inventory`, plus `dispositions` and `artifacts`.
References contain relative `path` and `sha256` fields and cannot escape the
bundle. Evidence older than seven days, missing coverage, mismatched identities,
unsupported runtime versions, and unresolved high/critical or unknown findings
block promotion.

Dispositions match exact artifact, advisory, package, version and package path.
Each needs an owner, reason, function scope, evidence URL and expiry. Only an
evidenced `not_affected` or `vendor_disputed` decision admits a high/critical
finding. A missing fix never admits a reachable defect. The narrowly bounded
`paste` 1.0.15 maintenance exception is informational and expires within 31 days;
it is tied to the registry crate checksum. It must be revisited when stable
upstream tokenizers removes paste. Neither that warning nor release-note claims
justify changing embedding semantics or adopting a prerelease tokenizer.

## Review cadence

Weekly CI collects advisories and proposes browser/Node changes; monthly CI
proposes an integrated refresh. Maintainers review base image digests, Debian
snapshot dates, native artifacts (Go/D2, Typst, ONNX and image/WASM codecs), parent
parser releases and support deadlines together. Proposals must identify exact
before/after pins, owner, affected recipes and verification gates. The intended
response targets are one business day to triage reachable critical issues,
72 hours for containment, and seven days for reachable high fixes or explicit
mitigation. These targets are operating goals, not delivery guarantees.

No proposal automatically activates a runtime or publishes a release. Native
platform checks, actual contained rendering, embedding/recall thresholds,
backup/recovery and performance comparison remain mandatory. Stage installations
in disposable homes. Live Codex activation and rollback require deployment
authorization and are reported separately from predeployment evidence.
