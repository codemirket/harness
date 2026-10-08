# Claude Code second opinions

Use a bounded question whose answer can materially improve the primary agent's
work. Good assignments include finding a concrete correctness defect in named
files, comparing two designs against supplied constraints, or checking whether a
conclusion follows from supplied research. Claude is an independent reviewer;
the primary ChatGPT/Codex agent retains implementation and integration ownership.

## Prepare and preview

Write a UTF-8 prompt containing the outcome, allowed project paths, relevant user
instructions, evidence already available, and expected response. Supply only the
needed context; no parent conversation is inherited. A prompt might ask:

> Review the retry implementation in src/client.ts and its existing tests. Find
> reproducible cases where a non-idempotent request could repeat. Return file:line
> evidence, the triggering input, and uncertainty. Do not edit or run anything.

From the harness checkout:

```sh
python3 ai.py delegate claude --project /absolute/project --prompt-file /absolute/review.txt --plan
python3 ai.py delegate claude --project /absolute/project --prompt-file /absolute/review.txt --timeout 120 --output /absolute/new-result.json
```

From an installed skill-catalog directory, use `python3 scripts/harness.py` in
place of `python3 ai.py`. Windows can use the available Python 3.9+ command.
`--prompt-file -` accepts stdin. Select `--model` only for an explicit task need or
user preference; otherwise the CLI chooses its default. No automatic retry or
fallback to another provider occurs.

When requested, `--effort low|medium|high|xhigh|max` passes an explicit reasoning
effort to Claude Code; for example, add `--model claude-opus-5-5 --effort max` for
a review requesting that model at maximum effort. Omit `--effort` to preserve the
CLI default. The runner validates the value and checks CLI flag availability
before a model request, failing rather than silently dropping the selection.
Model/provider support is still enforced by Claude Code; an unsupported effort
or failed response does not trigger a lower-effort retry or model fallback.

Preview probes CLI help and its official authentication status, then prints argv
without the prompt. It does not request a model response or write a result file.
An actual invocation sends the assignment and any files Claude reads to its
configured provider. It creates an output file only after a successful result and
refuses to replace an existing file. Results are staged privately and published
with a no-replace hard link, so output storage must support hard links. Write or
flush failure leaves no partial destination. Treat the report as project data;
do not publish it automatically.

## Runtime and boundaries

The runner requires the official native Claude Code binary on PATH or an explicit
`--executable`. Unsupported safety flags cause an actionable failure. It uses
Claude Code's existing authentication flow; it never opens credential files,
extracts OAuth tokens, logs in, or changes saved settings. `--bare` is unsuitable
for this subscription workflow because the installed CLI disables OAuth in that
mode. [Official authentication](https://code.claude.com/docs/en/authentication)

The plan reports a billing category without keys or identity. API environment,
custom endpoint, cloud-provider, Console or unknown routes require `--allow-api`
before a model request. This acknowledges the existing route; it does not switch
it. Subscription limits and API charges are separate; neither this runner nor a
successful login guarantees free usage or remaining quota.
[Subscription billing](https://support.claude.com/en/articles/11145838-use-claude-code-with-your-pro-or-max-plan)

Only Read, Glob and Grep are available. Shell execution, edits, network/search
tools, MCP, skills and agent spawning are excluded. Requests that would need a
permission prompt are denied. Restricted mode confines built-in file tools to the
project; safe mode suppresses normal customizations. No session is resumed or
persisted. Include needed project conventions in the assignment because ordinary
CLAUDE.md and skill discovery are disabled for this subordinate session.
[CLI controls](https://code.claude.com/docs/en/cli-reference)

The child environment disables automatic and manual CLI updates for every probe
and model invocation; saved settings and authentication stay unchanged.
[Update controls](https://code.claude.com/docs/en/env-vars)

Baseline Read deny rules exclude environment files, secrets/credential directories,
private-key/auth filenames, cloud/SSH settings, Claude/Codex configuration, Git
metadata and Terraform state. Read rules also apply to built-in file search.
The runner imports only positive Read deny rules from standard user and applicable
ancestor/project settings JSON. It does not import hooks, environment values or
allow rules. Settings-relative paths retain their anchor; negative exceptions are
omitted conservatively so they cannot reopen another source's exclusion. Malformed
or symlinked settings cause a failure rather than discarding their protection.
No credential file is read to construct this policy. Nonstandard filenames can
still contain secrets: choose the smallest suitable project root and name allowed
files in the assignment. These exclusions are not a secret detector.
[Read rule semantics](https://code.claude.com/docs/en/permissions)

These controls are not an OS sandbox. Administrator-managed policy still applies,
including managed hooks that local flags cannot disable. Authentication/model
traffic and the CLI's own operational state are not blocked. Use a separately
isolated environment if the project or machine requires stronger boundaries.
The default cannot perform web research; supply excerpts and URLs for analysis,
or let the primary agent retrieve sources with its available tools.
[Managed hook limits](https://code.claude.com/docs/en/hooks#disable-or-remove-hooks)

Timeout, interruption, excessive output, nonzero exit, malformed JSON and a
reported model error are failures, not successful opinions. Cancellation targets
the process group on POSIX and uses Windows taskkill for descendant cleanup;
native Windows behavior still needs device verification. Do not retry a failed
billable request blindly. Inspect the cause and decide whether another attempt is
useful and within the existing authorization.

## Integrate

Read the actual findings and check cited files/claims. Distinguish analysis from
executed tests: this read-only runner cannot execute checks. Resolve disagreement
with evidence rather than voting or automatically accepting the second provider.
Apply justified changes through the primary workflow and rerun affected checks.
