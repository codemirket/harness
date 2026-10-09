# Inspect project instructions and preserve task context

Use project instructions as an entry point to the knowledge needed for a task.
Keep architecture, operational procedures and verification details in their owning
documents and link to them. A short map is easier to navigate, but line count alone
does not measure instruction quality.

The harness offers a read-only diagnostic for the project instruction chain used
by the Codex convention. Run it from the harness checkout:

```sh
python3 ai.py context doctor --project /absolute/project
python3 ai.py context doctor --project /absolute/project --cwd /absolute/project/src
```

`--project` is the explicit inspection boundary, and `--cwd` is an existing launch
directory inside it. The default cwd is that boundary. This is a static audit of
the supplied paths, **not the effective context of a running client**. It does not
launch a client, execute instructions, change files, invoke a model, or read home
configuration, credentials, memories or session history.

## What the report tells you

The JSON report identifies selected instruction files along the root-to-cwd path,
shadowed alternatives and the bytes available under the supplied budget. It reports
metadata and diagnostics without printing instruction bodies.

At each directory, selection prefers `AGENTS.override.md`, then `AGENTS.md`, then
any explicitly supplied fallback filenames. Only one file is selected there.
An empty **project** override still shadows `AGENTS.md`; whitespace-only content
does not consume the content budget. Missing guidance and long files are advisory.
Unreadable or unsafe paths and content that cannot fit the budget need attention.

The default project budget is 32,768 bytes, based on the reviewed Codex source.
It is a byte budget, not a token count, and does not include global instructions.
Use your client's configured value when it differs:

```sh
python3 ai.py context doctor --project /absolute/project --max-bytes 65536
python3 ai.py context doctor --project /absolute/project --fallback TEAM_GUIDE.md
```

These flags only change the audit's assumptions; they do not change client
configuration. Prefer moving detail to linked documents before increasing a
budget. Repeat `--fallback` for additional filenames, in priority order. Fallbacks
must be plain filenames, not paths.

The command exits with `0` when there are no errors, `1` when the audit finds an
error, and uses argparse's `2` for invalid command syntax. Warnings remain visible
on successful runs. A successful result does not prove the agent read or followed
the instructions, or that their prose is correct.

## Check the boundary before interpreting it

Actual Codex discovery also depends on its version, trusted-project state, root
markers, fallback settings, environment selection and instruction caches. The
audit does not infer those settings. A nested repository or a worktree may change
the actual root; provide the root and launch directory you intend to inspect.
The client can also receive instructions from global, managed and session sources.

The tool intentionally inspects only the directory chain you name, not every
subdirectory. Repeat it for relevant launch locations. It does not model Claude's
instruction discovery or claim the same loading rules for both clients. The
portable guidance and handoff workflow below can be used with either client.

The static checker inspects at most 256 directories, accepts up to 32 fallback
names and reads at most 1 MiB per selected file. It refuses linked instruction
files and special files; line counts cover only the inspected prefix. These
limits are its own inspection policy, not proof that
Codex uses the same filesystem policy or an operating-system sandbox.

Global overrides are a separate check: a nonempty `AGENTS.override.md` in Codex's
home can mask installed global guidance even when `ai.py check` passes. The
project audit does not read that home. Inspect it deliberately when debugging
discovery, then verify in a fresh client session. The reviewed global loader
falls back after an empty override; the project selector described above does not.

For local Markdown links, headings and fences, use the existing workbench checker
with a new output directory:

```sh
python3 ai.py workbench markdown /absolute/project/AGENTS.md --output /absolute/new-instruction-report
```

Review the actual prose and run the project's checks too. The context audit does
not validate links or architecture, and the Markdown checker does not execute
examples or fetch external links. See [setup](../setup/README.md) for installation
checks and [workbench](workbench.md) for artifact checks.

## Keep enough context to resume correctly

Use the existing task record for complex or interruptible work. Preserve the
objective, constraints, original authorization, changed paths, evidence with its
input revision, decisions, blockers and next useful action. Link to complete
artifacts instead of putting raw logs or credentials into a handoff.

After a restart or compaction, check the working tree and relevant inputs before
reusing a completion claim. An old passing test does not validate changed source;
a worker's summary or a page excerpt does not grant new authority. Use the
[context lifecycle guide](../skills/context-management/references/context-lifecycle.md)
for a compact checkpoint template and a cold-resume exercise.

A fresh conversation reduces transcript sharing. A separate worktree separates
checkouts. Neither alone separates accounts, credentials, ports, databases or
network access. Inspect the actual host's supported boundaries and permissions;
see [agent coordination](../skills/agent-coordination/SKILL.md).

## Source basis and limits

The implementation was checked against [Codex's instruction discovery source](https://github.com/openai/codex/blob/a06545b311fe01e51ce855c7aa5d8da21e9e7aaf/codex-rs/core/src/agents_md.rs)
and the [global instruction provider](https://github.com/openai/codex/blob/a06545b311fe01e51ce855c7aa5d8da21e9e7aaf/codex-rs/codex-home/src/instructions/mod.rs).
Those pinned sources establish behavior for that revision, not the version running
on your device. Consult the [official AGENTS.md guide](https://learn.chatgpt.com/docs/agent-configuration/agents-md)
and inspect the actual client when they differ. The [research record](research/harness-context-2026-10-09.md)
documents the six requested resources, corrections and implementation decisions.
