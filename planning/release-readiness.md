# Release checklist

The v0.0.1 package for **macOS 26.5.2 ARM64** is locally qualified at implementation
commit `5b03547`. The [qualification](../docs/first-release-qualification-20261028.md)
and [release admission](../packaging/admission/first-release-macos-arm64.json)
bind its exact bytes and recipient materials. Public release publication has not
occurred. Source changes after that revision do not update or requalify its archive.

Use this checklist when an authorized task creates a new candidate. It is not an
instruction to rerun closed experiments or alter the user's installation.

| Gate | Observable evidence |
| --- | --- |
| Scope | Exact target host, source revision, changed outcomes and accepted exclusions. Windows and other macOS versions require native qualification; Desktop GUI remains waived and untested. |
| Dependencies and rights | Approved exact closure, explicit executable identities, all required notices/source, setup acquisition pins and vendor terms. Preserve the full recipient delivery. |
| Source | Build-input guard before compilation; locked/offline Rust tests, all-target Clippy with warnings denied, formatting, locked adapter build/process tests and affected repository checks. |
| Protocol | Actual tested binary/private Node MCP smoke; configured and absent payloads, stale source/release, wrong project and unsupported operations. |
| Installed outcomes | Disposable install/doctor; affected memory/navigation/design/video operations; source changes must affect saved output. Decode/inspect visual exports and verify editability where claimed. |
| Lifecycle | Reinstall/update/rollback/uninstall, busy/staged state, drift/collision refusal, unrelated setting preservation and memory/forgetting recovery. |
| Skills | Exact selected bodies/resources, hash/provenance refresh, retired-ID refusal and actual sampled outcomes. Do not convert planned development cases into completed acceptance. |
| Delivery | Manifest, installer, archive and corresponding source/notices agree; download/unpack rehearsal; current failure limits and independent review. |

Keep failed attempts and unqualified paths visible. A successful receipt, hash or
model verdict alone cannot close an outcome gate. No publication, purchase,
credential change, commit, push or live installation change follows merely from
passing this checklist.
