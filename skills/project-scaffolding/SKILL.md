---
name: project-scaffolding
description: Create a project or add a new application/package using an explicitly selected stack and its native generator while preserving existing repository work. Use for greenfield setup and scoped scaffolds, not as a reason to rewrite an established application.
---

# Project scaffolding

Inventory the destination first: instructions, tracked/untracked files, manifests,
lockfiles, workspace layout, CI and installed runtime/package manager. Determine
whether this is a new repository, package or feature. Follow established stack
choices; if there is no selected stack and the choice materially affects the
result, resolve it before generating. Read
[the staging recipe](references/staged-generator.md).

Use the explicitly selected stack's native generator when available. Verify its
installed version, help and side effects before execution. Do not invoke an
on-demand downloader as if it were an installed generator. Dependency
installation, hooks, Git initialization and external services need their own
existing authorization; selection of a framework does not grant it.

Generate into an empty staging directory when working near existing content.
Use supported flags to omit unrequested installs, Git, hooks and telemetry;
if the generator cannot separate these actions, stop that path and use an
available non-installing route. Do not guess flags or replace the selected
stack with a homegrown framework to bypass missing tooling.

Review the generated tree before copying: scripts, versions, lockfile, template
credentials, ignored artifacts, public assets and unnecessary examples. Merge
only reviewed paths into the target. Preserve its lockfile/package-manager
choice, workspace registrations and user edits. A conflicting file needs a
content-aware merge, not force generation.

Prove the intended entry point starts or builds with available dependencies;
exercise one real behavior and an invalid-input or failure path. Run the
project's applicable checks. Distinguish generated files from a runnable setup
when dependencies are unavailable. Record the generator/version/options and
remaining setup without claiming unrun gates passed.
