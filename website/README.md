# Harness website

[harness.mirket.dev](https://harness.mirket.dev) is Mirket's introduction and
practical usage guide. It is a dependency-free static site,
served by the existing Mirket VPS nginx. It has no analytics, cookies, remote
fonts, form collection or application daemon.

The main action copies an installation prompt. `public/install-prompt.txt` is the
canonical prompt; the homepage textarea and root README contain identical text,
checked by the build. The prompt links to `docs/agent-installation.md` in the
repository. Publish that guide before deploying a site that references it.
Manual installation remains in `docs/installation.md`.

Documentation is published in the repository, with `docs/README.md` as its index.
`docs/release-status.md` records the published release and separates download
identity, desktop activation and useful task outcomes. Publish linked guides before activating the site; do not change
the public release label until matching binaries and their manifest are published.

The dark landing page explains the harness through editable HTML/CSS visuals:
a task workbench, a three-stage bug-fix example, a local architecture map,
responsive design specimens and a checkpoint timeline. The workflow buttons
switch between Understand, Build and Verify; without JavaScript all three
explanations remain visible. These are illustrative explanations, not live runs
or evidence of product performance.

The visual system uses charcoal surfaces, warm white text and a brighter
raspberry accent derived from the existing brand. Wide layouts pair explanations
with their visual; narrow screens stack them. Detailed setup and verification
material stays in native disclosures. Motion is limited to short control color
transitions and respects reduced motion.

`docs/visual-quality.md` owns the full visual workflow. Runtime enforcement applies
to Mirket-owned workers; native host tools follow installed guidance. Neither
path certifies aesthetics or human approval. The original layout and explanatory
visuals are maintained as source; no third-party page copy or imagery is reused.

## Brand assets

`public/logo.png` is the unmodified logo from the
[codemirket GitHub organization](https://github.com/codemirket), retrieved from
its organization avatar on 2026-10-18. It is used in the header, footer and
favicon. The dark palette adapts its raspberry accent to `#f078a0` for contrast against
`#100f12`, with `#f3f0f4` text. The logo itself is not recolored.

## Build and verify

Use Node 22 or later:

```sh
node website/scripts/build.mjs
node --check website/public/app.js
node --test website/scripts/deploy.test.mjs
```

The build checks static assets, local anchors, prompt consistency and canonical
token digests. It writes a hash manifest alongside the files in `website/dist/`.
CI runs these checks when the site, shared README or agent guide changes.

Canonical tokens live in `design/tokens.json`; checked-in CSS is compiled through
Mirket's Style Dictionary tool. After editing tokens, copy the JSON into the
reported design workspace, call `tokens_compile`, copy its CSS output to
`public/tokens.css`, and update the SHA-256 pair in `design/compiled.json`.
No design runtime is required to serve already compiled static assets.

For visual QA, copy `public/` into the design package's reported workspace and use
`browser_screenshot` to observe the existing page without token injection. Inspect actual 2x desktop and narrow
captures. For the complete interaction checks, serve the site locally and use an
existing Playwright installation and browser (the script installs nothing):

```sh
python3 -m http.server 8767 --bind 127.0.0.1 --directory website/public
# In another terminal, set MIRKET_PLAYWRIGHT_MODULE and MIRKET_CHROMIUM_PATH
# to the installed module entrypoint and browser executable, then:
node website/scripts/browser-check.mjs http://127.0.0.1:8767/ /absolute/evidence
```

The build and browser checks bind the release badge and link to the canonical
installation prompt. The browser checks include upgrade and project-registration
disclosures, memory/recovery content, workflow selection,
project setup, architecture and the visual workflow disclosures.
They cover 1440, 768, 390 and 320 CSS-pixel widths, mobile touch emulation,
44px navigation targets, unbroken installation step numbers, keyboard access, prompt
copy success and clipboard denial, native disclosure controls, enlarged text,
console errors and JavaScript-disabled access. Inspect the saved captures as well
as the report. Browser emulation does not establish physical iPhone or Android
behavior. Keep evidence outside shipped source.

## Deploy

Deployment uses the existing `admin@mirket.dev` SSH access and performs file
operations as `deploy`, under `/opt/harness.mirket.dev`. Nginx configuration is in
`deploy/nginx.conf`. Preserve other vhosts and the existing certificate renewal.

After committing and publishing the source and linked documentation:

```sh
node website/scripts/build.mjs
node website/scripts/deploy.mjs --dry-run
node website/scripts/deploy.mjs
```

The deployer requires committed website source, refuses an existing release
directory, checks artifact hashes, atomically switches the active symlink, and
checks live asset bytes and real 404 responses. It restores the preceding release
if live verification fails. Failed uploads remain inactive for inspection.
SSH credentials stay outside source. Deployment is an explicit operator action.

Rollback: under the deployment lock, create a new symlink to a verified previous
directory in `/opt/harness.mirket.dev/releases/`, then rename it over `current`.
Keep the current and preceding releases; remove older inactive releases only
after successful verification. Never edit an active release in place.

## Legal information

`public/legal.html` presents the short [legal information document](../docs/legal-information.md)
with a homepage footer link. Keep both copies aligned with confirmed practices.
It covers the free project, local and provider data, website logs, licensing and
private contact. It is informational, not a certification of compliance.

Check the page using the same installed browser settings as the homepage:

```sh
node website/scripts/legal-browser-check.mjs http://127.0.0.1:8767/ /absolute/evidence/legal
```

The check exercises desktop and narrow layouts without JavaScript, navigation,
keyboard access, enlarged text, contact links and unexpected requests/storage.
Inspect the saved captures. Deployment uses the standard artifact integrity and
rollback checks described above.
