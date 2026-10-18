# Harness website

The public site for this repository is https://harness.mirket.dev. Its source lives
with the harness so product claims and documentation links can change together.
It is a dependency-free static site, served by the existing Mirket VPS nginx.
It has no analytics, cookies, remote fonts, form collection or application daemon.

## Build and verify

Use Node 22 or later:

```sh
node website/scripts/build.mjs
node --check website/public/app.js
```

Canonical tokens live in `design/tokens.json`; the checked-in CSS was compiled
through Mirket's Style Dictionary tool. After editing tokens, copy the JSON into
the reported design workspace, call `tokens_compile` on it, then copy its CSS
output to `public/tokens.css` and update the source/CSS SHA-256 pair in
`design/compiled.json`. Build rejects drift. No design runtime is needed to serve
or package already verified static assets.

For browser QA, copy `public/` into the design workspace and exercise it through
Playwright. Check desktop, narrow, enlarged text, keyboard interaction, the three
route examples, all download options, clipboard failure and JavaScript disabled.
Capture bounded 2x viewports, rather than one giant full-page image.

## Deploy

This follows the learning-platform site's existing static release convention,
with a separate root `/opt/harness.mirket.dev` owned by `deploy`. The one-time
nginx configuration is in `deploy/nginx.conf`. Its certificate is independent
and uses the existing Certbot webroot renewal service. Preserve other vhosts.

After committing source, build and deploy:

```sh
node website/scripts/build.mjs
node website/scripts/deploy.mjs --dry-run
node website/scripts/deploy.mjs
```

Deployment uses the existing `admin@mirket.dev` SSH access and runs file operations
as `deploy`. It refuses an existing release directory, verifies hashes before an
atomic symlink switch, checks live asset bytes and real 404 responses, and restores
the preceding release if verification fails. Failed uploads remain inactive for
inspection. SSH credentials stay outside source. CI checks the static source;
deployment is an explicit operator action, not an unconfigured CI promise.

Rollback: under the deployment lock, create a new symlink to a verified previous
directory in `/opt/harness.mirket.dev/releases/`, then rename it over `current`.
Keep the current and preceding releases; remove older inactive releases only
after successful verification. Never edit an active release in place.
