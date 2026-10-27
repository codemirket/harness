# Combined Design, Browser, Video and Raster Mac qualification — 2026-10-26

An isolated qualification fork on macOS 26.5.2 ARM64 opened Video and Raster
only with `MIRKET_VIDEO_QUALIFICATION_FORK=1` and
`MIRKET_RASTER_QUALIFICATION_FORK=1`. The shipping Rust source and binary still
refuse both release components. The fork passed the build-input guard and an
offline locked build; it added no production dependency.

The release source combined the previously qualified Browser+Video+Skills and
Design+Raster+Skills trees. Their overlapping payload files matched byte for
byte; only the manifest and fork executable needed replacement. The first-party
macOS installer then filled the deliberately omitted browser and Raster trees
from the pinned Chrome for Testing ZIP and six Sharp npm archives. It verified
the archives and the full installed release before activation in a disposable
user home. The resulting release ID is
`sha256:b0850f163852603ee82ba546a4f94fb652de7470b1d2919b737ce38e9d6fb254`.
`mirket doctor --json` reported a healthy launcher, manifest, managed Codex
registration and the `design`, `browser`, `raster`, `video`, `skills` component
set. All tests below used that one installed launcher and its private Node.

| Installed journey | Observed result |
| --- | --- |
| Design token/vector export | `tests/design-installed-smoke.mjs` passed; saved CSS 100 bytes and PNG 3,372 bytes. |
| Raster conversion | `tests/raster-mcp-smoke.mjs --home` passed stale-source refusal, PNG/JPEG/WebP/AVIF outputs, a 3,151,914-byte PNG input, saved hashes and scratch cleanup. |
| Browser capture | `tests/browser-capture-smoke.mjs` passed capture, article extraction, click, reconnect and lost-start recovery; nine observed owned processes were reaped. The 21,642-byte screenshot was visually checked for the heading, Turkish text and clicked button state. |
| Video export | `tests/video-mcp-smoke.mjs --home --format mp4` passed MCP preparation, render through the selected Chrome and FFmpeg, stale-source refusal and source preservation. The 53,373-byte MP4 had SHA-256 `766763043ac71f7be6a15002872e6e1066237c4ee1144e0f0165087d31e74db4`. |

The disposable source, homes, installer inputs and browser screenshot are under
`~/.codex/task-state/harness-rewrite-implementation/combined-design-browser-video-raster-20261026/`.
Separate detailed Video format and Raster format tests remain in their focused
qualification records; this combined run checked component coexistence and a
representative installed journey for each. It does not qualify semantic memory,
SQL or navigation in the same package, the macOS 15 floor, native Windows x64
or ARM64, Codex Desktop GUI activation, or the outstanding Video and Raster
source/notice and recipient rights. No user installation, commit or publication
was made.
