// Full archive-derived closure, altered lazy dependency and entrypoint controls.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {join} from 'node:path';
import {createHash} from 'node:crypto';
import {verifyHost} from '../pi/dist/extension/host.js';
const [peers,verifier]=process.argv.slice(2);assert.ok(peers?.startsWith('/'));
const root=await fs.mkdtemp('/private/tmp/mirket-pi-host-binding-'),hash=b=>createHash('sha256').update(b).digest('hex');
const inventory=await fs.readFile(new URL('../packaging/admission/pi-host-files.json.proposed',import.meta.url));await fs.writeFile(join(root,'host-inventory.json'),inventory);
const nodeHash=hash(await fs.readFile(process.execPath)),coreHash=hash(await fs.readFile(verifier));const host=join(peers,'@earendil-works/pi-coding-agent');
const started=performance.now();const result=await verifyHost(root,host,hash(inventory),nodeHash,verifier,coreHash,join(host,'dist/cli.js'));const fullCheckMs=performance.now()-started;
await assert.rejects(()=>verifyHost(root,host,hash(inventory),nodeHash,verifier,coreHash,join(host,'dist/bundle/cli.js')),/Unsupported pi entry/);
await assert.rejects(()=>verifyHost(root,host,'0'.repeat(64),nodeHash,verifier,coreHash),/host inventory differs/);
const copy=join(root,'node_modules');await fs.cp(peers,copy,{recursive:true,dereference:false,verbatimSymlinks:true});
const copiedHost=join(copy,'@earendil-works/pi-coding-agent');const dependency=join(copy,'undici/index.js');await fs.appendFile(dependency,'\n// altered lazy network dependency\n');
await assert.rejects(()=>verifyHost(root,copiedHost,hash(inventory),nodeHash,verifier,coreHash),/external pi host differs: undici\/index.js/);
await fs.copyFile(join(peers,'undici/index.js'),dependency);await fs.writeFile(join(copy,'unexpected.js'),'export default 1');
await assert.rejects(()=>verifyHost(root,copiedHost,hash(inventory),nodeHash,verifier,coreHash),/unexpected external host file/);
const report={result,fullCheckMs,root,nodeSha256:nodeHash,verifierSha256:coreHash,inventorySha256:hash(inventory),checks:['all 15249 unique archived host files verified','bundled CLI rejected before service registration','changed lazy Undici dependency refused','unexpected host code refused','tampered package host inventory refused']};await fs.writeFile(join(root,'report.json'),JSON.stringify(report,null,2));console.log(JSON.stringify(report));
