# Provider-specific project selections

New project setup defaults to both Codex Desktop and Claude Desktop Code. Public
target names are `codex-desktop`, `claude-desktop`, and `all`; desktop aliases
resolve to `codex` and `claude` in manifests and locks. The legacy `codex`,
`claude-code`, `claude`, and `both` names remain compatible; `both` and `all`
select the same two targets. Codex uses `.agents/skills`; Claude uses
`.claude/skills`. See [target coverage](targets.md) for Desktop Code and account
customization boundaries.

Manifests from the former three-client setup must remove `zed` from `targets`
and `target_skills` before reconciliation. Preserve any desired portable skills
by declaring them for Codex, then run plan, sync and doctor to regenerate the lock.
Do not hand-edit the lock or delete `.agents/skills`: Codex uses that directory.
Unsupported targets fail before project writes; existing client settings are not
uninstalled.

A shared selection applies to every target in `.ai/project.json`. Use schema v2
and `target_skills` to add explicitly scoped skills without creating a separate
installation or checker exception:

```json
{
  "schema_version": 2,
  "targets": ["codex", "claude"],
  "profiles": ["project-foundation"],
  "skills": [],
  "skip": [],
  "target_skills": {
    "claude": ["matt-git-guardrails-claude-code"]
  }
}
```

This example installs the four complementary project skills for both clients and the
guardrails payload for Claude Code only. It does not configure or execute its hook. Keep an
existing project's own shared profiles, skills and skip list when adding scoped
entries; the short example is not a replacement for that project's selection.

The normal setup combines 14 global skills with these four project entries.
Use `portable-foundation` explicitly for the full 12 portable fundamentals when
the corresponding globals are absent. Do not install both versions by default:
[Codex can list both same-name skills](https://learn.chatgpt.com/docs/build-skills#where-codex-loads-local-skills),
and [Claude Code gives personal skills precedence](https://code.claude.com/docs/en/skills#resolve-skills-that-share-a-name).
A prose instruction to prefer the project copy cannot override that host behavior.
For old projects, compare exact paths and retain the intended version; sync
preserves unselected old copies, so removing a profile alone does not clean up
duplicate registrations. Do not delete user-modified copies automatically.

Plan, add previews, sync and doctor attach a `global_skill_name_overlap` warning
when a selected project skill shares a configured global name, including through
specialist profiles. This compares configuration; it does not prove the global
copy is installed or determine which copy a client uses. Intentional portable
selections remain valid.

Plan, sync and doctor also append `kind: diagnostic`, `action: preserved` rows
for unselected receipt-bearing copies under active targets. These identify the
path, readable receipt ID and `global_overlap` when applicable. Unknown or malformed
receipts and symlinks are reported without trusting them or traversing linked
copies. Unmanaged ordinary directories without receipts are outside this check.
Diagnostics neither install nor delete files, add lock entries, or fail doctor
by themselves. Selected-copy drift and a stale lock still fail doctor. Review
preserved paths and actual client discovery before deliberate cleanup or reselection.
Existing name-collision preflight also remains strict: an unreadable `SKILL.md`
in a target receiving selected skills can block reconciliation before reporting.
The diagnostic error handling does not bypass that safety check.

For a new project, `project init --target both
--target-skill claude:matt-git-guardrails-claude-code --project /absolute/project`
creates this form. Repeat `--target-skill TARGET:ID` as needed. An existing manifest
can use `project add --project /absolute/project --target-skill
claude:matt-git-guardrails-claude-code` when Claude is already a target. Add preserves
existing choices, includes the configured foundation and upgrades to v2 for scoped
additions. Preview with `--dry-run`, then run sync and doctor. It changes only the
declaration, not installed files or the lock. Init does not overwrite a manifest.

Choose additional skills for current or credible later needs grounded in the
project. The foundation is rich, but unrelated stacks and every catalog entry do
not belong in every project. Existing manifest selections remain unchanged under
plan/sync/doctor; only explicit init/add applies the default foundation. The manual
migration steps below preserve the exact historical selections and counts; using
add also selects the current foundation, so inspect its preview for extra entries.

## Resolution and compatibility

- `target_skills` maps active target names to lists of catalog IDs. `claude-code`
  is accepted as an alias for `claude`; resolved output and locks use `claude`.
  Unknown targets, inactive targets and duplicate aliases are rejected.
- Shared profiles and IDs apply to every declared target. Scoped IDs are additive,
  not overrides. To move a skill to one provider, remove it from the shared
  selection and put it under that provider. An incompatible shared selection
  fails; the installer never silently drops it from a target.
- `skip` retains its shared-selection behavior. A scoped ID also listed in `skip`
  is an error. A required companion cannot be skipped. Companions inherit the
  target of the skill that requires them and must support that target themselves.
- Conflicts and duplicate install names are checked on each target's complete
  resolved selection, including shared entries and companions. Alternatives may
  exist on different targets when they never coexist on the same target.
- Plan, sync and doctor use the same resolved assignments. Schema v2 locks retain
  the source/adapted hashes and provenance fields and add a canonical `targets`
  list to each resolved skill row. Companion rows receive their actual targets
  too. Doctor rejects a stale manifest, hash or target assignment in the lock.
- V2 plan, sync and doctor rows also report `excluded_targets`. Each excluded
  active target has reason `unsupported` when the catalog forbids that provider,
  or `not_requested` when the skill supports it but the manifest does not select
  it there. Shared rows report an empty list. These are explanations, not extra
  installation jobs or permission to discard unsupported explicit requests.
- Existing schema v1 manifests and locks retain their shared behavior and format.
  New scoped manifests use schema v2 so older harness versions reject them instead
  of silently ignoring the scope. Using `target_skills` in schema v1 is an error.
- Sync adopts an intact matching one-off catalog installation without rewriting
  it. Modified copies still block synchronization. Removed or unselected copies
  are preserved; the project owner must deliberately remove obsolete copies.

Use [the installer](../setup/README.md) to reconcile the declaration. Required
files and both source/adapted hashes are checked before publishing changes.
Registration proves file state, not tool availability or workflow activation.

When current reviewed content matches an installed copy but its source provenance
differs, project plan, sync and doctor report both origins. The receipt retains
the original installation; the selection lock tracks the current catalog. Doctor
can pass this content-equivalent state without claiming the newer source was
fetched or reinstalled. Unchanged installed bytes and receipts keep their timestamps.

## Installed executable integrity

On POSIX, installed files must have no execute bits unless declared in
`executable_files`; declared helpers must retain all three execute bits. This
also rejects execute bits added to the installer receipt. Ordinary read/write
permission differences are allowed. Windows does not apply POSIX execute-bit
checks; declared helpers must still exist as regular files.

Current-copy validation and managed updates enforce the same rule. An update
checks the old copy against its receipt's executable contract before replacing it,
so a new catalog revision cannot disguise permission edits. Project preflight and
publication also compare execute bits to detect changes during preparation or
staging. The installer reports drift and preserves the copy instead of repairing
permissions automatically. These checks do not validate ACLs or provide an OS
security sandbox.

Payload hashes remain hashes of paths and bytes; executable declarations stay in
catalog entries and installation receipts. There is no hash or lock migration for
this validation change. On POSIX, an older receipt without executable metadata
cannot authorize any existing execute bits: even a helper still named in today's
catalog requires review of its historical contract. Non-executable legacy copies
remain supported, including updates that add helpers. Do not fill an old receipt
from the current catalog to bypass this check; the current declaration cannot
prove which permissions the previous installation had. Review the old installation
and historical source, then deliberately reconcile or replace it under project
authorization.

## Add Trixpo's previously excluded Claude-only selection

Trixpo can keep its 149 shared IDs, upgrade its manifest to schema v2 and add the
same Claude-only `target_skills` entry shown above. Plan should then report 149
Codex and 150 Claude registrations, with the guardrails row explicitly excluding
Codex as `unsupported`. No hook is activated by that registration.

Before adopting v2 in Trixpo, update its `scripts/agent-skill-lock.mjs` checker and
`scripts/agent-harness.test.mjs` fixtures to accept v2 and derive the expected set
for each provider from lock-row `targets`. Retain its hash, receipt, execute-bit,
missing/extra-copy and provenance checks. Then use project plan/sync/doctor and the
project's own gates. These are later migration instructions; this repair does not
change Trixpo's manifest, checker or installed copies.

## Replace a temporary Claude-only sidecar

These steps replace the `target-extras.json` workaround used by maje-websites.
They are instructions for a later authorized project migration; the harness repair
does not resynchronize that project.

1. Use this updated harness on every device that will reconcile the project.
   In the project's `.ai/project.json`, change `schema_version` to `2`, retain its
   existing shared selection and targets, and add:

   ```json
   "target_skills": {
     "claude": ["matt-git-guardrails-claude-code"]
   }
   ```

   Keep that ID out of shared `skills`, shared profiles and `skip`. Keep the
   existing `.claude/skills/git-guardrails-claude-code/` payload and its receipt.

2. From the harness checkout, run:

   ```sh
   python3 ai.py project plan --project /absolute/project
   python3 ai.py project sync --project /absolute/project
   python3 ai.py project doctor --project /absolute/project
   ```

   The matching Claude-only copy is adopted into the generated v2 lock. The
   repaired Vercel payloads update through normal reconciliation if their old
   managed copies are intact. Modified copies fail preflight and need deliberate
   review; do not patch them or fabricate receipts to force an update.

3. Update `scripts/agents-check.mjs` and `scripts/agents-check.test.mjs` to accept
   v2 manifest/lock data and build each target's expected set from `lock.skills`
   rows whose `targets` include that target. Validate nonempty, unique, active
   targets, the scoped declarations, exact manifest correspondence and the existing
   hashes/provenance/receipt checks. Cross-check copy identity between providers
   only for IDs assigned to both. Reject missing, extra, duplicated or wrongly
   targeted installations. Remove the hard-coded Claude-only sidecar branch;
   keep the general integrity checks and regression cases.

4. Once the checker uses the lock, remove `.ai/target-extras.json` and its checker
   fixtures. Update `.agents/README.md`, `docs/SkillMigration.md` and any active
   selection/verification records to describe the v2 declaration and single
   reconciliation workflow. Retire the separate `catalog install --agent claude`
   maintenance command. Preserve historical migration evidence as historical.

5. Repeat sync and doctor. Confirm the repeat is unchanged, the guardrails row has
   exactly `"targets": ["claude"]`, and no Codex guardrails copy exists. With the
   recorded selection unchanged, expect 190 Codex and 191 Claude registrations.
   Run `pnpm agents:check`, `pnpm test:tooling`, `pnpm format:check`, `pnpm lint` and
   `git diff --check` in maje-websites. Report runtime prerequisites separately;
   this migration does not install dependencies or activate hooks.
