# Design tools runtime dependencies

The fixed production dependency versions and transitive package integrity hashes are in `package-lock.json`. The runtime image installs them with `npm ci --omit=dev --ignore-scripts`.

| Direct package | Version | License (package metadata) |
| --- | --- | --- |
| @open-pencil/cli | 0.16.0 | MIT |
| @modelcontextprotocol/sdk | 1.32.1 | MIT |
| @resvg/resvg-js | 2.6.2 | MPL-2.0 |
| ajv | 8.18.0 | MIT |
| css-tree | 3.2.1 | MIT |
| odiff-bin | 4.5.0 | MIT |
| playwright-core | 1.64.1 | Apache-2.0 |
| satori | 0.62.0 | MPL-2.0 |
| sharp | 0.35.5 | Apache-2.0 |
| saxes | 6.0.0 | ISC |
| style-dictionary | 5.7.0 | Apache-2.0 |
| svgo | 4.1.0 | MIT |

The image also installs Typst 0.15.1 from its official release archive after SHA-256 verification, and Debian Chromium and fonts packages. Typst is Apache-2.0; Debian package licenses are provided in `/usr/share/doc` inside the image. See each installed npm package's `LICENSE` file for complete license terms and notices.

OpenPencil uses its official published CLI, engine, .fig/.pen codecs, CanvasKit and
Yoga WASM dependencies, all pinned with integrity hashes in the npm lock. No
OpenPencil application source is patched. The file-mode wrapper applies upstream
validation, filesystem permissions and offline asset handling. See installed
package licenses (including bundled font assets) for full terms.

The transitive `image-size` package is overridden to 2.0.4 to address
GHSA-5p2g-fcmc-qvqq and GHSA-w3rx-r6r6-pgpr. The supported OpenPencil file/export
paths are covered by regression tests; PowerPoint export is not exposed by Mirket.

## OpenPencil schema adaptation license

The reviewed tool descriptors are derived from the pinned OpenPencil release.

```text
MIT License

Copyright (c) 2026 Danila Poyarkov and OpenPencil contributors

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```
