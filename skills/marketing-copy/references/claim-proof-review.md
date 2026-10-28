# Claim-to-proof review

Before treating copy as ready, save the exact text under review and create a
project-local `product/copy-review.json` containing its SHA-256, review date,
and one record per material claim. A claim links to evidence ids and is marked
`supported`, `conditional`, `unknown`, or `rejected`.

Testimonials require real participant evidence, consent, and the exact quote;
synthetic or inferred material can help draft a placeholder but cannot prove a
testimonial. Metrics have a `valid_until` date and need an explicit
`revalidated_at` date after expiry. Unknown or rejected claims must not remain
in the saved copy. Review the actual saved artifact, not only a draft in the
conversation, and preserve missing-proof findings for the next decision.

The repository qualification checker at `scripts/check-product-learning.py`
verifies the digest, claim presence, evidence links, consent, and expiry. It
is not shipped as an installed skill resource and does not certify legal
compliance, persuasion, or conversion. Publishing, sending, or changing a
live campaign still requires authorization. The evidence-first mechanics are
compatible with the [Corey Haines product-marketing skill](https://raw.githubusercontent.com/coreyhaines31/marketingskills/main/skills/product-marketing/SKILL.md),
which was inspected at its moving main-branch revision observed on
2026-10-27 (`0cd9b3a0ef4c`); this link is research context, not a bundled
dependency or a pinned runtime resource.
