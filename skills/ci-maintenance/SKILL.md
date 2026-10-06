---
name: ci-maintenance
description: Diagnose failing CI, flaky tests, build or release pipelines, and review feedback using exact revisions, logs, reproducible failures, and focused fixes.
---

# CI and review maintenance

Identify the repository, base/head revisions, failing run/job, event, platform,
and expected gate. For pull requests from forks, fetch runs and discussions from
the base repository; the fork's branch owner does not own the PR number.

## Establish what failed

Use the existing CI provider's tools and supported structured output. Distinguish
a command that successfully reports failing checks from a command that failed to
retrieve them: inspect its documented exit codes and output together. Retrieve
the complete relevant pages of review threads, check runs, and job logs. Record
pagination limits, pending checks, external checks, and inaccessible logs.

Find the first causal failure rather than the last cascading error. Compare the
exact revision, dependency lockfile, runner image, environment, secrets access,
permissions, services, timeouts, caches, and generated artifacts with passing
runs. Redact credentials and private payloads before quoting logs.

## Repair at the appropriate layer

Reproduce with the project's command and environment when possible. A local pass
does not disprove a runner-specific problem. For flakiness, investigate shared
state, races, timing, order, resource contention, external dependencies, and seeds.
Use bounded repeated runs to gather evidence; retries alone do not fix the cause.

Preserve required security checks, coverage thresholds, and deployment gates.
Do not make a pipeline green by ignoring its failure or running untrusted fork
code with privileged tokens. Review workflow triggers, token scope, action pinning,
artifact trust, and cache-key boundaries when changing the pipeline itself.

For review feedback, verify each finding against current code and the requested
behavior. Implement justified fixes, explain disagreements with evidence, and
keep unrelated refactors separate. Posting replies, resolving threads, committing,
or pushing follows existing user authorization.

## Verify the current revision

Run focused checks after fixes, then required project gates. Associate results
with the tested revision and inputs. If remote CI requires a push that is not yet
authorized, report local evidence and the pending remote gate. A previous run or
a newly queued run does not verify the final changes.
