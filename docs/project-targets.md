# Project selections

Project setup supports `codex`, `claude`, and `all`. The default `all` selects
Codex Desktop and Claude Desktop Code. Manifests and locks use `codex` and
`claude`; Codex project skills live in `.agents/skills`, and Claude project skills
live in `.claude/skills`. See [target coverage](targets.md) for client boundaries.

## One declaration

Schema version `1` supports shared skills and provider-specific additions:

```json
{
  "schema_version": 1,
  "targets": ["codex", "claude"],
  "profiles": ["project-foundation"],
  "skills": [],
  "skip": [],
  "target_skills": {
    "claude": ["matt-git-guardrails-claude-code"]
  }
}
```

This installs the four complementary foundation skills for each client and the
guardrails guidance for Claude only. Registering that skill does not configure or
execute its hook. The declaration is project-owned; preserve its choices when
adding expertise.

Create a new declaration with:

```sh
python3 ai.py project init --project /absolute/project --target all --target-skill claude:matt-git-guardrails-claude-code
```

Init refuses to overwrite a differing declaration. To extend an existing project:

```sh
python3 ai.py project add --project /absolute/project --capability "CFO" --dry-run
python3 ai.py project add --project /absolute/project --capability "CFO"
python3 ai.py project plan --project /absolute/project
python3 ai.py project sync --project /absolute/project
python3 ai.py project doctor --project /absolute/project
```

Use `--skill ID`, `--profile ID`, or repeated `--target-skill TARGET:ID` for other
reviewed selections. Add changes the declaration, so sync and doctor follow it.
Plan, sync and doctor retain the existing declaration; init and add apply the
configured foundation. Choose specialists for concrete project needs.

## Resolution

- Shared profiles and skill IDs apply to every declared target. `target_skills`
  maps active canonical target names to additional catalog IDs. Unknown and
  inactive targets fail before writes.
- Scoped IDs are additive. To select a skill for only one client, put it under
  that target and remove it from shared profiles or skills. An unsupported shared
  selection fails; the installer never silently drops a requested provider.
- `skip` excludes shared choices. A scoped ID also listed in `skip` is an error,
  and a required companion cannot be skipped. Companions inherit their owner's
  target and must support it.
- Conflicts and duplicate installation names are checked against each target's
  complete resolved selection, including shared entries and companions.
- Schema version `1` locks contain source and adapted hashes, provenance and a
  `targets` list on every resolved skill. Doctor rejects a stale manifest, hash
  or target assignment. Generate locks through sync; do not edit them by hand.
- Plan, sync and doctor explain omitted active targets through `excluded_targets`.
  The reason is `unsupported` when the catalog forbids a provider, or
  `not_requested` when the declaration does not select it there. These rows do
  not create installation work or permit a requested target to be discarded.

Source pins, companions, payload hashes and executable declarations are checked
before publication. A matching intact one-off catalog installation can be adopted
without rewriting it. Modified or unselected copies are preserved for deliberate
review. Registration proves file state; it does not prove runtime availability or
workflow activation.

## Shared global guidance and discovery

The normal installation combines 14 global workflows with four project foundation
skills. Use `portable-foundation` explicitly when the global workflows are absent.
Do not install overlapping copies by default:
[Codex can list same-name skills](https://learn.chatgpt.com/docs/build-skills#where-codex-loads-local-skills),
while [Claude gives personal skills precedence](https://code.claude.com/docs/en/skills#resolve-skills-that-share-a-name).
A prose instruction cannot override client discovery behavior.

Project commands report `global_skill_name_overlap` when a selection shares a
configured global name. This compares declarations; it does not prove a global
copy is installed or determine which copy a running client selects. Inspect exact
paths and use a fresh session to verify discovery and actual skill use.

Plan, sync and doctor report unselected receipt-bearing copies under active targets
as `kind: diagnostic`, `action: preserved`. Rows include a readable receipt ID and
`global_overlap` when applicable. Unknown or malformed receipts and symlinks are
reported without trusting or traversing them. These diagnostics do not install,
delete, add lock entries or fail doctor by themselves. Selected-copy drift and stale
locks still fail. An unreadable skill in a selected target can block collision
preflight. Removing a declaration does not automatically remove a saved copy.

## Provenance and executable integrity

When installed bytes match reviewed content but source provenance differs, project
commands report both origins. The installation receipt retains its actual origin;
the selection lock records the current catalog. Doctor can accept content equivalence
without claiming that a newer source was fetched. Unchanged copies retain timestamps.

Every selected skill receipt requires `id`, `repository`, `commit`, `path`,
`sha256`, `installed_sha256` and `executable_files`. Local authored payloads record
`repository: "codemirket/harness"` and an explicit `commit: null`; upstream payloads
record their pinned commit. `sha256` identifies the reviewed source payload, while
`installed_sha256` identifies the published payload after any reviewed adaptation.
Both hashes are recorded even when they are equal. `executable_files` is an
explicit list, including `[]` when no helpers are executable. Missing or invalid
required fields block selected-copy reconciliation; they are not inferred from
the current catalog.

On POSIX, ordinary installed files and receipts must have no execute bits;
declared helpers must have all three. An
update validates the saved copy against its own receipt before replacing it, so a
new catalog entry cannot conceal permission edits. Preparation and publication also
check mode drift. The installer preserves drifted copies for review instead of
repairing their permissions automatically.

Windows requires declared helpers to exist as regular files but does not apply
POSIX execute-bit checks. Payload hashes cover paths and bytes; executable declarations
are recorded separately in catalog entries and receipts. These checks do not validate
ACLs, provide a sandbox, or authorize execution.
