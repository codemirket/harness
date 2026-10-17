# Design tools runtime dependencies

The fixed production dependency versions and transitive package integrity hashes are in `package-lock.json`. The runtime image installs them with `npm ci --omit=dev --ignore-scripts`.

| Direct package | Version | License (package metadata) |
| --- | --- | --- |
| @modelcontextprotocol/sdk | 1.32.1 | MIT |
| @resvg/resvg-js | 2.6.2 | MPL-2.0 |
| ajv | 8.17.1 | MIT |
| hyperframes | 0.8.149 | Apache-2.0 |
| odiff-bin | 4.5.0 | MIT |
| playwright-core | 1.64.1 | Apache-2.0 |
| satori | 0.62.0 | MPL-2.0 |
| saxes | 6.0.0 | ISC |
| style-dictionary | 5.7.0 | Apache-2.0 |
| svgo | 4.1.0 | MIT |

The image also installs Typst 0.15.1 from its official release archive after SHA-256 verification, and Debian Chromium, FFmpeg, and fonts packages. Typst is Apache-2.0; Debian package licenses are provided in `/usr/share/doc` inside the image. See each installed npm package's `LICENSE` file for complete license terms and notices.
