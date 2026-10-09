# Target migration: 2026-10-09

## Current target scope

The user subsequently removed Zed from the requested harness. Installation and
new project setup now default to `all`: Codex Desktop and Claude Desktop Code.
The shared capability sources, safe configuration merging, and manual Claude
Chat/Cowork handoff remain. Existing local Zed settings are preserved. Manifests
that still name the removed target require explicit migration before sync.

The conversion and local installation evidence below is historical; it does not
establish the current target set. Verification of the removal is recorded at the
end of this document.

## Original outcome and scope

Convert the personal Codex-first harness into a shared capability source with
Zed as the default and explicit Codex Desktop and Claude Desktop Code adapters.
Keep the reviewed skill catalog and project registration workflow. Make client
differences visible rather than claiming uniform support.

The initial conversion changed the repository and was verified in isolated homes.
The subsequent user request authorized local Zed configuration, commit and push;
local installation evidence is recorded below. Account/provider changes remain
outside this installation.

## Decisions

- Share one instruction source and the existing 14 authored global skills.
- Put target destinations, settings and extension policy in `registry/targets.json`.
- Describe HTTP/stdio MCP sources in `registry/mcp.json`; translate at installation.
- Make `install` and read-only `check` select `zed`, `codex-desktop`,
  `claude-desktop`, or `all`; default to Zed.
- Preserve unrelated settings, comments, user skills and project registrations.
  Back up existing settings and fail on conflicting endpoint ownership.
- Leave portable Codex appearance settings and legacy maintenance opt-in.
- Retain project init/add/plan/sync/doctor. Require matching Zed/Codex selections
  because both use `.agents/skills`.
- Generate manual account handoffs for Chat/Cowork instead of pretending local
  Code configuration covers those surfaces.

## Delivery sequence

1. Inventory instructions, installer, catalog, settings, tests and existing state.
2. Verify current official target contracts and their boundaries.
3. Implement neutral sources and target rendering with preservation and preflight.
4. Extend project target selection and platform entry points.
5. Update primary documentation and targeted regression coverage.
6. Run isolated installations, drift checks, project compatibility tests and the
   full repository gates; review the final diff.

## Evidence ledger

| Area | Evidence | Status |
| --- | --- | --- |
| Target contracts | Official documentation and Zed source linked in [targets](../targets.md) | Reviewed 2026-10-09 |
| Public commands | CLI help; isolated Zed dry-run, all-target link and copy install/check, handoff and all-target project init/plan/sync/doctor | Passed, exit 0 on macOS |
| Configuration preservation | Existing JSONC/TOML, unrelated keys, endpoint conflicts, backups, concurrency and active-client regression checks | Passed |
| Project compatibility | Existing manifests/locks plus Zed/Codex shared-destination selection cases | Passed |
| Focused regression checks | 17 target installer tests, 12 JSONC tests and 18 adapter tests | Passed during integration |
| Repository gates | `python3 -m unittest discover -s tests` | 519 tests passed in 42.572 seconds on macOS |
| Source consistency | `python3 scripts/render_registry.py --check`, `sh -n setup/macos.sh`, `git diff --check` | Passed |
| Local Zed installation | `install --target zed`, then `check --target zed` | Passed on installed Zed 1.23.2; details below |

Endpoint review additionally identified credential-forwarding risk when changing
an existing MCP URL or executable. The implementation now rejects changed URLs,
commands and arguments rather than carrying retained credentials to a new endpoint.
Codex TOML inline-header handling was also corrected during integration review.

Documentation checks passed for local link destinations and whitespace. The
isolated checks used an existing temporary home and project, explicit copy mode
for install/check, a verified foundation Codex plugin export, and a new handoff destination under a resolved parent path.
No installer was applied to the real user home. Existing source-backed guidance
links follow these repository edits immediately.

Independent configuration review findings were repaired and covered by regression
tests. All source edits preceded the final full suite and isolated artifact checks.
File generation proves configuration preparation; it does not prove live client
activation or model behavior. Native Windows/Linux execution remains a separate
on-device check unless recorded explicitly.

## Local Zed installation: 2026-10-09

The user authorized configuration on this Mac and publication of the conversion.
Zed 1.23.2 was quit normally, configured with `ai.py install --target zed`, and
reopened with this repository. The installer linked personal AGENTS.md, verified
all 14 shared skill links, merged the OpenAI Docs MCP declaration, confirmation
default and TOML extension policy. Existing unrelated settings were compared
with the private local backup and preserved. The backup remains beside the user's
settings file and is not part of Git.

`ai.py check --target zed` reports ready with no pending changes after reopening.
Zed's native Skills settings visibly list the shared harness skills. With a project
open, its MCP settings list enabled `openai-docs` with a green status indicator.
The TOML extension installation includes its local extension manifest. An empty
project initially showed no configured MCP servers; opening the harness project
resolved that view. No model request or MCP tool invocation was submitted, so
model entitlement, prompt application and tool-result behavior remain unverified.

## Removal verification: 2026-10-09

- Removed the target adapter, CLI/wrapper choices, extension policy, platform paths,
  catalog compatibility mapping and project shared-destination constraint for Zed.
  Public desktop aliases and legacy `both` remain compatible; `all` selects only
  Codex and Claude, and is the new installation/project default.
- Kept neutral skill/MCP sources and shared instructions. Claude account exports
  now use shared transport fields directly rather than a removed client renderer.
- Updated current setup guidance. The earlier conversion evidence above remains
  historical. Existing Zed configuration is neither removed nor rewritten.
- Full repository suite: 520 tests passed in 36.194 seconds on macOS. Regression
  coverage includes rejected removed targets, preserved existing Zed settings,
  two-target defaults, separate project selections and stdio account handoffs.
- Isolated CLI checks passed for default/all copy and link installation, read-only
  check, repeat no-op installation, project init/plan/sync/doctor and repeat sync.
  Each home had 14 global skills per client; each project had four foundation skills
  per client. Install/check/project/catalog CLI paths rejected `zed`.
- Claude account handoff generated 13 portable skill archives. Registry rendering,
  shell syntax and whitespace checks passed. Final prose edits were followed by
  the focused adapter/installer and catalog checks.

Verification used temporary homes and projects. No client settings were installed
in the real home, and no model or MCP request was invoked. Native Windows wrapper
execution was not tested on this Mac. Shared instruction/skill links naturally
follow the revised source content.
