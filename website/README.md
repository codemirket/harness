# Mirket website

Buildless static source for the Pi-only harness at <https://harness.mirket.dev/>.
HTML, CSS and local images need no package manager or build step. Content follows
`docs/architecture.md`, `docs/skills.md`, `docs/release-0.0.1.md` and
`docs/qualification.md`; package qualification does not qualify this website.

## Local preview

From the repository root:

```sh
python3 -m http.server 4173 --bind 127.0.0.1 --directory website
```

Open <http://127.0.0.1:4173/>; stop with Ctrl-C. This simple server serves
`404.html` directly but does not emulate nginx’s custom missing-page response.
Check `legal.html`, `robots.txt` and `sitemap.xml` alongside the homepage.
All styles and assets are local, with no inline scripts or styles, for the
host’s strict same-origin Content Security Policy.

The original logo is reusable from repository-root Markdown:

```html
<img src="website/assets/mirket-logo.png" width="80" alt="Mirket">
```

[Asset provenance](assets/PROVENANCE.md) retains its source, hash and rights
context. `logo.png` preserves the existing public `/logo.png` URL and must
remain byte-identical to `assets/mirket-logo.png`. The former site’s host, runtime
and release claims were not reused.

## Deployment

Deployment requires explicit authorization. Source edits alone do not publish.

1. Use a reviewed, committed source revision. Connect to `admin@mirket.dev` and
   perform deployment operations as the `deploy` user. Stage into a unique,
   never-overwritten release directory; staging needs no global lock.
2. Stage the committed `website/` contents as the web root in
   `/opt/harness.mirket.dev/releases/<full-commit-SHA>/`. Do not overwrite an
   existing release. Generate deployment-only `build.json` with the site/source
   identity and `files` mapping relative file paths to SHA-256 hashes. Keep the
   manifest out of source; its identity is known after commit.
3. Check the staged bytes against the source and manifest. Preserve permissions,
   nginx configuration and the previous release. nginx serves
   `/opt/harness.mirket.dev/current` and uses internal `404.html` for errors.
4. Acquire the exclusive `/opt/harness.mirket.dev/deploy.lock`, check the
   current symlink and retain its target for rollback. Atomically replace
   `current` with the verified release, then release the lock.
5. Verify live HTTPS status, content types and hashes for the homepage,
   stylesheet, both logo URLs, legal page, robots and sitemap against the staged
   manifest.
   Request a missing nested path and confirm HTTP 404 with the custom page and
   working root-relative assets. Verify browser layout, navigation and the
   existing security headers. Avoid stale cache responses when comparing bytes.

If verification fails, atomically restore `current` to the retained previous
release under the deployment lock and verify live recovery. Keep both releases
and the deployment evidence; do not delete the rollback release during promotion.
