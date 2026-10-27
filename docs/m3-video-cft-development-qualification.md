# Video with the setup-acquired browser — 2026-10-26

The Sharp-free HyperFrames development worker completed four public MCP export
smokes on macOS 26.5.2 ARM64 using the Chrome for Testing executable from a
disposable installed Mirket browser release and the neutral private
FFmpeg/ffprobe successor. The Rust server selected those exact paths through
its development command. **Video is still closed to normal installed releases.**

| Format | Saved bytes | Reported artifact SHA-256 |
| --- | ---: | --- |
| MP4 | 55,329 | `d406f6917b441a6862179325887318316062f5d9bed6c8dbd3935012e5440f31` |
| WebM | 48,443 | `5a2c18c1ce0bc19860ded0ff78b2741762465d8836059bf131db79cb8f74dbfc` |
| GIF | 174,408 | `ad4eea5bc460614c30472af4ff299f99dac8fe2e872263668c812a3e59ea249e` |
| PNG-sequence tar | 431,104 | `d745ca59c13f531da85be8a529e76e08605dbc7695846e22d1b3de06c3887f16` |

Each smoke passed MCP preparation, start, receipt and publication; the private
browser and media render; stale-source refusal; and source preservation. The
exact input hashes and results are in private task-state evidence
`video-cft-four-format-smoke.json`
(SHA-256 `fc303e10bfda354e08db88c770c0c909ed16834cc22bc9729be3c1f445bc7a17`).
These smokes did not retain independently decoded outputs. Earlier [four-format
development verification](m3-video-sharp-free-development-export.md) checked
decoded motion, captions, local assets and audio with the previous browser;
the [neutral FFmpeg qualification](m3-ffmpeg-neutral-successor-qualification.md)
checked media behavior and source delivery in its own scope.

This was the proposal state at the time of the browser development check. The
[later candidate](m3-video-installed-route-preparation.md#four-format-and-neutral-media-successor)
pins the corrected Sharp-free worker and neutral pair. An isolated installed
qualification is recorded there, while source/notice delivery, admitted
installation, Windows native tests and Codex Desktop GUI activation remain
separate gates. The browser acquisition is verified on this Mac; it does not
by itself establish Video's rights or release readiness.
