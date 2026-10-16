# Saleor: reconciled commerce

**Language:** Reconciled commerce. This is a curator's name for the relationship studied here, not an upstream category or a license to reproduce Saleor's identity.

**Use when:** A buyer must choose a product configuration and inspect the complete amount before a recoverable checkout commitment.

**Poor fit:** Marketing discovery alone, or a real payment, tax or inventory implementation expected from this local specimen.

## Brief → concept

A buyer at the fictional Fieldwork supply store needs a Daybreak flask. Capacity changes the unit price; quantity changes the subtotal. Delivery and sample tax must be included before the buyer finishes a local review. A rejected availability check must preserve the address and contact text.

**Concept:** One selection, one reconcilable amount. Product presentation supports choosing; named line items and explicit costs support commitment. A correction invalidates the prior review rather than silently carrying its approval forward.

## Source evidence → transferable grammar

**Actually observed:** The public Paper storefront and Plimsolls page were rendered on 2026-10-15 at 1360×1000. The product's image, labelled size/color choices, price and option-dependent add action were visible. No cart mutation or order was made. The live deployment is not asserted to match the repository revision. Checkout evidence is source-only: the pinned order-summary, flow and information-step files were read, showing named checkout stages, item quantities, cost roles and error handling. No upstream payment or server recovery flow was exercised.

**Grammar to transfer:** Keep configuration close to its price; name the selected variant in the bag; align money for comparison; distinguish an items subtotal from the complete amount; put review and supported correction before completion. The source's particular tax policy, prices, product photography and backend architecture do not transfer.

## Located decisions in the original specimen

| Location in `index.html` | Decision | Purpose |
| --- | --- | --- |
| `#capacity`, `#quantity`, `#update-bag`, `#draft-message` | Draft controls update the bag explicitly. Changed fields must be applied before review. | Typed intent cannot masquerade as an already-priced selection. |
| `#line-variant`, `#subtotal`, `#shipping`, `#tax`, `#total` | A committed capacity/quantity determines all amounts in integer cents. Sample tax is 8% of items plus delivery. | The amount can be independently reconciled and compared with the reviewed selection. |
| `#review-form`, `#review-status`, `#use-available` | Three 750 ml flasks fail the local availability check. A correction to two preserves email/address and requires checking again. | Recovery changes the selection and total without losing unrelated input. |
| `#ready`, `#finish`, `#completion` | Completion follows a successful current review and explicitly says no order or payment occurred. | Local review is not misrepresented as a commerce backend. |

The identity, copy, SVG flask, CSS and JavaScript are original first-party teaching material. Only system fonts and inline code/art are used. Essential default product and complete cost facts remain readable without JavaScript; enhanced controls are unavailable. The teaching aside is outside `main`.

## Attractive wrong variant

Open `?variant=misleading`. A spacious original collection story precedes the purchase task and a large “Total” displays only the items subtotal. The actual delivery, tax and review amount remain below. The restrained composition is plausible retail design; its dominant amount has the wrong meaning at the commitment point. Compare at matching content and width.

## Discriminating failure probe

Select 750 ml, quantity 3 and update. Subtotal must be $120.00, delivery $6.00, tax $10.08 and total $136.08. An attempted quantity 0 must preserve that bag and name the error. Open review, change the address, and check availability: rejection must preserve the entered text. “Use available quantity (2)” must yield $80.00 + $6.00 + $6.88 = **$92.88**, then require a new review before local completion. `data-testid` values mirror the amount and state IDs above.

Reject the misleading variant when its headline “Total” differs from the sum that will be reviewed. This probes semantic truth rather than preference for a palette. Inspect focus after Back, text at 200%, and a 390px viewport; no required action is hover-only.

## Recovery feedback practice

A correct amount can still sit beside an obsolete error. Use the original [recovery-feedback practice](recovery-feedback.md): reject a draft, correct it to valid quantities, inspect the current summary and error together, then review and return to edit. Preserve the draft and context; recompute feedback so it describes the current problem. Its misleading version permits valid review while retaining the old rejection. A working next button alone does not pass this probe.

## Materially different transfer task

Design a community print-run quote. The inputs are total pages, color pages, copy count and pickup versus delivery; the quote must reject a color-page count greater than total pages while preserving the job description. Derive units and rounding from that contract. Provide a corrected review and state which prior checks become stale after a change. Do not reuse flask capacities, stock rules or the sample tax assumption.

**Reject the transfer when:** A dominant quote omits a required charge, or an edited page count retains an obsolete review confirmation.

## Exact owner, read and render route

Craft owner: `interface-design`. Embedded owner: `design-inspiration`. Instance ID: `saleor-commerce`.

```sh
mirket catalog read design-inspiration --path references/design-languages/saleor-commerce.md
mirket catalog read design-inspiration --path assets/design-languages/saleor-commerce/index.html
```

MCP: `skill_read {"skill_id":"design-inspiration","path":"references/design-languages/saleor-commerce.md"}` and `skill_read {"skill_id":"design-inspiration","path":"assets/design-languages/saleor-commerce/index.html"}`.

Resources: `mirket://skills/design-inspiration/references/design-languages/saleor-commerce.md` and `mirket://skills/design-inspiration/assets/design-languages/saleor-commerce/index.html`.

Render the saved HTML from the actual installed `design-inspiration` directory, using a local-file browser open or loopback static server; no build or dependency install is needed. Codex project copies commonly live under `.agents/skills/design-inspiration/`, Claude copies under `.claude/skills/design-inspiration/`; use Mirket's returned location. Compare the base URL and `?variant=misleading`. A source read is not a render.

## Source edition, rights and exclusions

- [Saleor Paper: live Plimsolls product](https://storefront.saleor.io/en/default/products/blue-plimsolls) — Mutable live demo inspected 2026-10-15. Product pixels support only the observed selection composition. Demo media, brand identity and fonts remain excluded.
- [Saleor Paper: checkout order summary](https://github.com/saleor/storefront/blob/dcf6f9c9852932843ce09a15144d139e7e2933fd/src/checkout/views/saleor-checkout/order-summary.tsx) — Commit `dcf6f9c9852932843ce09a15144d139e7e2933fd`, committed 2026-10-14, read 2026-10-15 alongside `flow.ts` and `information-step.tsx`. Checkout source-only evidence. [The pinned license](https://github.com/saleor/storefront/blob/dcf6f9c9852932843ce09a15144d139e7e2933fd/LICENSE) is **FSL-1.1-ALv2**, with a conditional future Apache-2.0 license after two years; this is not the older storefront's BSD license or present blanket Apache permission.

All upstream material is **link-only**. No Saleor source, logo, product copy, photos, fonts or UI assets are redistributed. The original example has its own repository license. Access to a website and a software license do not clear its branding or media.

## Verification scope

On 2026-10-15, local Chrome 155 on macOS rendered both variants at 1280×900 and 390×900. Checks cover native keyboard focus, 200% text sizing, JavaScript-off default reading, zero external requests and automated axe checks of purposeful defaults. The quantity/cost, availability rejection, retained-address, correction and local-completion effects were exercised. No real payment, inventory, tax service, screen-reader session, physical device, native-client activation, human visual acceptance or measured usability gain is claimed.
