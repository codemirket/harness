# Recovery feedback follows the current draft

This is a focused original practice linked to [Saleor's reconciled-commerce lesson](saleor-commerce.md), not another reference instance or a claim about Saleor's runtime. It can be chosen directly; reading Saleor first is not required. The existing lesson's [source and rights record](saleor-commerce.md#source-edition-rights-and-exclusions) provides context for recoverable review. No upstream assets, identity or code are used here.

## Choose this practice

Use it when obsolete rejection can remain beside a corrected draft, including review/return only when those views belong to the real journey. A basic editable form, save control or draft label alone does not require this practice. If the brief already settles validation and saved/draft behavior, follow that contract and use ordinary interface checks unless another relationship remains open.

## Brief → relationship → visible decisions

Rook Press is a fictional community print room. A Tuesday proofing batch holds eight units. A pocket notebook uses one unit; a foldout map uses two. The requester must correct an over-capacity draft, review it and return to edit without losing its quantities or identifying label.

The relationship is **current quantities → current summary → current feedback**. A valid total beside an obsolete red error still makes recovery appear unresolved, even when Review works. Retiring a resolved error is different from discarding the draft or ignoring a remaining problem.

Located decisions in `assets/design-languages/recovery-feedback/index.html`:

- `current()` and `problem()` derive quantities, used units and validity from the same current inputs. Units and item counts stay distinct.
- `render()` updates `#composition`, `#used-units` and the batch label together. Once validation has been attempted, the purposeful version recomputes feedback on quantity edits; invalid corrections still receive the relevant current error.
- `feedback()` reconciles visible `#request-error` and input invalid states. Successful review also retires resolved feedback.
- `#review` and `#return-edit` change the view without rebuilding the inputs. The label, quantities and Tuesday context survive; return focuses the first quantity.

The default facts and draft remain readable without JavaScript. Native labelled inputs and buttons own keyboard interaction; enhanced edit/review controls appear only when JavaScript is available. Teaching notes sit outside `main`.

## Plausible wrong version

Open the same page with `?variant=misleading`. It has the same composition, arithmetic, input rejection and valid review path. After the first rejection, its summary updates while previous feedback remains. Valid submission can therefore succeed beside a stale error, and returning to edit brings that error back with the preserved draft. Visual polish and a working next button do not resolve this contradiction.

## Discriminating probe

At wide and 390px widths, use the keyboard:

1. Change the label to “River survey packs”. Choose six notebooks and two maps: ten units, eight available. Press Review; rejection must retain those inputs and the label.
2. Change notebooks to four: six items, eight units. Inspect the **current summary and visible error together**. The purposeful version clears the resolved capacity error and invalid state. The misleading version retains the ten-unit error despite the eight-unit summary.
3. Review, then Return to edit. Four notebooks, two maps, the label and Tuesday context must survive. The purposeful version remains clear; the misleading version still shows the obsolete rejection.
4. Try a correction that remains invalid, such as zero items or a fractional quantity. Current feedback must explain that problem rather than being cleared merely because typing occurred.

Judge this example's actual recovered view and return path, not only whether review opens.

## Transfer within the product's state contract

The transferable relationship is current state and truthful feedback. This example recomputes validation on edits after the first attempt. Preserve the target product's cadence: a clear-on-edit, validate-on-save form may retire the previous rejection and show a pending draft, then validate on Save. That does not establish that the new value is valid. Where live validation is required, a still-invalid correction needs its current problem rather than a blanket clear.

Here the summary follows current inputs. If the target comparison belongs to saved data, keep it on saved data until a successful commit and show the draft separately. Do not import this example's live summary, batch arithmetic, editable label or review/return screens into a brief that does not require them.

Restored data has a separate validity boundary. Malformed saved data is not evidence that storage is unavailable; preserve a usable current view, permit an explicit replacement save where the product allows it, and keep retry possible after a rejected write. Inspect which condition remains current before clearing feedback. This is not a reason to read another lesson automatically.

## Read, render and rights

Existing `design-inspiration` path reads return source text. Open [the original practice](../../assets/design-languages/recovery-feedback/index.html) with an available local-file browser, or serve the installed skill root on loopback using the [index's route](index.md#read-and-render-through-existing-delivery). Append `?variant=misleading` for the contrast. No build, network assets or dependency is required. This practice is not a twenty-eighth inventory cell.

The Rook Press name, fictional copy, HTML, CSS and JavaScript are original Mirket teaching material under the library's [MIT license](../../assets/design-languages/LICENSE.txt). No third-party media, fonts, marks or code are included. Rendering and local state checks establish only the exercised browser behavior; they do not establish screen-reader compatibility, physical-device use, human acceptance or a measured quality gain.
