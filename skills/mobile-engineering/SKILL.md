---
name: mobile-engineering
description: Implement, debug or review native and shared-framework mobile apps across lifecycle, navigation, offline state, permissions and platform builds. Use for iOS, Android, React Native or Flutter engineering; use mobile-design for journey/layout decisions and frontend-engineering for ordinary mobile web.
---

# Mobile engineering

Work in the application's actual platform and build chain. Inspect the repository,
resolved dependencies, supported OS versions, app identifiers, build variants and
existing test targets. Discover available SDKs, simulators/emulators and hardware
before promising native verification. A web preview does not establish a native
build; adding a framework is not part of applying this skill.

## Own state across lifecycle boundaries

Separate transient view state, recoverable drafts, durable local operations and
server truth. Decide what survives navigation, rotation/window changes, background
suspension, process recreation and account changes. Use the platform's supported
restoration/storage mechanisms; background execution is bounded and may never run.
Cancellation of an awaiting UI task does not establish cancellation of a server
write. Keep state ownership out of a component that can disappear mid-operation.

For offline or interruptible writes, read
[the interrupted-order example](references/interrupted-order.md). Persist the
user's intent before sending when the workflow requires recovery. Reuse an
operation identity on retries, bind it to the correct account and payload, and
define server deduplication and reconciliation. A retry queue needs ordering,
expiry/conflict handling and storage bounds, not only connectivity detection.
Connectivity status is a hint; the actual request can still fail or succeed late.

## Integrate with the platform contract

- Model deep links, notification taps and normal navigation as inputs to the same
  destination logic. Check cold launch and an already-running app, authentication
  transitions, account ownership and repeated delivery. A link's identifier is
  untrusted; validate it and authorize the resource at the data boundary.
- Request permissions when the feature needs them and preserve a useful denied,
  restricted or later-revoked state. Check declared capabilities and actual OS
  behavior; an app setting or manifest entry does not prove access was granted.
- Use platform-backed credential storage for secrets that must be retained; decide
  backup, access and account lifecycle from the threat model. Ordinary preferences,
  logs, screenshots and bundled config are not a secret store. Do not embed a
  server credential in a distributable app.
- Preserve accessibility semantics and platform navigation when replacing controls
  or gestures. A faster custom touch handler must still provide the intended
  focus, role, action and assistive behavior. Coordinate visual craft with
  `mobile-design` when available.
- Trace startup work, main/UI-thread stalls, list identity and memory growth before
  optimizing. Measure the relevant build mode and device; debug-mode timings and
  a smooth desktop preview do not establish release performance.

## Build, migrate and verify

Use the project's pinned toolchain and existing commands. Distinguish source
compilation, native linking, simulator installation, signed device installation
and store/distribution readiness. Do not request credentials or change signing
identities merely to complete local checks; state a material unavailable stage.

Check local database migrations and client/server compatibility when changing
persisted state or APIs. Installed clients can remain old after the backend is
updated. Test the supported upgrade path and mixed versions; rollback of a binary
does not undo a destructive data migration.

Exercise the affected journey with actual lifecycle events: process recreation,
background/foreground, repeated input, response loss, offline recovery, deep-link
entry or permission revocation as relevant. Confirm durable results with a later
read. Use simulator/emulator evidence for the supported behavior it can show and
physical devices for consequential performance, battery, camera/Bluetooth,
notification, secure-storage or hardware-interaction claims.

Report the build variant, OS/device, input path and observed result. Keep compile,
simulator, hardware, accessibility and distribution evidence distinct. Finish the
authorized implementation and available checks; an unavailable device is a named
verification limit, not proof of success or a reason to stop unrelated work.
