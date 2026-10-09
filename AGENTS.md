# Working on Harness

This repository supplies shared instructions, reviewed skills and installation
tools for Codex Desktop and Claude Desktop Code. Python 3.9+ and Git are enough
for the core CLI; optional artifact workflows have separate runtime requirements.

## Find the owning source

- Read [CONTRIBUTING.md](CONTRIBUTING.md) for isolated development and review.
- Use the [architecture map](docs/architecture.md) to locate command owners,
  authoritative inputs and generated outputs.
- For installation behavior, start with [adoption](docs/adoption.md) and
  [target coverage](docs/targets.md).
- For skill or catalog changes, read [source review](docs/source-review.md) and
  the affected skill body and references. Preserve provenance and licenses.
- For workflow quality, use [delivery quality](docs/quality-harness.md) and
  the existing [evaluation cases](evaluations/suite.json).
- Keep task progress in the existing task record; `docs/work/` holds retained
  project work. Treat [verification records](docs/verification.md) as dated evidence.

## Preserve the boundaries

- This file governs work on the harness. [instructions/AGENTS.md](instructions/AGENTS.md)
  is the global payload installed into other clients; keep project details here.
- Preserve user changes and unrelated settings. Managed-copy conflicts must remain
  visible; do not overwrite them or weaken hash, path or executable checks.
- Registry selection, source review and payload registration do not grant runtime
  permissions or prove client activation. Do not execute upstream code incidentally.
- A linked installation exposes edits to its source checkout immediately. Use
  disposable homes and projects for installer verification, following CONTRIBUTING.
- Keep deployment, publication, dependencies, accounts and client settings within
  the user's authorization. A repository check must not install or configure them.
- Refresh affected catalog hashes and generated tables after source changes.
  Never rewrite historical evidence as if it tested new bytes.

## Verify the result

Contributor gates require a full source checkout with `tests/`. Exported plugin
`_harness` snapshots provide the runtime and reference docs but omit that test
suite; use a full checkout for contribution work. From the full checkout, run
the focused checks for the changed behavior, then:

```sh
python3 ai.py context doctor --project .
python3 scripts/render_registry.py --check
python3 -m unittest discover -s tests
sh -n setup/macos.sh
git diff --check
```

Run the shell check in a POSIX shell. Follow CONTRIBUTING for local Markdown
checks and isolated installer exercises. Inspect rendered artifacts and exercise
affected examples when applicable; passing structural checks is not acceptance.
Report skipped runtimes, unverified platforms and remaining failures explicitly.
