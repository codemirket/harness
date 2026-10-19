import fs from 'node:fs/promises';
import {Resvg} from '@resvg/resvg-js';
const [input, output, json] = process.argv.slice(2);
const {width, height, fonts} = JSON.parse(json);
const source = await fs.readFile(input, 'utf8');
// The pinned image already ships these fonts. Restrict discovery to that owned
// directory so ordinary SVG text has a fallback without scanning host fonts.
const font = {
  loadSystemFonts: false,
  fontFiles: fonts,
  fontDirs: ['/usr/share/fonts/truetype/dejavu'],
  defaultFontFamily: 'DejaVu Sans',
  serifFamily: 'DejaVu Serif',
  sansSerifFamily: 'DejaVu Sans',
  monospaceFamily: 'DejaVu Sans Mono',
};
const intrinsic = new Resvg(source, {font});
if (!(intrinsic.width > 0 && intrinsic.height > 0)) throw Error('SVG has no usable dimensions');
const zoom = Math.min(width / intrinsic.width, height / intrinsic.height);
const renderer = new Resvg(source, {font, fitTo: {mode: 'zoom', value: zoom}});
if (renderer.width < 1 || renderer.height < 1 || renderer.width * renderer.height > 16 * 1024 * 1024 || renderer.width > width || renderer.height > height) throw Error('SVG exceeds preview pixel bounds');
await fs.writeFile(output, renderer.render().asPng(), {mode: 0o600});
