# Style Dictionary output consumed by a rendered component

On 2026-10-26, a disposable macOS ARM64 development session compiled three
token sources through Mirket's actual `design_prepare`, `design_start` and
`job_status` MCP route. One unchanged authored HTML/CSS workshop card consumed
the saved Style Dictionary CSS outputs. Browser inspection and saved captures
confirmed that generated colors, spacing, typography and radii reached the
rendered component. This closes one representative token-consumption check;
it does not admit the private browser or establish full release/GUI behavior.

The historical runner was `tests/token-consumption-smoke.mjs`; it was removed
with the general Mirket browser route. The results below remain dated evidence,
not a currently runnable browser contract. Current live UI checks belong to
Codex or the project toolchain.
That runner required explicit canonical absolute paths for `--mirket`, `--node`,
`--adapter`, `--design-adapter`, `--font`, `--browser-adapter`,
`--browser-executable` and an existing external `--evidence-root`.
It copies the selected core before launch, creates its own private temporary
state/project, retains sources, CSS, captures, receipts and runtime identities
outside the repository, then removes successful temporary state. It neither
sets up an installation nor modifies an existing home or browser profile.

## Inputs and specimen

The run used macOS **26.5.2**, ARM64, private **Node 24.22.1**, the previously
assembled design payload from `typescript-new-schema-installed-20261026/source`,
and its pinned Noto Sans font. The design component uses **Style Dictionary
5.7.1**. The current development browser adapter used the previously tested
private **Chromium 156.0.8078.0** snapshot from
`browser-probe-20261025/browser`. That snapshot remains rejected for release
because its source/rights closure is incomplete; it was used only as an
explicit local development renderer. No new dependency, upstream helper,
download or user-browser attachment occurred.

The native DOM specimen is a fictional community repair-workshop card, with an
original inline SVG mug, local font, visible event copy and a working reserve
button. The HTML links directly to `/tokens.css`. Its component rules use
`var(--color-brand)`, canvas/ink/text colors, font family/body/title sizes,
card padding/gap and card/button radii. The consumed properties have no fallback
token values. There are no remote assets. One hidden diagnostic records
computed styles; the public browser inspection reads that DOM text through its
fixed read-only extractor, without accepting caller-supplied JavaScript.

The 1280 × 720 baseline is the frozen minimal specimen. The saved HTML identity
is `sha256:98e342e4ff243d09b3dec2ce11641c782cbf581af29eb5e2dd0ec0e5a486fb7f`.
The same source bytes served all three token variants. Host image inspection
of the complete baseline found clear title/body hierarchy, legible local-font
copy, an intact mug/handle/mending seam, comfortable spacing and an unclipped
primary action. No visual repair was needed.

## Discriminating results

| Case | Compiled/consumed result | Saved capture SHA-256 |
| --- | --- | --- |
| Baseline | Brand `#17594e` became computed `rgb(23, 89, 78)` on the mug panel and button. | `9b9a08dbef4c85b2a531ce93ded48c50aa6e45debbc42b35465742391365721a` |
| Changed token | Only the source brand token changed to `#a23d2a`; the unchanged consumer became `rgb(162, 61, 42)`. Host inspection confirmed both accent surfaces changed while text/layout remained consistent. | `172dc43d9de93cabc0b430999f5a6078e47c8cc7b23db3796e9f57387f6688b0` |
| Missing token | Compilation of the remaining tokens succeeded; the consumer check identified missing `--color-brand`. The panel became transparent (`rgba(0, 0, 0, 0)`) and the white mug/button text disappeared against the white card. This intentionally failed visual state is retained as negative evidence. | `1066bfcbfbed6425467076aec7bb29b20e6690a24ccfaa3930740276504e4632` |
| Narrow baseline | At 390 × 844, the component stacked its artwork above text, used its deliberate 30px title rule and had no horizontal overflow. All copy and the action remained visible. | `0e44bd784ebebbe0a8b42a805672ee122adeb08cae436cc368c0ea017b8bc517` |

Both healthy desktop variants computed **20px gap**, **28px card padding**,
**20px card radius**, **38px title**, the local `Proof Sans` font family with
font loading confirmed, and canvas `rgb(247, 242, 231)`. The primary button
changed its actual text to **Seat reserved** after a browser click. Motion was
not part of the specimen. Keyboard navigation, screen-reader behavior and
independent human design acceptance were not tested.

All three compilation jobs and four screenshot jobs returned `succeeded`,
matching saved artifact revisions and `scratch_cleanup: complete`. Browser
close and server EOF completed successfully; the temporary-profile directory
was empty. This script did not independently observe every native process
identity and makes no broader process-tree or resource claim.

## Retained evidence and limits

The successful run is retained privately at
`~/.codex/task-state/harness-rewrite-implementation/token-consumption-qualification-20261026/token-consumption-DX1vcS/`.
It contains `consumer.html`, all three original `.tokens.json` sources, their
actual `.tokens.css` outputs, `font.ttf`, four PNGs and `result.json` with input
paths/digests, source revisions, computed-style observations and job receipts.
The core used by this run had SHA-256
`1a38c4666dd75fe2f74309cdf5277ebfbbec7251acd45d6241d65f81dcae8520`.

The source script passed Node syntax validation and the actual smoke journey.
Host image inspection covered all four captures, including the intentionally
broken missing-token state. The token compiler supports CSS and nested JSON;
this check qualifies only CSS consumption by one responsive DOM component.
It does not establish other token export formats, all design outputs,
installed browser packaging, the proposed macOS floor, Windows behavior,
Codex Desktop GUI activation, signing/notarization or release/source-rights
admission. The user's installation and original payload sources were untouched.
