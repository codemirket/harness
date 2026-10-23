# Mirket release status

Mirket 0.0.1 is the clean native baseline. The release manifest and checksums
for the selected artifact are authoritative; a source checkout or local build
is not a published binary.

| Distribution | Verified target boundary |
| --- | --- |
| Native CLI and installers | macOS ARM64, Linux x86-64 and Windows x86-64, each requiring its own build and installer checks |
| Optional design and agent-runtime packages | Managed Podman on macOS ARM64 with a verified Linux ARM64 container |
| Application data | One private `.mirket` root containing executable, package receipts, native databases and runtime dependencies |

The 0.0.1 package starts with fresh native fact and execution databases. It does
not migrate or reinterpret incompatible task, memory, checkpoint or design
state. Candidate verification uses fresh disposable homes and explicit project
registration. Deployment may retain a supported installation receipt and client
configuration through the ordinary setup transaction. An intentional release-number
reset requires the exact prior version and a hash-bound local candidate; it does
not authorize database conversion. Existing unsupported data remains preserved
and is rejected if accessed, rather than silently adopted or erased.

The core binary requires no Python, Node or background service. The optional
agent-runtime uses the Rust fact store, Rust execution journal and a contained
fastembed-rs helper. The optional design package is separately verified and
does not establish agent-runtime readiness. A package receipt or executable
identity does not prove a live desktop connection or useful task outcome.

## 0.0.1 acceptance boundary

Before deployment, the candidate must pass native formatting, Clippy, all-target
tests, catalog checks, disposable setup and doctor checks, contained helper and
runtime acceptance, same-format backup/restore, fault probes, and the exact
design/browser/accessibility checks selected for the package. Performance claims
must be backed by the fixed synthetic corpus and measured receipts. Unverified
platforms and native-client activations remain explicitly unverified.

## Historical releases

The repository contains historical reports for earlier experiments and
releases. Those files are retained as evidence and are not current 0.0.1
contracts. They may mention the superseded Python Mem0/Qdrant/LangGraph
runtime; no compatibility adapter for that runtime ships in 0.0.1.
