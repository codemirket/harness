---
name: raster-art
description: "Prepare raster assets through crop, resize, composite or web-image export with source-linked dimensions, color and alpha checks. Skip new image generation without a transformation task."
---

# Raster preparation

Inspect the original image, intended placement, focal subject, output sizes and transparency/color requirements. Preserve the original and its provenance. Choose crop and scale from actual content; avoid stretching, destructive repeated recompression or silently inventing image detail.

Use the qualified Mirket Sharp route for deterministic transformations. Record the operation chain and selected encoding/quality. Check orientation and metadata handling, color profile/appearance, edge quality, alpha and file dimensions against the consumer. Use controlled fonts/assets and do not infer reuse rights from availability.

Inspect saved outputs at actual display size and a zoom that exposes artifacts. Verify responsive variants preserve the focal subject and that transparent edges behave on light and dark backgrounds. Compare decoded appearance rather than trusting extension, encoder success or byte size.

Return source-linked outputs, dimensions/format, transformation choices and relevant metadata/rights limits. Unless the task sets another budget, share 15 minutes and two final repairs across static production and review, with one target repair; combined interface/paged work uses 30 minutes/three and motion uses 45 minutes/four, not their sum. Stop when required variants are verified or report unresolved defects at the limit. Report unsupported color/format behavior instead of silently flattening alpha or replacing the original.
