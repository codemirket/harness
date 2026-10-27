# Native Python navigation candidate: ty 0.0.91

On 2026-10-25, the official [ty 0.0.91 release](https://github.com/astral-sh/ty/releases/tag/0.0.91)
published a macOS ARM64 archive and Windows x64/ARM64 archives. Its
[language-server feature table](https://docs.astral.sh/ty/features/language-server/)
lists document symbols, definitions and references. The
[untrusted-workspace setting](https://docs.astral.sh/ty/reference/editor-settings/#untrustedworkspace)
is documented to suppress external commands and uv integration. These are
reasons to qualify ty as a **candidate**, not proof of Mirket integration.

The macOS ARM64 archive was fetched as an inert file, without an installer,
from that pinned release. Its SHA-256
`8ec3e0315cab3f8969a974db1fb1d431ef38525793455dda411089e733c71fbe`
matches the official `sha256.sum` asset; that checksum file itself hashes to
the release-listed
`9dfd6992d01187f64e91374cc1c4662f8c2d03dcccb482c471eb877478150dfd`.
The archive contains one 26,599,936-byte
ARM64 executable, whose digest is
`39ab93d3aefb55f00338770e33509291569040c20111f52a2bc43abb77850459`.
macOS strict code-signature verification passed; the signature reports an
OpenAI OpCo, LLC Developer ID and a 2026-10-22 timestamp. The executable
reports `ty 0.0.91 (67fbd2874 2026-10-22)`. Its dynamic references are Apple
system libraries/frameworks. The official source archive digest
`8e841a976d1b599d22026c67b492181a55942d67de539270ae07744f0e13fb80`
matches the release checksum and includes ty's MIT license. A complete linked
dependency source/notice inventory has **not** been established.

A disposable offline LSP probe used `ty server`, an explicit project with a
Unicode directory and CRLF Python sources, `untrustedWorkspace: true`,
`experimental.useUv: off`, a private home and PATH command sentinels. It ran
under macOS network denial. Seven exact assertions passed: UTF-16 negotiation,
cross-file definition to the original declaration, three declaration/reference
locations, one document symbol, clean shutdown/framing, no observed sentinel
execution and unchanged source bytes. The probe saved its request/response
trace and result under the implementation task-state directory
`ty-qualification-20261025/`. This is a small accuracy corpus and a PATH
sentinel, not proof that every possible absolute-path command or project form
is passive.

Ty is a plausible simpler replacement for the blocked Pyright packaging path:
one signed native analyzer rather than an npm bundle and separate Node runtime
for this capability. Its own [version policy](https://github.com/astral-sh/ty#version-policy)
still calls it beta and warns that its API can break between `0.0.x` releases;
that stability tradeoff must be weighed against Pyright's packaging risk.
Selection still requires representative Python fixtures,
prepared environment/stub behavior, project-config precedence and execution
sentinels, full redistribution rights and native Windows/macOS-floor checks.
The production LSP owner must revalidate all source/project revisions and
original UTF-8 locations; raw ty responses do not establish currentness.
No ty binary is in Mirket's release, no language tool is exposed for it, and
this experiment did not modify a user installation.
