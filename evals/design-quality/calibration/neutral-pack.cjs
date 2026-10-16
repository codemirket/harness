#!/usr/bin/env node
/* MIT. Keep image identities private during render-only calibration. */
'use strict';
const fs = require('node:fs/promises');
const path = require('node:path');
const crypto = require('node:crypto');
const { pack } = require('./pack.cjs');
const sha = bytes => crypto.createHash('sha256').update(bytes).digest('hex');

async function neutralPack(observations, destination) {
  const out = path.resolve(destination);
  try {
    await fs.lstat(out);
    throw new Error('Refusing an existing calibration destination');
  } catch (error) {
    if (error.code !== 'ENOENT') throw error;
  }
  const key = await pack(observations, out);
  for (const [id, identity] of Object.entries(key.packets)) {
    const file = path.join(out, id, 'packet.json');
    const packet = JSON.parse(await fs.readFile(file, 'utf8'));
    for (const pair of packet.pairs) for (const artifact of pair.artifacts) {
      artifact.images = artifact.images.map(image => ({ path: image.path }));
    }
    const bytes = Buffer.from(JSON.stringify(packet, null, 2) + '\n');
    await fs.writeFile(file, bytes);
    identity.packet_sha256 = sha(bytes);
  }
  key.identity_metadata = {
    location: 'Private key only; judges receive paths and pixels.',
    reason: 'Visible repeated hashes can reveal duplicate identity before perceptual scoring.',
    original_packer_sha256: sha(await fs.readFile(path.join(__dirname, 'pack.cjs'))),
    neutral_packer_sha256: sha(await fs.readFile(__filename)),
    limits: 'This removes a metadata cue; it does not establish blinding or perceptual truth.'
  };
  await fs.writeFile(path.join(out, 'key.json'), JSON.stringify(key, null, 2) + '\n');
  return { packets: Object.keys(key.packets), key: path.join(out, 'key.json'),
    synthetic: true, model_executions: 0, visible_image_identity_metadata: false };
}

if (require.main === module) {
  const [observations, destination, ...extra] = process.argv.slice(2);
  if (!observations || !destination || extra.length) {
    console.error('Usage: node neutral-pack.cjs OBSERVATIONS_DIR NEW_OUTPUT_DIR');
    process.exitCode = 1;
  } else neutralPack(observations, destination).then(result => console.log(JSON.stringify(result)))
    .catch(error => { console.error(error.stack); process.exitCode = 1; });
}
module.exports = { neutralPack };
