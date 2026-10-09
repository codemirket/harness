# Reproduce the assurance exercises

These standard-library fixtures exercise specific failure boundaries. They are
small fictional systems, not production libraries, scanners or proof of general
expertise. No dependencies or authenticated services are required. Run from the
harness root with Python 3.9+ and ordinary assertions enabled (do not use `-O`).

```sh
mkdir -p build/assurance
python3 examples/assurance/engineering.py --output build/assurance/engineering.json
python3 examples/assurance/systems.py --output build/assurance/systems.json
python3 examples/assurance/seo.py --output build/assurance/seo
```

Output paths must be new; use a new name for another run. Temporary databases and
the loopback HTTP server are cleaned up. Inspect raw results, not just the exit code.

| Exercise | Observable evidence | Scope limit |
| --- | --- | --- |
| Engineering | Durable replay, conflicting key, tenant denial, rollback, concurrent debit/duplicate checks; indexed-query parity, raw read/write times and storage costs | Trusted callable context and local SQLite; no HTTP authentication, power-loss test or production throughput |
| Systems | Preserve existing files, execute a staged entry, reject copy conflicts, model bounded queue admission/recovery and detect changed plan inputs | Handwritten fixture, discrete queue and synthetic fingerprints; no native generator, real broker or IaC apply |
| SEO | Raw HTTP bodies/headers for eight local URLs; noindex/robots/canonical distinctions; reject a deliberately wrong classifier and observe a changed response | Fixture-specific robots parser, no JS rendering, search-console access, Google crawl or ranking evidence |

Systems' linked-ancestor probe needs OS symlink support; its report names a skipped
probe if the host cannot create one. The other checks still run. The engineering
benchmark has no speed threshold: uncontrolled host timing is measurement, not a
stable CI gate. The important hard condition is identical results.

Use `python3 ai.py eval list` for separate task workspaces and qualitative rubrics
covering frontend, documentation, integration, research, analysis and database
work. Use [the workbench](../../docs/workbench.md) to render actual artifacts, and
[the capability contracts](../../docs/capabilities.md) to select the next relevant
failure probe. A synthetic pass does not replace the target project's own checks.
