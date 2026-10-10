---
name: it-operations
description: Diagnose workstation, operating-system, network, DNS/TLS, access and storage problems for a specific user or service. Use for IT troubleshooting and reversible repairs; use infrastructure-engineering for desired infrastructure configuration and systems-engineering for service topology/capacity design.
---

# IT operations

Resolve the affected user's actual service path. Identify the exact machine,
OS/build, account context, application/process, network/VPN/proxy and target
hostname/port from available evidence. For remote or virtualized work, distinguish
the host, guest, container and session. A successful command on the wrong machine
does not diagnose the reported failure.

## Locate the failing boundary

Capture the operation, time, exact non-secret error and last known working case.
Separate name resolution, routing/TCP reachability, TLS identity/trust, application
authentication, authorization and the requested operation. A response at an earlier
layer does not prove a later layer works: ping is not service health, valid TLS is
not access, and a login page is not the user's completed task.

Compare a working and failing path while controlling the target, identity and
time. A browser may use a proxy, cache, DNS mechanism or trust store different
from the failing application. A shell opened as an administrator may use different
environment, mounts and permissions. Read
[the worked path diagnosis](references/service-path.md) for a concrete comparison.

Use the narrowest available read-only probe that distinguishes plausible causes.
Check effective runtime configuration, not only a settings file. Relevant signals
include resolver answers/TTL, selected route, listener owner, certificate hostname
and validity, system time, identity/ACL, service logs and resource pressure. Avoid
bulk environment dumps or diagnostic bundles that expose secrets unnecessarily.

For storage, distinguish capacity, file/inode limits where relevant, mount state,
permissions, quotas and file locks. Identify the writer and data owner before
cleanup. Free space alone cannot establish writable storage or recoverability.
For access failures, distinguish expired authentication from a missing permission;
repeated login or broad privilege grants may conceal the actual defect.

## Repair with a reversible experiment

Choose a change that tests the supported explanation. Capture the specific prior
state needed to restore it; change one relevant boundary where practical. Use the
OS/provider's supported configuration owner so a local edit is not immediately
overwritten. Preserve established project and production authorization; reuse
permission already given rather than prompting on every diagnostic step.

A restart can restore service but erase evidence and does not identify root cause
by itself. Use it when proportionate to the task, then verify the hypothesis or
label the result as mitigation. Do not turn troubleshooting into broad cache
deletion, permission resets, security disabling, account changes or dependency
installation without a concrete need and the applicable authorization.

An ambiguous mutating request may already have succeeded. Inspect its operation
state before retrying, particularly for device enrollment, account provisioning,
data transfer and service actions. Keep credentials in existing trusted channels;
diagnosis rarely requires the user to paste a secret into chat.

## Verify the user's effect

Repeat the original operation from the affected machine, application and identity.
Check a later read or reconnect when persistence matters, and remove temporary
overrides or instrumentation. Compare relevant before/after evidence; do not claim
causation from unrelated changes made at the same time.

Report the observed cause or narrowed boundary, the repair or mitigation, the
exact environment tested and the user's resulting service state. State remaining
access/device evidence gaps plainly. A changed setting, running process or remote
success log is not sufficient when the affected user's operation is still untested.
