---
name: desktop-integration
description: "Build and audit desktop application boundaries: native lifecycle, IPC, filesystem access, background work, updates, signing, and packaging for Electron, Tauri, or native shells."
---

# Desktop integration

Identify the target OS versions, framework and version, packaging/distribution
channel, sandbox model, and existing app lifecycle. Use current official framework
and platform documentation for version-specific APIs and signing requirements.

## Establish process and trust boundaries

For Electron, map main, preload, renderer, worker, and utility processes. Expose
narrow typed preload methods; validate IPC sender, arguments, resource ownership,
and authorization in the privileged process. Avoid exposing raw filesystem, shell,
or arbitrary IPC primitives to rendered content. Preserve context isolation and
sandboxing where supported; do not solve feature gaps by disabling protections.

For Tauri, map webview commands, Rust state, capabilities/permissions, sidecars,
and event channels. Scope commands and resource access to the feature. Treat local
HTML and remote URLs as different trust domains. Check navigation and window-open
behavior before adding external content.

For native apps, map UI-thread ownership, worker tasks, window/scene lifecycle,
permissions, entitlements, and inter-process interfaces. UI responsiveness requires
measuring the actual work path, not merely declaring functions asynchronous.

## Handle desktop behavior explicitly

- Launch: single/multiple instance behavior, restoration, deep links, file-open
  events, protocol handlers, and startup failures. Validate untrusted URLs/paths.
- Windows: focus, activation, close versus quit, multiple displays, scaling,
  minimize/tray behavior, keyboard commands, and state restoration.
- Files: dialogs/bookmarks, permissions, atomic saves, concurrent writers,
  non-ASCII paths, case sensitivity, and recovery from partial writes.
- Background work: cancellation, progress, sleep/wake, network changes, child
  process cleanup, and safe handoff when a window closes.
- Accessibility: keyboard navigation, names/roles, focus recovery, screen readers,
  reduced motion, contrast, and platform text scaling.

## Package and operate

Verify packaged behavior separately from development mode. Check bundled assets,
resource paths, native modules/sidecars, target architectures, and application data
locations. Signing, notarization, entitlements, update signatures, and release
channels are platform-specific requirements; keep signing material outside source.

Design update compatibility around persisted data and migrations. A binary rollback
may not restore an older data schema. Preserve user data and provide recovery for
interrupted updates. Never weaken signature or TLS checks to make updating work.

Build, launch, exercise changed interactions, and inspect logs on target platforms
when available. A successful build or process spawn does not prove a visible,
responsive application. State unsupported platform verification explicitly.
