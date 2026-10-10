# Capability explorer QA

Browser continuation completed on 2026-10-10, approximately 11:01–11:07 UTC, using the installed host Chrome through the Codex browser tools. The search, selection, empty state, keyboard journey, and inspected desktop/phone layouts passed. No functional or layout defect requiring a change to `index.html` was found in this bounded review.

This was a separate continuation after the original bundled Codex CLI trial timed out following a sandboxed browser-launch failure. That original trial remains **incomplete**. This report does not turn its source checks into browser evidence or claim the CLI completed the browser work autonomously.

## Artifact and method

- Served only this public local trial directory with Python's standard-library HTTP server, bound to `127.0.0.1` on an ephemeral port. The temporary URL was `http://127.0.0.1:63283/`.
- Interacted with the real page through browser controls and read its DOM/accessibility state. Inspected screenshots at desktop **1440 × 1000**, phone **390 × 844**, and narrow-phone **320 × 740** CSS pixels.
- Used native keyboard input and browser locator actions for the journeys below. DOM evaluation was read-only; it measured focus, visible text, element dimensions, overflow, and computed styles.
- Used tab-scoped Chrome emulation for `prefers-reduced-motion: reduce`, then cleared it. These phone dimensions are browser viewport tests, not physical-device or mobile-browser tests.
- Screenshots were rendered and inspected inline in the browser-tool record. The documented capture API exposed image bytes without a filesystem-save operation; no standalone screenshot files are retained in this directory. This report records the exact states and observations, but does not provide a durable pixel comparison artifact.
- No dependencies or services were installed. No page source or supplied capability data was changed during this continuation.

## Actual browser checks

| Check | Action and observed result |
| --- | --- |
| Desktop first load | At 1440 × 1000, the editorial introduction, labeled search, 33-role list, selected Frontend Engineer, deliverable, and numbered acceptance evidence rendered with clear hierarchy. The two-column composition had no horizontal overflow. |
| Search and selection | Entered `CFO`: one result appeared, with a selection prompt instead of stale Frontend Engineer details. Arrow Down focused CFO with a visible 3px brown outline; Enter selected it, set its pressed state, and focused the outcome heading. |
| Exact CFO content | Rendered deliverable: “A reconciled financial decision model with cash timing, obligations, base/downside scenarios and decision thresholds.” The two rendered criteria were “Opening cash plus receipts minus payments reconciles to closing cash” and “Profit, liquidity, assumptions and financing needs remain distinct.” These match the supplied capability. |
| Return and search shortcut | Escape from the selected heading restored focus to the CFO result. `/` returned focus to the search input. |
| Discriminating no-match case | `zzzz-no-such-role` produced `0 capabilities found`, hid the stale results/detail, and displayed “No matching roles” plus an explanation containing the query. Inspected at 1440 and 320 pixels wide; no horizontal overflow. The empty-state Clear search button worked and returned focus to the input with all 33 roles. At 320 × 740 it is below the initial fold and reached by normal scrolling during the click. |
| Phone role list | At 390 × 844 the layout became a single column, search and first role buttons remained readable, and desktop helper content was reduced. No side-by-side detail competed with the list. |
| Alias and long detail | `Senior Team Lead Engineer` found CTO. Selecting it showed the supplied Technical Leadership deliverable, both acceptance criteria, and the other aliases. At 390 × 844 the long aliases and deliverable wrapped within the card. Focus moved to `detail-title`; the visible Back to roles button measured 44px high. |
| Phone return | Back to roles preserved `Senior Team Lead Engineer`, hid the detail, restored the matching result list, and focused the selected CTO result. |
| Narrow phone | At 320 × 740, `Devops Engineer` found one role. Its long “Devops and Infrastructure Engineering” label wrapped inside a 272px-wide result button; the button was about 91px high. The selected detail stayed within a 288px-wide card, with readable wrapped deliverable/evidence and a 44px Back to roles target. Document scroll width equaled the 320px viewport. |
| Reduced motion | With Chrome's reduced-motion emulation enabled, the media query matched and role-button transition duration computed to `0s`. Returning to results, clearing with Escape, keyboard list navigation, and selection continued to work. The emulation was cleared afterwards. |
| Keyboard boundaries | Arrow Down from the input focused the first role. End focused Graphic Designer, Arrow Down wrapped to Frontend Engineer, Arrow Up wrapped back to Graphic Designer, and Home returned to Frontend Engineer. Each had a visible 3px focus outline. Enter selected Frontend Engineer. |
| Reload and shortcuts | Reload restored the default 33-role state and Frontend Engineer outcome. Tab exposed the visible Skip to role search link; Enter focused the search. The next Tab reached Jump to selected outcome; Enter focused the selected outcome heading. |
| Readable colors | Inspected screenshots and sampled computed colors: body `rgb(34,55,44)` on `rgb(245,245,239)`; muted category `rgb(94,104,95)` on `rgb(255,254,250)`; deliverable text `rgb(34,55,44)` on `rgb(234,240,223)`. These agree with the previously checked palette. Focus outlines were visible in the rendered keyboard states. |
| Runtime observations | No page JavaScript error was returned by the browser log query. One warning belonged to the browser extension's integration script. The local server recorded successful document load/reload and a benign browser request for the absent `/favicon.ico` (404); the page supplies no favicon. |

## Source evidence reused without rewriting it

The existing `qa/source-checks.json` is dated `2026-10-10T10:51:06.801Z`. Its nine checks cover JavaScript parsing, exact projection of all 33 supplied capability records, required role/outcome content, static IDs and ARIA references, absence of external assets/services, exclusion of installer/schema metadata, search/accessibility hooks, shipped search expressions, and declared palette contrast. Its lowest tested palette ratio is **4.82:1**. The record explicitly identifies these as source/expression checks, not rendered accessibility acceptance.

SHA-256 values were recomputed after browser verification and matched that unchanged source-check record:

```text
index.html        b8a25496eb081b22b6a4d6e2526a239c00578f1075f4701e999ade26d709084c
capabilities.json 0825ec8ef0d643d779aa74fbafffd6a2ae82a31794bb44f7a1d2c3d9dddcc773
```

The original source-check report and helper were left intact. Palette ratios and structural ARIA checks do not establish WCAG conformance.

## Remaining limits and cleanup

- No physical phone, touch keyboard, screen reader, OS text scaling, Safari, Firefox, or offline `file:` launch was tested. Chrome's exact version was not captured.
- All 33 records have exact source-level data checks; the browser review manually selected Frontend Engineer, CFO, CTO, and Devops Engineer, not every role.
- The JavaScript-disabled explanation is present in source but was not exercised with JavaScript disabled. No persistence across reload was requested; reload returns to the default selection.
- Visual inspection is agent evidence, not the user's visual approval or a formal accessibility audit. No impact or usability metric was measured.
- Temporary viewport and reduced-motion overrides were reset. The agent-created QA tab was closed. The local HTTP server was stopped with Ctrl-C; its process exited. No server files, dependency trees, or other scratch artifacts were created. Only this report was added.
