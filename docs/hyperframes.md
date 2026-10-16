# Local HyperFrames compositions

Mirket's `hyperframes` package pins HyperFrames 0.8.149 (source `bf881bf79d68cc78b52624f6431644ac152b2723`). Its adapter calls the upstream producer's capture-session, timeline seeking, compilation, media processing and encoding APIs. This package runs pinned Linux Node, Chromium and FFmpeg/FFprobe in Mirket’s Podman environment on macOS ARM64. Installation extracts verified archives; no npm, Bun, pip, lifecycle scripts or runtime downloads run.

Start the approved environment with `mirket environment start`, then use `mirket package install hyperframes` to add it to a saved environment, then `mirket package probe hyperframes --project /absolute/project --json` to check the runtime. For an explicitly disposable first installation, use `mirket --home /absolute/disposable-home setup --target all --package hyperframes --yes`. A package installation, successful probe and visually accepted animation are different results.

A project-relative manifest binds every authored input byte:

```json
{"version":1,"entry":"scene.html","assets":["tone.wav"],"width":640,"height":360,"fps":30,"frames":90}
```

The entry is an HTML filename in the manifest's directory. Assets are relative to that directory and may use subdirectories. Only listed assets are copied into the private render project. The root and GSAP timeline use HyperFrames' own authoring contract:

```html
<script src="/__mirket/gsap.min.js"></script>
<div data-composition-id="main" data-start="0"
     data-width="640" data-height="360" data-duration="3">
  <div id="mark">Hello</div>
  <audio id="sound" src="tone.wav" data-start="0" data-end="3"></audio>
</div>
<script>
window.__timelines = {
  main: gsap.timeline({paused:true}).to('#mark', {x:300, duration:3, ease:'none'})
};
</script>
```

`/__mirket/gsap.min.js` is the pinned GSAP 3.15.0 asset in the render stage. To preview this HTML independently, serve or map the same GSAP bytes locally and use a playback control for the paused timeline. Mirket's server injects the actual HyperFrames player runtime; arbitrary `renderAt` callbacks are not its authoring API.

`composition_inspect` opens the actual producer runtime. `composition_frame` seeks a zero-based frame and reconciles its reported index, byte count and SHA-256 before publishing the PNG. `composition_verify` compares the first, middle and last frames in forward and reverse order. `composition_render` uses HyperFrames' full pipeline and publishes H.264/AAC MP4 or VP9/Opus WebM. WebM can retain transparent backgrounds; the report decodes the first RGBA frame and counts transparent/opaque pixels. Silent video elements should declare `muted data-has-audio="false"`; clips with authored sound use `data-has-audio="true"`. Each render also checks video dimensions, frame count, frame rate, codec and duration, then decodes all video and any audio. Decoded frame hashes reject a fully frozen video when source samples change; deliberately static compositions remain valid. This guard does not establish broader source/export visual fidelity. Authored audio must survive into the export.

Outputs must be distinct project-relative files with existing parents. Existing output requires `overwrite:true`; optional `expected_sha256` protects the old revision. Source remains editable. Inspect representative PNGs and watch the saved output to judge composition, transitions, pacing and sound.

The profile allows local HTML/CSS/JS/JSON, inline editable SVG, static mapped SVG, PNG/JPEG/WebP, WOFF/WOFF2, WAV/MP3 and MP4/WebM. Compressed media is probed before extraction: duration at most 30 seconds, at most two streams, video up to 1080p pixel count/1920 per side/60 fps with H.264/VP8/VP9, audio up to stereo/48 kHz with supported PCM/MP3/AAC/Opus/Vorbis. Jobs use one software browser worker, even dimensions 64..1920, at most 2,073,600 pixels, 24/30/60 fps and 900 frames/30 seconds. Limits are 64 listed assets, 8 MiB per asset, 32 MiB total inputs, 32 MiB output, 256 MiB private scratch and 90 seconds per job. A complex job can hit a bound before the nominal maximum.

The adapter refuses remote resources, missing assets, embedded browsing contexts, popups, manifest/root mismatch and page failures. Browser request interception and Node request checks restrict supported paths to local staged data. The container disables Chromium’s inner sandbox because its zygote requires a chroot prohibited by the capability-free profile. Results explicitly report `browser_sandbox: false` and `isolation: podman`; native browser tools retain their own sandbox. Site isolation and web security remain enabled. The Podman profile additionally enforces no network, read-only validated input, a read-only root, no capabilities and bounded private temporary storage. Authored JavaScript runs inside that container. Cancellation and disconnect terminate its captured immutable container before publication. See [execution environment](podman.md) for setup, quotas, recovery and platform limits.

Provenance and the full reviewed dependency/license inventory are in [`SOURCE-INVENTORY.json`](licenses/packages/hyperframes/SOURCE-INVENTORY.json). HyperFrames is Apache-2.0; GSAP has its own standard no-charge terms, including restrictions on competing visual animation builders. The adapter exposes code-authored compositions, and does not expose upstream hosted services, distributed/cloud rendering, project scaffolding, npm commands, arbitrary encoder flags, GIF/HLS/ProRes, HDR or optional native Canvas.

Primary sources: [pinned HyperFrames source](https://github.com/heygen-com/hyperframes/tree/bf881bf79d68cc78b52624f6431644ac152b2723), [published package](https://registry.npmjs.org/hyperframes/0.8.149), [producer capture](https://github.com/heygen-com/hyperframes/blob/bf881bf79d68cc78b52624f6431644ac152b2723/packages/engine/src/services/frameCapture.ts), [GSAP terms](https://gsap.com/community/standard-license/).
