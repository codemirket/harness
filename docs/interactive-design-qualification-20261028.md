# Interactive design qualification — 2026-10-28

This is a disposable host-browser QA trial, not a Mirket browser or video
qualification. The fixture was served from a task-private project and opened in
Codex's host Chrome through the CUA browser route on macOS ARM64. The saved
evidence is under:

`/Users/nazmiyilmaz/.codex/task-state/interactive-design-qa-20261028/trial-BLR6zu/`

The source revision is recorded in `qa-results-host.json`. Saved captures include
`clean-cdp.png`, `clean-mobile-cdp.png`, `missing-focus-cdp.png`,
`overflow-mobile-cdp.png`, `font-failure-cdp.png`, `reduced-clean-cdp.png` and
`reduced-motion-cdp.png`. These are saved through
the host Chrome tab's permitted CDP screenshot route and hash-bound to the
fixture revision. The page's independent DOM measurements are retained in the
same result record; screenshots are evidence of the rendered state, not an
automated visual-quality claim.

Observed outcomes:

- Clean controls: local `QALocal` font loaded, focus outline was 3px solid,
  and no horizontal overflow was measured at both the default 1728×819 host
  viewport and the clean 390×700 mobile viewport. A retained manual observation records keyboard Enter closing the disclosure
  while retaining focus; the validator does not authenticate that input event.
- Missing-focus specimen: the active button had `outline-style: none`; the
  checker reported focus visibility false.
- Overflow specimen: at the actual host-browser 390×700 viewport,
  `scrollWidth=2008` versus `clientWidth=390`; the horizontal overflow failure
  is reproducible and saved in `overflow-mobile-cdp.png`.
- Font-failure specimen: `QALocal` remained `unloaded` and the loaded-font
  predicate was false.
- Reduced-motion specimen: the host browser's real
  `prefers-reduced-motion: reduce` media feature was emulated through the
  permitted CDP route; the defective animation remained 1000ms/infinite. The
  clean reduced-motion control measured 1ms/one iteration in the same media
  environment.

The repository test checks source identity, unique case IDs, scoped capture paths,
screenshot bytes/dimensions and the recorded measurements. These joins detect
accidental drift; they do not authenticate a screenshot-to-DOM capture or a
keyboard event. The lead separately viewed the clean mobile, missing-focus and
overflow captures: readable mobile reflow and the missing outline are visible;
the overflow screenshot exposes displaced/offscreen content. A still image
cannot verify animation duration, which remains a browser measurement.

The optional headless runner failed to emit output and was removed after review
found a partial-stdout race. Actual evidence uses the connected host browser.
`fixtureHtml` is exported by the repository fixture for future disposable browser
runs. The font case deliberately leaves the local font unused/unloaded; it does
not qualify network font-error recovery. No aesthetic uplift, universal browser
support or release UI acceptance is claimed. No production dependency, live
installation, catalog entry or skill body was changed.
