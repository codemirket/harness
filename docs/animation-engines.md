# Local animation engines

Mirket packages two complementary production routes. Ordinary interface motion
remains DOM/CSS/SVG/Web Animations in the project's actual browser workflow.

| Engine | Pinned version | Use |
| --- | --- | --- |
| [HyperFrames](https://github.com/heygen-com/hyperframes/tree/bf881bf79d68cc78b52624f6431644ac152b2723) | 0.8.149 | Finite HTML/CSS/SVG/Canvas compositions with the real producer/player, GSAP timelines, local media and encoded exports. |
| [Motion Canvas](https://github.com/motion-canvas/motion-canvas/tree/e735a995aa738ca682feb4204a39ee7db7b5d279) | 3.17.2 | Drawn scenes with actual scene-graph generators, paths, gradients, clipping, text and coordinated timing. |

The verified runtime profiles cover macOS ARM64 with Linux ARM64 Podman workers.
Both use pinned Linux Node, Chromium and FFmpeg; HyperFrames also uses matching
FFprobe. Complete dependency
archives, native build tools and notices are pinned in the package definitions;
setup never runs package-manager install hooks. Serving never downloads software.

Choose from the deliverable and authoring medium. Keep editable source alongside
its derived PNG/video. Inspect sampled frames forward and backward, then the
saved playback at normal speed. Usable encoding does not prove art direction,
smoothness or a performance gain. Video export does not replace responsive,
interactive, focus or reduced-motion testing of a website.

Mirket owns scoped inputs, browser launch, fixed runner/build profiles, resource
limits, cancellation and atomic output publication. It does not expose arbitrary
encoder flags, shell commands, project build hooks or raw editor scripts. These
engines run in non-root, offline Podman workers with read-only roots and validated
staged inputs. Ordinary website and browser QA use the host's available tools and
permissions. See the isolation boundaries and platform limits in [Podman](podman.md).
See [packages](packages.md), [HyperFrames](hyperframes.md), [Motion Canvas](motion-canvas.md)
and the [local video workflow](../skills/advanced-motion/references/local-video.md).
