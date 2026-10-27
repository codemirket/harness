# Chromium-only Mirket browser file selection

The Chromium-only file selection is now the compiled Design inventory at
`packaging/admission/m4-design-chromium-macos.runtime.json`. Its reviewed
predecessor comparison remains in
`packaging/admission/m4-design-chromium-macos.runtime.json.proposed`.
This changes only selected files, not the Playwright dependency or its source.
The user's installation and release admission for Chromium are unchanged.

The historical predecessor remains byte-identical at
`packaging/admission/m4-design-macos.runtime.json`, SHA-256
`f144d7a089143764fa0c2d5dbc253a6411492e49ab5b60a0ccfbae1b8660a49c`.
The selection retains every non-Playwright file and every retained source hash,
package record, source lock, font and notice. Playwright retains 11 of 115 files,
6,799,133 bytes. The successor has 6,024 package files, 72,590,389 bytes; it omits
104 unused Playwright files, 6,857,034 bytes.

The retained Playwright files are the package manifest, `index.js`, `index.mjs`,
`browsers.json`, `lib/bootstrap.js`, `lib/coreBundle.js`, `lib/utilsBundle.js`,
`lib/utilsBundle.js.LICENSE`, `LICENSE`, `NOTICE` and `ThirdPartyNotices.txt`.
This preserves Apache-2.0 source headers and bundled third-party notices.
The standalone CLI, installer helpers, server registry, Electron loader,
dashboard/recorder/trace viewer assets, types and upstream skills are omitted.

The unchanged upstream bundle and index exports still contain dormant Firefox,
WebKit, CLI and other library implementations. Their removal is not claimed.
Mirket imports only `chromium`, requires an explicit private executable and
exposes no engine selector, package installer or caller JavaScript. The protocol
test supplies hostile Firefox/WebKit exports and verifies that startup engine
selectors and non-Chromium requests refuse without dispatch.

## Executed browser qualification

On macOS 26.5.2 ARM64, the 11-file Playwright subset passed the public MCP
browser journey with private Node 24.22.1 and an explicitly selected
development Chromium snapshot. Navigation, article extraction and no-article
handling, interaction, a 1,280 × 720 screenshot, lost-start recovery,
reconnection, unknown-field refusal and process/profile cleanup passed. The
saved PNG is 21,642 bytes, SHA-256
`99ba50f2ebaa2d943e5935dd2c33e28f451e2d1279be955d6d76f41d2e2e221d`;
visual inspection confirms the heading, Turkish glyphs and clicked button.
Nine observed owned processes were reaped. The result and artifact are retained
under `~/.codex/task-state/harness-rewrite-implementation/design-chromium-subset-smoke-20261026/chromium-capture-T9dOnA/`.
The package-delta test passed. This proves the reduced Playwright file set
works against that development browser; it does not admit a distributable
Chromium binary or qualify an installed final release.

## Reproduction and installed qualification

Use a disposable private candidate directory and the project's existing explicit
private Node runtime. Run the source checks without installing packages or
executing upstream helpers:

```sh
python3 scripts/check-build-inputs.py
python3 -m unittest discover -s tests -p test_design_chromium_selection.py
"$MIRKET_TEST_NODE" adapters/node_modules/typescript/bin/tsc -p adapters/tsconfig.json
"$MIRKET_TEST_NODE" --test adapters/tests/browser-main.test.mjs adapters/tests/browser-session.test.mjs
```

For a disposable release source, set `MIRKET_TEST_TREE` to a hook-free production
`node_modules` matching the current adapter lock, `MIRKET_TEST_FONT` to the pinned
Noto Sans font, and `MIRKET_TEST_RELEASE` to a new empty absolute directory:

```sh
python3 scripts/assemble-design-runtime.py \
  --tree "$MIRKET_TEST_TREE" \
  --font "$MIRKET_TEST_FONT" \
  --release "$MIRKET_TEST_RELEASE"
```

Confirm that the staged `playwright-core` tree contains exactly the 11 selected
files and that importing its `index.mjs` resolves only files inside that tree.
Copy the current compiled `main.js`, `browser-main.js`, `browser-session.js` and
`browser-article.js` into its `adapters` directory. Copy the separately pinned
Readability files/notices from
`packaging/admission/m3-browser-readability.runtime.json`, checking every size
and hash. These browser additions are separate from the Design selection.

Run the existing public MCP journey against the staged subset and the explicitly
selected private Chromium executable:

```sh
"$MIRKET_TEST_NODE" tests/browser-capture-smoke.mjs \
  --mirket "$MIRKET_TEST_BINARY" \
  --node "$MIRKET_TEST_NODE" \
  --adapter "$MIRKET_TEST_RELEASE/adapters/main.js" \
  --browser-adapter "$MIRKET_TEST_RELEASE/adapters/browser-main.js" \
  --browser-executable "$MIRKET_TEST_CHROMIUM" \
  --evidence-root "$MIRKET_TEST_EVIDENCE"
```

That journey must exercise navigation, inspection, article extraction,
interaction, screenshot publication and process/profile cleanup. Its existing
engine-selector rejection is an additional public MCP check. A module import
alone does not qualify Chromium behavior.

The compiled Design inventory and assembler now select the same successor.
The predecessor remains only as historical comparison evidence. On macOS
26.5.2 ARM64, a guarded locked/offline build and disposable installation of
the saved successor passed assembly, manifest validation, staging, activation,
incomplete-install completion, healthy doctor checks and the unchanged
`design-installed-smoke.mjs` and `code-navigation-installed-smoke.mjs` tests.
The release identity was
`sha256:896a1b6e710f8de9fb0dcc97fcf4b46529090c8fa80f719e7ed0f3aae40fd03e`;
its manifest contains 6,039 files, including the 6,024 selected package files.
The exact commands, log hashes, source, manifest, binary and disposable home are
retained under
`~/.codex/task-state/harness-rewrite-implementation/design-successor-installed-20261026/`
(`result.json` SHA-256
`9b08c694b2d5a03fe3cca9c019f12fd7026d9e4e4512e6be828bca7e82fd5149`).
No user installation or client setting changed. Chromium recipient rights,
source build and final browser release qualification remain separate open gates.
