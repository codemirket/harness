# Brand artwork

`social-preview.png` is the 1280×640 GitHub repository social preview.
Its editable Satori composition is `social-preview.json`. It uses the same
organization logo (`../public/logo.png`) and canonical `tokens.json` as the site.

To render with Mirket's installed design package, create `harness-social/` in
the package's reported workspace. Copy the logo and tokens there, and supply
licensed Arial regular and bold font files as `Arial.ttf` and `Arial-Bold.ttf`.
The fonts are rendering inputs and are not redistributed in this repository.
Compile the tokens with `tokens_compile`, then pass the composition JSON to
`static_render`. For an existing output, use its current SHA-256 and the tool's
overwrite option. Inspect the exported image before replacing the saved PNG.

Upload the PNG under GitHub repository Settings → General → Social preview.
The artwork is maintained here separately from the website's public assets.
