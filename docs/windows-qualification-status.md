# Windows 11 qualification status

**Planned, gated target; not part of the macOS ARM64 0.0.1 release.** The user selected Windows 11 x64 and ARM64 for later native qualification. No Windows host or pi session is available in the current workspace. A Mac source check or cross-compilation cannot establish Windows support.

The retained Windows scope is pi package integration, scoped SQLite/FTS and lazy MiniLM memory with Rust cosine scoring, selected-source JS/TS navigation, native skills, and the design/video package. General browsing, SQL navigation, C/C++ analyzers, generic file editing and sqlite-vec are outside Mirket's scope. The earlier Windows review in task history described those removed components and must not be used as the current payload checklist.

## Required native gates

1. Implement and exercise Windows ownership of each private child process tree with Job Objects. Confirm cancellation, timeout, abrupt owner exit, descendant reap, unknown-effect reconciliation and heavy-job accounting on x64 and ARM64.
2. Implement Windows project/private-state identity checks, reparse-point refusal, current-user ACL checks, volume/file identity and safe maintenance activation. Exercise paths with spaces and non-ASCII names.
3. Qualify exact Windows private Node, semantic model/ONNX, design, OpenPencil, D2, Chrome for Testing and FFmpeg inputs, with rights, hashes, complete manifests and native outputs. A vendor archive or static inventory alone is insufficient.
4. Run normal-user setup, doctor, skills, project opt-in, update, rollback, uninstall and memory recovery in disposable Windows homes. Preserve unrelated pi configuration and user data.
5. Exercise actual Windows pi package/skill loading, MCP calls and a saved design/video result. Measure cold/warm, idle/peak process memory and combined load on the intended machines. Treat x64 and ARM64 as separate qualifications; any ARM64 emulation route needs its own evidence.

The previous Windows bootstrap was removed during the clean pi-only migration.
Source target triples do not establish Windows packaging or readiness. Windows
remains unqualified until the native gates above pass. No user installation or
public release is changed by this status.
