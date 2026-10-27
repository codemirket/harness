---
name: native-desktop
description: "Implement macOS or Windows desktop integration when permissions, application lifecycle and installed GUI behavior must be verified natively. Skip browser-only interfaces."
---

# Native desktop

Identify the exact supported OS/architecture, app identity, installation topology and native integration owner. Verify relevant platform API behavior against the current SDK/docs. Keep permissions, trust prompts, process ownership and user settings explicit.

Implement through the project's native toolchain and established lifecycle. Handle startup, reconnect, close, update and recovery at the owning component. Do not treat a cross-compiled binary or source-level portability as evidence that native GUI behavior works.

Build and test on each affected native platform. Inspect the installed application's actual user journey, permissions and child cleanup, including a denied/unavailable integration case. Preserve unrelated settings and use disposable installation/account state where available. Record signing/package and client activation separately.

Return platform/runtime identity, installed behavior evidence and untested targets. Stop at an unavailable required platform/API or permission boundary and report the exact gap. Do not claim Windows behavior from macOS tests or infer user approval from a trust dialog's presence.
