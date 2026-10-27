---
name: ci-release
description: "Diagnose or change CI, build, packaging or update behavior using exact revisions, logs and recovery evidence. Skip local feature work with passing delivery paths."
---

# CI and release

Bind the failure/change to the exact revision, workflow, target platform, lockfiles and artifact identity. Read the decisive logs and reproduce the relevant step with the established toolchain. Treat cache, network, signing and platform differences as hypotheses to test.

Fix the owning workflow or packaging contract. Do not disable gates, weaken assertions, remove platforms or run upstream installers/hooks incidentally. Distinguish source tests, package assembly, installed execution and actual client activation; one cannot stand in for another.

Exercise a failure/recovery path appropriate to the change: unavailable input, interrupted activation, drift, rollback or stale package identity. Use disposable homes/projects for installer checks and preserve unrelated client configuration. Verify the saved package and real entry point rather than only build output.

Return the exact fix, platform/revision, checks and remaining release gates. Stop when affected delivery behavior passes or a concrete credential/platform prerequisite prevents verification. Publication, signing-account actions and production rollout still require their own authorization.
