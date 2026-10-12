# Motion Canvas package provenance

Motion Canvas core and 2D are pinned to 3.17.2. The full declared production archive closure and exact package-lock source identity are retained in SOURCE-INVENTORY.json. Original archive license and notice files are retained beside this inventory.

No npm lifecycle scripts, esbuild install helper, browser download helper, Vite configuration, project package.json, or official FFmpeg exporter executes. The Linux esbuild package for the target architecture is extracted and used directly inside the reviewed Podman job. Node, Chromium and FFmpeg reuse the shared Mirket definitions and notices. Playwright uses the existing pinned archive and retained optional package notices.

Sources: https://github.com/motion-canvas/motion-canvas/tree/e735a995aa738ca682feb4204a39ee7db7b5d279 and exact archive URLs in registry/packages-motion-canvas.json.

Inter Variable 4.1 is retained under its original SIL Open Font License. mj-context-menu 0.6.1 omits a standalone license file in both its published archive and pinned repository: the exact published Apache-2.0 license metadata and canonical Apache text are retained, with retrieval sources and digests in the inventory. The Linux esbuild archives omit their license file; the exact upstream MIT license is retained in `../container-runtime/ESBUILD-LICENSE.md` with pin provenance.
