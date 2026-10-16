# Evidence from actual viewports

`viewport-proof.cjs` captures a current viewport and a uniform vertical sequence
of real viewport PNGs. It does not stitch or resize them. A post-lock Chrome 155
probe found that scrolled `fullPage` screenshots can draw an unfocused fixed skip
link inside the document image even when its entire bounds are above the actual
viewport. Changing accessible product code to satisfy that image would repair
the wrong layer.

Use this module for a newly frozen evaluation. Preserve prior captures, scores
and failures; adding a capture method does not authorize regrading an opened
study or demonstrate an improvement in generated design.

```js
const { captureViewportProof } = require('./viewport-proof.cjs');
// The caller supplies an existing local browser page and output directory.
const evidence = await captureViewportProof(page, outputDirectory, 'saved-320');
if (!evidence.document_scan_complete || !evidence.scroll_restored) {
  throw new Error('Incomplete observation; inspect the retained evidence');
}
```

The current frame is captured before scrolling. The scan starts at zero and
advances by viewport height minus 128 CSS pixels, ending at the document bottom.
Every PNG has the actual viewport dimensions. The method retains requested and
observed scroll positions, document dimensions, active control, focus and
ancestor indicator geometry, fixed/sticky bounds, before/after geometry and byte
identities. The original scroll position is restored. It preserves fixed and
sticky content; a genuine obstruction must remain visible in the evidence.

Call with the page at the intended fixed state and width. The caller still owns
state transitions, keyboard checks, source/output isolation, runtime readiness
and visual inspection. Compare the actual viewport and its relevant bounds
before calling something an in-use overlap. A containing label may draw focus
while the native input has `outline: none`; inspect the pixels and the indicator
owner. Computed styles and rectangles are hints, not accessibility verdicts.

The output directory must already exist. Existing destinations and symlink
directories are refused. The default cap is 64 frames, including the current
frame; an excessive document is refused before capture. Recorded geometry changes
or unexpected scrolling leave the retained scan incomplete. Lazy content and
scroll-triggered behavior require diagnosis; do not silently omit them or raise
the cap after seeing a condition's result. Only the recorded geometry and focus
are compared for stability; unrelated application data changes need separate
state assertions.

Screenshots disable animations and use CSS-pixel scale. They establish neither
motion quality nor physical-device, assistive-technology or human acceptance.
File hashes bind saved pixels, not visual attention or semantic correctness.

Run the discriminating regression with the existing local runtime, without
installing packages:

```sh
node evals/design-quality/test_viewport_proof.cjs
```

The regression uses original synthetic pages and tests offscreen skip links,
genuinely focused links, real fixed obstruction, containing-label focus,
scroll coverage/restoration and failed observation paths. Its result is capture
method evidence, not a model-output design-quality gain.
