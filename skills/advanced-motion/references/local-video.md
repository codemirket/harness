# Editable source to local video

HyperFrames in the `design` package offers experimental encoded-video export or exact frames.
Its supported evidence is a bounded synthetic export, not demonstrated autonomous
production quality or natural adoption. HyperFrames and FFmpeg are still bundled
with design core; there is no independent video installation switch. Keep this
experimental path only for an explicitly requested export whose result can be
inspected. Interactive browser work does not require it.
Keep editable source distinct from rendered frames and video. Inspect the actual
tool schema and runtime readiness before promising an output format.

Compile Style Dictionary tokens before the dependent scene render. Run
HyperFrames lint, snapshot and render in order: lint catches source issues,
snapshot gives a frame to inspect, and render produces the final sequence.

Author a finite timeline with deterministic inputs and declared local assets.
Capture representative start, middle and end frames, including transitions that
carry meaning. Check ordinary-speed playback, dimensions, duration, frame count,
audio when requested and visible content at the intended size. A deterministic
hash or a successful encode alone does not establish visual quality.

Interactive website motion still needs its real DOM/CSS/SVG/Canvas and Playwright
checks, including interruption and reduced motion. Preserve the user's source
format and report any unsupported export or unverified playback path.

For interactive browser scenes, select `interactive-motion` rather than encoded
video. `browser_screenshot` can exercise bounded local actions, emulate reduced
motion and sample CSS/WAAPI at `animation_time_ms`. A JS timeline may expose an
authored `window.__mirketSeek(milliseconds)` that seeks and settles it. A seek is
composition evidence, not a playback or smoothness measurement.
