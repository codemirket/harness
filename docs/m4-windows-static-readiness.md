# Windows static readiness review — 2026-10-26

Status: **Windows 11 x64 and ARM64 remain unqualified release gates**. This
review continues source work while the native hosts are unavailable, in line
with the [approved plan](../planning/implementation-plan.md) and
[Windows qualification status](windows-qualification-status.md). It records one
bootstrap repair and concrete implementation gaps; it does not establish native
installation, runtime behavior or Codex Desktop activation.

Scope: the PowerShell bootstrap, installation/update/recovery owners, permanent
locks, paths, child-process ownership and saved Codex registration. No package
was installed, no user installation or client settings were modified, and no
dependency, SDK, commit or publication was added. The source references below
are observations of this working tree on the review date. P1 means a Windows
release blocker, not a claim of exposure in an installed Windows product.

## Repaired: PowerShell 7 bootstrap could not create its private directory

**P1, fixed in source.** The bootstrap used
`Directory.CreateDirectory(String, DirectorySecurity)`. That overload is a
.NET Framework API. PowerShell 7 runs on .NET, whose corresponding API is
`FileSystemAclExtensions.CreateDirectory(DirectorySecurity, String)`. The old
call therefore could not bind on PowerShell 7 and stopped before copying or
launching the verified Mirket binary. Microsoft documents the
[Framework overload](https://learn.microsoft.com/en-us/dotnet/api/system.io.directory.createdirectory?view=netframework-4.8#system-io-directory-createdirectory(system-string-system-security-accesscontrol-directorysecurity))
and the [.NET replacement](https://learn.microsoft.com/en-us/dotnet/api/system.io.filesystemaclextensions.createdirectory).

[`scripts/install-windows.ps1:89`](../scripts/install-windows.ps1#L89) now
selects the ACL-at-creation API by PowerShell edition: Core uses the .NET
extension; Windows PowerShell retains the Framework overload. Both branches
apply the ACL during creation. The patch does not introduce a create-then-set
permissions interval or a fallback to a broadly accessible directory.

The selected existing private PowerShell **7.6.8, Core, macOS 26.5.2 ARM64**
provided these controls:

| Check | Observed result | Limit |
| --- | --- | --- |
| Parse the complete saved installer with `Parser.ParseFile` | Zero parse errors | Syntax only |
| Reflect the exact .NET `DirectorySecurity, String` signature | `System.IO.DirectoryInfo CreateDirectory(System.Security.AccessControl.DirectorySecurity, System.String)` exists | API availability on this runtime |
| Reflect the former Framework `String, DirectorySecurity` signature | Absent | Discriminating control for the original defect |
| Extract and invoke the saved Core selection branch with a null ACL and empty destination | The selected method bound and returned `PlatformNotSupportedException` from the ACL API | Expected non-Windows refusal; no usable destination or native ACL operation |

Windows PowerShell 5.1 execution, actual ACL creation and bootstrap installation
must still run on native Windows. No tracked source-text assertion was added;
the checks inspect the actual runtime API and the saved script's executable
branch.

## Prioritized implementation findings

### P1: interrupted Windows staging has no resumable file identity

[`src/release_store.rs:1225`](../src/release_store.rs#L1225) returns `None`
for every non-Unix staging identity. New copy journals persist that result at
[`src/release_store.rs:657`](../src/release_store.rs#L657). If a process stops
with an existing partially copied file, the next explicit stage operation
revalidates the retained tree. Its Copying-phase check at
[`src/release_store.rs:1355`](../src/release_store.rs#L1355) rejects
`file_identity.is_none()` as an unbound staging file. Thus an ordinary interrupted
Windows copy cannot use this recovery route, even when its file and bytes are
unchanged. This is a source control-flow conclusion; no Windows crash was run.

Required repair: journal a Windows volume/file identity from the actual open
handle and compare it on recovery before truncation or continued copying. Retain
the current refusal when identity is unavailable or differs. Native acceptance
must distinguish an unchanged partial copy, replacement with an equal-length
file, a hard-link alias, a replaced stage directory and a corrupt journal. A
successful complete copy alone cannot close this gap.

### P1: installed state and locks have no complete Windows ownership checks

The bootstrap protects its temporary executable directory, but that does not
establish the installed home's permissions. Installed directories are created
by [`src/release_store.rs:1710`](../src/release_store.rs#L1710) and
[`src/installation.rs:1437`](../src/installation.rs#L1437), with explicit
private modes only on Unix. The corresponding
[`verify_private`](../src/release_store.rs#L1719) and
[`maintenance::require_private`](../src/maintenance.rs#L226) checks do not
validate a Windows DACL or current-user ownership. In a permissive existing
parent, the source supplies no equivalent installed-state ACL guarantee.

For maintenance exclusion,
[`maintenance::same_file`](../src/maintenance.rs#L271) compares creation times
and directory status outside Unix; it does not compare a Windows native file
identity. The Codex config lock's post-open identity comparison is Unix-only at
[`src/codex_config.rs:416`](../src/codex_config.rs#L416), as is the release
update lock's comparison at
[`src/release_store.rs:188`](../src/release_store.rs#L188).

Required repair: one consistent Windows handle/file-identity and privacy
contract across installation, maintenance, staging and Codex-owned receipts.
Test replacement, aliasing, permissive ACLs and reparse-point destinations under
normal-user permissions. Do not replace permanent lock files to recover them.
Project identity separately refuses Windows at
[`src/identity.rs:320`](../src/identity.rs#L320), and uninstall identity also
explicitly refuses it at
[`src/installation_uninstall.rs:624`](../src/installation_uninstall.rs#L624).
Those refusals should remain until their native implementations are verified.

### P1: Windows child ownership and resource admission remain unavailable

The common runner refuses non-Unix execution before spawn at
[`src/process.rs:162`](../src/process.rs#L162). Navigation, SQL, design,
OpenPencil and embedding have analogous refusals; browser and video are still
restricted to their qualified macOS development route. These are meaningful
fail-closed controls, not Windows implementations. The non-macOS resource
sampler is an unsupported stub at
[`src/resource_admission.rs:29`](../src/resource_admission.rs#L29).

The separate Windows active-core handoff at
[`src/main.rs:786`](../src/main.rs#L786) uses `Command::status()` when the
running binary differs from the selected active core. That path contains no
Mirket Job Object owner. Include this handoff in ownership qualification as well
as the adapter leaves; do not infer owner-death cleanup from waiting for a direct
child during ordinary completion.

Required implementation: establish ownership before the child can execute,
retain the process/job handles, reject an assignment failure, prevent unintended
breakaway, and confirm complete tree exit before releasing admission or claiming
cleanup. Microsoft's [Job Objects contract](https://learn.microsoft.com/en-us/windows/win32/procthread/job-objects)
describes tree membership and kill-on-last-handle-close behavior; actual Chromium
and media descendants, inherited/nested jobs, cancellation and abrupt owner death
still need native tests on both architectures.

### P2: Windows replacement and Codex startup semantics have only source evidence

The Codex entry uses TOML string/array values for the explicit launcher path and
`serve --home` arguments at
[`src/codex_config.rs:543`](../src/codex_config.rs#L543); it does not construct
a shell command. The managed entry sets `required = true` and a 60-second startup
timeout. That saved configuration does not establish how Windows Codex Desktop
starts, fails or recovers the required server.

Codex config and installation replacements use UTF-16 `MoveFileExW` calls with
`MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH` at
[`src/codex_config.rs:780`](../src/codex_config.rs#L780) and
[`src/release_store.rs:1649`](../src/release_store.rs#L1649). Those flags do
not by themselves prove the multi-file transaction, ACL preservation, sharing
behavior or crash recovery. Microsoft documents the
[move flags and constraints](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-movefileexw)
and specifically describes attribute/ACL preservation for
[`ReplaceFile`](https://learn.microsoft.com/en-us/windows/win32/fileio/moving-and-replacing-files).
Choose replacement semantics from the owned-file contract and test them rather
than changing APIs from source resemblance alone.

Native acceptance needs existing unrelated TOML, a preexisting custom config ACL,
an open destination handle, a running launcher, interruption at each journal
phase, busy sessions, rollback and fresh startup after recovery. Include paths
with spaces/non-ASCII names, drive-letter spelling, drive roots, long paths and
reparse-point ancestors. The PowerShell wrapper explicitly rejects UNC paths at
[`scripts/install-windows.ps1:27`](../scripts/install-windows.ps1#L27);
direct CLI/library entry points must have a consistent, documented path policy.

## Build evidence and limits

The locked toolchain is Rust **1.99.1**. Both Windows Rust standard-library
targets are installed on this Mac; no Windows SDK or C runtime headers were
installed for this review.

1. Before compilation, `python3 scripts/check-build-inputs.py` passed for the
   current host payload: `aarch64-apple-darwin`.
2. `cargo check --locked --offline --target x86_64-pc-windows-msvc` stopped in
   admitted vendored `onig_sys v69.9.3`: its C compiler could not find
   `stdlib.h` for the Windows MSVC target. It did not qualify Mirket's Windows
   source or link a Windows executable.
3. Explicit input checks for **both** Windows targets refused the current
   `target/native/sqlite/lib` layout, which contains the selected macOS native
   payload. No receipt was rewritten, guard bypassed or native payload replaced.
   No second cross-build was attempted after those refusals.

The Windows SDK/CRT and target-specific SQLite/vector build inputs are separate
prerequisites. Installing a Rust target is insufficient. Neither this review nor
the PowerShell API check closes any native Windows gate, proves the proposed
Windows 11 25H2 floor, qualifies x64/ARM64 packages, or verifies Codex Desktop GUI
activation. Use the existing [ordered Windows acceptance gates](windows-qualification-status.md#ordered-acceptance-gates)
when the native hosts become available.
