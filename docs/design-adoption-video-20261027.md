# Video authoring adoption evidence — 2026-10-27

This record retains the exact executable Anime.js v4 example, disposable
commands, runtime identities, decoded frame paths and pixel oracle used for the
motion/video skill update. It is development evidence, not release admission.
The historical 2026-10-26 frame set and the fresh 2026-10-27 native run below
are separate runs; historical pixels are not presented as proof of the current
checkout.
No dependency, installed release or `~/.mirket` state was changed. The local catalog was refreshed for the authored skill resources.

## Authoring fixture

The retained executable fixture is private at:

`/Users/nazmiyilmaz/.codex/task-state/harness-rewrite-implementation/video-intended-api-retest-20261026/fixture/`

The source body is `fixture/index.html` (1,959 bytes, SHA-256
`ea954e74953b92e19b25214e9229610c5a70a6830fb50622a94ddcfc0700a7d6`). Its
authoring-critical fragment is:

```html
<div id="root" data-no-timeline data-composition-id="qualification"
     data-width="640" data-height="360" data-start="0" data-duration="2">
  <div id="anime" class="track"></div>
  <div id="css" class="track"></div>
  <div id="waapi" class="track"></div>
  <audio id="voice" src="./voice.wav" data-start="0.25"
         data-duration="1" data-volume="0.5"></audio>
  <audio id="music" src="./music.wav" data-start="0.75"
         data-duration="1" data-volume="0.25"></audio>
  <script src="./anime.umd.min.js"></script>
  <script>
    window.__hfAnime = [anime.animate('#anime', {
      x: [0, 480], duration: 2000, ease: 'linear', autoplay: false
    })];
    document.querySelector('#waapi').animate(
      [{transform:'translateX(0)'}, {transform:'translateX(480px)'}],
      {duration:2000, fill:'both'});
  </script>
</div>
```

The exact local Anime.js 4.5.0 UMD bundle is
`fixture/anime.umd.min.js`, 118,043 bytes, SHA-256
`8d5b3a58a1f64023a04a4cedeef135a2263b18da04b35177ac13438a0bad033b`.
The local Noto Sans font is 2,049,096 bytes, SHA-256
`bfb7bb691513f12e734dc346c03a03f784912432d7e3fa8e56efcf906fe86b3d`.
The fixture uses only selected local sources; no CDN or remote asset was used.

This is the no-GSAP profile. `data-no-timeline` is intentional and avoids the
pinned 0.8.197 readiness wait. It must not be generalized to a GSAP timeline
composition. The source registers Anime.js through `window.__hfAnime`; an
unregistered instance can produce a valid file with a frozen intended object.

## Runtime identities

Recorded probe paths and SHA-256 values were (these identify the probe run, not later rebuilds containing refreshed skill metadata):

| Runtime | Path | SHA-256 |
| --- | --- | --- |
| Mirket binary | `/Users/nazmiyilmaz/Projects/harness/target/debug/mirket` | `31923781900a69fc4bc07461ac878a8c49a6a359938e5d70d2c118dffc18d744` |
| Video adapter | `/Users/nazmiyilmaz/Projects/harness/adapters/dist/video-main.js` | `8fad5adccd0d54b984a37e8c3e3b475bed995d40ecb1b718e0a6ab774887d6a0` |
| Private Node 24.22.1 | `/Users/nazmiyilmaz/.codex/task-state/harness-rewrite-implementation/openpencil-qualification-20261025/node-v24.22.1-darwin-arm64/bin/node` | `6f32069480ec14d52b6d7b1f3c7124cfbbc46203abf5ddf5c65b127435f3f329` |
| Private Chromium | `/Users/nazmiyilmaz/.codex/task-state/harness-rewrite-implementation/browser-probe-20261025/browser/chrome-mac/Chromium.app/Contents/MacOS/Chromium` | `18bc94c01fff4c2d9b4db67f5911e748fd1ce0f213b059e6e83d6dce3ca82324` |
| Private FFmpeg | `/Users/nazmiyilmaz/.codex/task-state/harness-rewrite-implementation/ffmpeg-qualification-20261025/video-followup-20261025/private/bin/ffmpeg` | `8374f88f171b233c072da4acbf9058e9b563ce048d9300edabbe6036f616505e` |
| Private ffprobe | `/Users/nazmiyilmaz/.codex/task-state/harness-rewrite-implementation/ffmpeg-qualification-20261025/video-followup-20261025/private/bin/ffprobe` | `8c6f1213081e3805be3e2d69b9db85ff291235771b76b4f35bd64bf5ad280a1c` |

The historical retained run records platform `macOS-26.5.2-arm64-arm-64bit`
and all identities in
`video-intended-api-retest-20261026/entry-identities.json`.

## Commands and outcomes

The new disposable authoring-contract checks were run from the current
checkout:

```sh
node tests/design-adoption-video.mjs
```

They passed valid local preflight, missing `data-no-timeline`, remote asset and
missing asset refusal cases. The optional exact-bundle execution was run with
the private Node and paths above:

```sh
/Users/nazmiyilmaz/.codex/task-state/harness-rewrite-implementation/openpencil-qualification-20261025/node-v24.22.1-darwin-arm64/bin/node \
  tests/design-adoption-video.mjs --native \
  --chromium /Users/nazmiyilmaz/.codex/task-state/harness-rewrite-implementation/browser-probe-20261025/browser/chrome-mac/Chromium.app/Contents/MacOS/Chromium \
  --ffmpeg /Users/nazmiyilmaz/.codex/task-state/harness-rewrite-implementation/ffmpeg-qualification-20261025/video-followup-20261025/private/bin/ffmpeg \
  --ffprobe /Users/nazmiyilmaz/.codex/task-state/harness-rewrite-implementation/ffmpeg-qualification-20261025/video-followup-20261025/private/bin/ffprobe \
  --fixture /Users/nazmiyilmaz/.codex/task-state/harness-rewrite-implementation/motion-video-qualification-20261025/fixture
```

The exact local HyperFrames package executed 24 frames at 640×360/12 fps,
producing an MP4 whose private temporary output was 52,099 bytes in the final
rerun. Independent FFmpeg decoding found the orange Anime element at first
sample minimum X=72 and final sample maximum X=539. The temporary project,
output and frames were removed with the disposable root. This result predates
the retained-output run below and is not substituted for it.

### Fresh current native evidence

The current checkout was rerun with a new empty evidence directory. This is a
direct HyperFrames producer run only; the Rust/MCP route was not repeated here
because the unchanged route was already exercised above. The private Node used
for this run was the explicit `codex-gui-core` runtime (Node v24.22.1), not
`PATH`:

```sh
/Users/nazmiyilmaz/.codex/task-state/harness-rewrite-implementation/codex-gui-core-20261026/release-source/runtime/node/bin/node \
  tests/design-adoption-video.mjs --native \
  --evidence-dir /tmp/mirket-video-evidence-current-dDbzy2 \
  --chromium /Users/nazmiyilmaz/.codex/task-state/harness-rewrite-implementation/browser-probe-20261025/browser/chrome-mac/Chromium.app/Contents/MacOS/Chromium \
  --ffmpeg /Users/nazmiyilmaz/.codex/task-state/harness-rewrite-implementation/ffmpeg-qualification-20261025/video-followup-20261025/private/bin/ffmpeg \
  --ffprobe /Users/nazmiyilmaz/.codex/task-state/harness-rewrite-implementation/ffmpeg-qualification-20261025/video-followup-20261025/private/bin/ffprobe \
  --fixture /Users/nazmiyilmaz/.codex/task-state/harness-rewrite-implementation/video-intended-api-retest-20261026/fixture
```

The run passed both a registered and a negative same-fixture render. It
retains only the selected source files, two MP4s, six decoded PNG samples and
the JSON oracle (no browser profile or crash directory):

`/tmp/mirket-video-evidence-current-dDbzy2/`

The exact current paths are:

* source: `source/index.html`, `source/anime.umd.min.js`,
  `source/NotoSans.ttf`, `source/voice.wav`, `source/music.wav`;
* registered output: `registered.mp4` (52,077 bytes,
  SHA-256 `1293f9efcd14e771d845b144f413311435850f9e433b26bbc8d7764f7402d325`);
* negative output: `unregistered.mp4` (51,888 bytes,
  SHA-256 `a0ae7d804f0ac10ac9f309690022480c18a48d4697d3c737cc36084dec820605`);
* decoded samples: `registered-first.png`, `registered-middle.png`,
  `registered-last.png`, `unregistered-first.png`,
  `unregistered-middle.png`, `unregistered-last.png`;
* pixel oracle: `oracle.json` (SHA-256
  `68fb05dafcb5d472901d52d7f7d3c9790a8c5e15a3635d9e1340f9d516e9c9db`).

The independent FFmpeg RGB oracle scans the orange element at y=100..135 with
`R>180,G<130,B<120`, and the blue control at y=172..207 with
`R<100,60<G<190,B>120`. Current values are registered orange min-X 72, 272,
492 (first/middle/last); unregistered orange min-X 32, 32, 32; and independent
blue min-X 72→492. Thus the registered Anime object moves, while removing only
`window.__hfAnime =` freezes the orange object and leaves unrelated CSS motion
moving. The saved pixels were viewed independently (registered first/middle/
last and unregistered first/last) before recording this result.

The full current Rust/MCP route was also run with explicit private paths:

```sh
node tests/video-mcp-smoke.mjs \
  --mirket /Users/nazmiyilmaz/Projects/harness/target/debug/mirket \
  --node /Users/nazmiyilmaz/.codex/task-state/harness-rewrite-implementation/openpencil-qualification-20261025/node-v24.22.1-darwin-arm64/bin/node \
  --video-adapter /Users/nazmiyilmaz/Projects/harness/adapters/dist/video-main.js \
  --chromium /Users/nazmiyilmaz/.codex/task-state/harness-rewrite-implementation/browser-probe-20261025/browser/chrome-mac/Chromium.app/Contents/MacOS/Chromium \
  --ffmpeg /Users/nazmiyilmaz/.codex/task-state/harness-rewrite-implementation/ffmpeg-qualification-20261025/video-followup-20261025/private/bin/ffmpeg \
  --ffprobe /Users/nazmiyilmaz/.codex/task-state/harness-rewrite-implementation/ffmpeg-qualification-20261025/video-followup-20261025/private/bin/ffprobe \
  --fixture /Users/nazmiyilmaz/.codex/task-state/harness-rewrite-implementation/motion-video-qualification-20261025/fixture \
  --scenario publish
```

Observed result: macOS Darwin ARM64, `status: pass`, 55,329-byte MP4,
SHA-256 `d406f6917b441a6862179325887318316062f5d9bed6c8dbd3935012e5440f31`;
Rust preparation/start/receipt/publication, selected Chromium/FFmpeg render,
stale-source refusal and source preservation all passed.

The retained prior MCP command and complete evidence are private at
`~/.codex/task-state/harness-rewrite-implementation/video-intended-api-retest-20261026/intended-api-run.json`.
That record used the same fixture and exact private media pair, with release
admission explicitly false.

## Independent frozen trial route oracle

The three frozen specimens were read before staging and then copied without
editing into private disposable projects. The current Rust/MCP route was
invoked with the explicit private runtime above, `runtime_status` was checked
first, and each job was prepared and started through `video_prepare` /
`video_start`. This is an operation-qualification result, not a model repair or
an export claim. The retained raw records are at:

`/Users/nazmiyilmaz/.codex/task-state/harness-design-adoption-20261027/runtime-oracle-20261027-d/runtime-oracle.json`

Runtime identity for all three calls was macOS Darwin ARM64, release
`video-trial-20261027`, version `0.0.1`, with `video.configured: true`,
`skill_capabilities: ["video"]`, a 2 GiB output cap and a sampled 5 GiB work
cap. Configured status did not imply that these particular authored sources
were admitted or renderable.

| specimen | selected source hashes (SHA-256) | prepare | actual start/result | output/decoded evidence |
| --- | --- | --- | --- | --- |
| `video-no-skill` | `index.html` `553cf5b032fe180503ba2cb7f1456e835780cdee9f4a434ce9c1053957fdadb7`; `styles.css` `2249d955e76e32190bcc592a716965db5a1b8c9b11f6109d06b0cb3bdbab1f20`; `film.js` `50ff12afe60f8e2241983a1918d912dcba59a408d3f2b584dc533baaf56f5f4e`; `captions.json` `5e41232274feb7fdd42e90969adf02614ff92993113bd43e5971f65a1965282a` | `prepared` | failed before renderer: `authored_root_profile_mismatch` (the root lacks the required `data-no-timeline` marker) | no artifact; cleanup `removed_verified_owned_files`; no frames |
| `video-baseline` | `index.html` `ee6634b94740b79f06d74658a129a11e9151b4ccd2dc0589eadb13fc2855a321`; `style.css` `b98c1d940138ac400aded5edb9abbbd0fe2d5d5920cf84a9ec142058796f482c`; `composition.js` `2be1312713b4e460ad3bdf14287ab4e04c14509b11d1fb52781b93a172ab9bca`; `captions.json` `c5ae21b1d361fb7b36c7305b7386f25f34a5c2ab767e220defe2c2faee0eb98d` | `prepared` | failed before renderer: `authored_root_profile_mismatch` (same missing no-timeline marker) | no artifact; cleanup `removed_verified_owned_files`; no frames |
| `video-current` | `index.html` `68da48a2b746bbcdbc6d41dba3ecb5e48c61002c26615b59ea150d1b3151a22c`; `film.css` `add1c7c233ebc6f93b8be4388ca676703e2a718c6cf008e3ea618432658dc560`; `film.js` `4bfcf108e3dd7e08553e3cf13c305bc0ef1a4cfdc87589fb14d985215b6de0f5`; `captions.json` `8de87bed36cff44b4dec459de0b20e32b4df9f0746515e2e5f2f87d8665a520f` | `prepared` | failed before renderer: `explicit_local_font_required` (the source declares a local Helvetica face but selects no bundled font binary) | no artifact; cleanup `removed_verified_owned_files`; no frames |

The first two refusals are source-contract failures, not HyperFrames render
failures. The current specimen reaches the font contract and demonstrates a
real gap in the authored trial: its README says font portability must be
checked, but the exact route requires a selected `.ttf`, `.otf`, `.woff` or
`.woff2` source plus an `@font-face` declaration. Since the instruction was to
render exact frozen sources, I did not inject a font or alter their timing.
Consequently there are no honest decoded-frame or intended-object timing
comparisons for these three trials. Their frame/caption timing remains only a
source-level claim until the specimens are corrected and rerun. The hardened
runner was `tests/design-adoption-video-runtime.mjs`; it takes the trial root
as an explicit absolute argument, bounds server shutdown, rejects pending RPCs
on child failure, asserts these three frozen refusal/effect/cleanup outcomes,
and retains a successful MP4 before disposable cleanup if a future specimen is
accepted. Its source copies lived only under the temporary private project
roots and were removed after each run.

## Retained decoded frames and pixel oracle

The saved MP4 from the retained MCP run is:

`/Users/nazmiyilmaz/.codex/task-state/harness-rewrite-implementation/video-intended-api-retest-20261026/work/mirket-video-mcp-wWWsz3/project/result.mp4`

The exact FFmpeg frame extraction command and success receipt are retained in
`decoded-frame-pngs.json` and `final-receipt.json`. There are 24 saved 640×360
RGB PNGs at:

`/Users/nazmiyilmaz/.codex/task-state/harness-rewrite-implementation/video-intended-api-retest-20261026/frames/frame_000001.png`

through `frame_000024.png`. For immediate inspection, use frames:

* first: `frames/frame_000001.png` (SHA-256 `64d6d4d2aa07108de79e3afdefafb2268834d4e073bd3fddb5e6182c8ee00a9f`);
* middle: `frames/frame_000013.png` (35,600 bytes, SHA-256 `ceb3512fc8951eddb76b3380ed94e1772c4d785aac6f6583f294bdc9d2d17e9b`);
* last: `frames/frame_000024.png` (27,216 bytes, SHA-256 `d76a36e0f62157a26bf1131c24eb156e9a41ec62bdce2d099e7247de50adf90f`).

The independent raw decode is `decoded.rgb`, 16,588,800 bytes, SHA-256
`bdfbc50276f6c19df694b42dd79177fbad70d9398c6d21d1af34f9a55407ac04`.
The retained pixel oracle scans the orange track row with a lossy tolerance:

```text
for each decoded frame:
  scan y=100..135, x=0..639
  collect x where R > 180, G < 130, B < 120
  report min(x), max(x)
assert frame 1 left ~= 32 and frame 24 left ~= 492 (±2 px)
```

The complete recorded per-frame oracle is
`video-intended-api-retest-20261026/motion-observation.json`: frame 1 left=32,
right=79; frame 13 left=272, right=319; frame 24 left=492, right=539. The
saved first, middle and last PNGs were independently viewed; the visual record
is `visual-observation.json`. It reports all three tracks moving and the timed
caption visible in frame 12 and absent in frames 1 and 24.

## Profile-path incident and cleanup

The first optional direct-producer attempt used the repository as its current
working directory. Because the Mirket video policy intentionally places browser
profile/crash state below `cwd`, it created a generated browser profile at:

`/Users/nazmiyilmaz/Projects/harness/profile/`

This was not the requested project and was an incorrect test path, although it
did not write an artifact into the project source. The exact generated profile
was moved intact to `/tmp/mirket-video-profile-20261027` for cleanup/recovery.
The test was corrected to `chdir` into its canonical disposable root before
launch; the rerun succeeded and removed the entire temporary root, including
profile, crash state, output and temporary frames. The full Rust/MCP smoke uses
per-job disposable roots and did not leave a repository profile. No user
installation or live project was touched.

## Limits

This proves the exact Anime.js v4 example executes and moves its intended
object on one macOS 26.5.2 ARM64 fixture through the current route. It does not
qualify arbitrary compositions, GSAP timelines, other formats, playback,
Windows, long renders, complete rights/source delivery, or audio listening and
gain/synchronization. The retained receipt explicitly records those limits.
