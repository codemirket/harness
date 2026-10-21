# Host-owned video export

Mirket no longer bundles an encoded-video renderer. Existing `motion` selections
remain as guidance and explicitly require a host encoder chosen and authorized
for the task. Do not claim package readiness or silently substitute browser
animation, another provider or a new dependency. Preserve existing scenes and
outputs; old workers cannot resume with a changed renderer binding.

Keep editable source distinct from frames and video. Inspect the chosen encoder's
actual schema, formats and readiness. Reuse project tokens where supported.

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
