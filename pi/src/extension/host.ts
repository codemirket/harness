// Exact external host identity. Never install or repair a host while loading.
import {createHash} from 'node:crypto';
import {realpathSync} from 'node:fs';
import {dirname,join,resolve,basename} from 'node:path';
import {execFile} from 'node:child_process';
import {promisify} from 'node:util';
import {regularBytes} from './runtime.js';
const execute=promisify(execFile);
const hash=(bytes:Buffer)=>createHash('sha256').update(bytes).digest('hex');
export async function verifyHost(packageRoot:string,hostPackage:string,expectedManifest:string,expectedNode:string,verifier:string,expectedCore:string,entry=process.argv[1]):Promise<{root:string;entry:string;lockSha256:string;verifiedFiles:number}> {
 if(hash(regularBytes(process.execPath,256*1024*1024))!==expectedNode)throw new Error('Pi must use the explicitly qualified Node executable');
 if(hash(regularBytes(verifier,256*1024*1024))!==expectedCore)throw new Error('Host verifier identity mismatch');
 const root=dirname(dirname(hostPackage));
 if(realpathSync(root)!==resolve(root)||hostPackage!==join(root,'@earendil-works/pi-coding-agent'))throw new Error('Host package path is aliased or unexpected');
 const qualifiedEntry=join(hostPackage,'dist/cli.js');
 // CLI/TUI/RPC must use the unbundled entry. SDK fixture callers are external
 // consumers; this check is not containment of arbitrary caller JavaScript.
 if(entry&&(basename(entry)==='cli.js'||resolve(entry).startsWith(hostPackage+'/'))&&realpathSync(entry)!==qualifiedEntry)throw new Error('Unsupported pi entry; select the qualified unbundled dist/cli.js');
 const {stdout}=await execute(verifier,['verify-pi-host','--root',root,'--inventory',join(packageRoot,'host-inventory.json'),'--sha256',expectedManifest],{timeout:15000,maxBuffer:16384});
 const result=JSON.parse(stdout);
 if(result.root!==root||result.entry!==qualifiedEntry||result.verifiedFiles!==15249)throw new Error('Host verification response differs');
 return result;
}
