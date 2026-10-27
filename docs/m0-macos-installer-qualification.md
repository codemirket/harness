# macOS local installer entry point — 2026-10-26

`scripts/install-macos.sh` accepts only an explicitly selected canonical local
release and two exact SHA-256 values on macOS ARM64. It checks the manifest, copies the release
binary to a private temporary directory, checks that executable copy, then calls
the existing `mirket setup` or explicit `mirket update` owner with the selected
user home's `.mirket` path. The script downloads nothing, runs no upstream
helper and has no PATH fallback for Mirket runtimes. It uses fixed macOS system
paths for hashing and bootstrap file operations. The Rust owner still verifies
the full manifest, payload inventory and managed Codex destinations.

The entry point also exposes explicit same-home `--reinstall` and
`--reinstall-reconcile` actions, with the same local source/hash checks. Their
later actual CLI/MCP qualification is recorded in
[the reinstall report](m0-macos-reinstall-qualification.md).

On macOS 26.5.2 ARM64, a disposable skill-enabled local release fixture with
manifest SHA-256 `d723c65e39ffaa80fe0b7ec5190d8d232ed0c20dc5e5e4d45fbb9626e66b1cbf`
and binary SHA-256 `75e557a48ee52559b9811bd6796e1133d5821ed6f0a119e000bc1b6a7f3d4239`
installed into a new private fake user home. The resulting release identity was
`sha256:20c14c04e14d0149bca523352dbf0c648e6946e0c963791583c97cd4435599d6`;
the installed `mirket doctor --json` reported healthy core, launcher, nine
global skills, guidance and MCP registration. This is an isolated CLI path,
not actual Codex Desktop activation. An explicit same-release `--update`
completed and left the skill copies unchanged.

A deliberately wrong executable hash refused before setup/update and left the
existing fixture's config and launcher hashes unchanged. Repeating fresh setup
against the same installed home refused before changing its config or active
release bytes. A wrong manifest hash and a noncanonical source path each
refused before invoking setup. `sh -n` passed. The test fixture is retained under
`/Users/nazmiyilmaz/.codex/task-state/harness-rewrite-implementation/install-macos-fixture.txQkvp5h`;
the original user installation and Codex settings were untouched.

This qualifies the local macOS entry point against one older disposable
skill-enabled source package. It does not authenticate an unpublished release,
qualify the macOS 15 floor, source downloads, Chromium/Raster/video payloads,
Windows, uninstall, or live Codex Desktop activation. Publication and installer
distribution remain separate gates.
