# Numeric motion observations

In the harness repository, use scripts/check-motion-observations.mjs only after the existing preview/export
path has produced its own observations. It does not render, decode, display or
judge pixels. It checks numeric geometry observations and capture identity; a
passing result is not visual inspection.
The script is repository qualification tooling, not an installed skill or MCP
tool. Other projects use their own measurement checks and retain the same evidence.

## Input contract

The packet is JSON with:

- schema_version: 1;
- capture.id, rational fps, finite frame_count, and measurement_basis equal to
  browser_frame_geometry, decoded_frame_geometry or decoded_frame_pixels;
- source_revision as sha256:<64 hex>; export_revision only for a bound export;
- viewport, DPR, theme and locale; browser-only captures remain explicitly unbound;
- font: {path, sha256, loaded} with loaded: true;
- frames, each containing a unique frame number, matching timestamp_ms and entity rectangles
  {x,y,width,height,visible};
- checks, each with an explicit id and supported kind.

The source/font paths are required in the CLI; a bound export also requires its
actual file. Their current hashes
must match the packet. This binds an observation to the actual capture inputs
without claiming that a model or viewer inspected them.

Supported checks:

- moves: entity displacement along x/y/distance over a frame interval;
- holds: geometry remains within a tolerance over all frames;
- settles: final geometry remains within a tolerance over the requested range;
- inside: target rectangle remains inside a container with optional margin;
- position: target rectangle stays at an expected x/y within a tolerance;
- no_clip: a measured text/asset box reports no scroll overflow or clipping;
- no_overlap: two visible rectangles do not overlap, optionally with margin.

Use frame intervals that include just-before, midpoint and just-after states
when a timing defect is plausible. A defect that preserves the first and final
frames must still fail an appropriate hold or settle expectation when intermediate
geometry is wrong. Endpoint displacement alone cannot detect that defect.

## Evidence boundary

The output declares
numeric_observation_contract_only; no_render_no_decode_no_visual_inspection.
It is a machine check of supplied measurements, not a claim that the measurements
are truthful. `browser_frame_geometry` is an actual source-DOM measurement, not
decoded-pixel evidence. Keep the source frame/geometry extractor independent from the
authoring model and retain the original decoded frames. Pair this checker with
saved-pixel display by an image-capable tool and an independent visual reviewer.

Do not use this checker to approve an aesthetic direction, infer easing from one
rectangle, replace accessibility review, or certify an unmanaged Anime/timer/
canvas/media track. Those paths remain unqualified unless their own observation
extractor and visual evidence are retained.
