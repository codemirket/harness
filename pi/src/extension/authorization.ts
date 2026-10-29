import { createHash, randomUUID } from 'node:crypto';
import { mkdirSync, renameSync, unlinkSync, rmdirSync, writeFileSync, realpathSync, lstatSync } from 'node:fs';
import { join } from 'node:path';
import { regularBytes, type VerifiedRuntime } from './runtime.js';
interface Grant { key: string; expires: number }
interface Grants { schema: 1; grants: Grant[] }
// Maximum effects from the explicit MCP taxonomy. A new native route must
// be classified here before the Pi extension will dispatch it.
export const MIRKET_TOOL_CLASSES = Object.freeze({
  mcp__mirket__runtime_status: 'O0',
  mcp__mirket__code_inspect: 'O1',
  mcp__mirket__code_navigate: 'O1',
  mcp__mirket__diagram_render_prepare: 'P1',
  mcp__mirket__diagram_render_start: 'F1',
  mcp__mirket__design_prepare: 'P1',
  mcp__mirket__design_start: 'F1',
  mcp__mirket__video_prepare: 'P1',
  mcp__mirket__video_start: 'F1',
  mcp__mirket__video_status: 'O2',
  mcp__mirket__video_cancel: 'K1',
  mcp__mirket__video_result: 'O2',
  mcp__mirket__job_status: 'O2',
  mcp__mirket__job_cancel: 'K1',
  mcp__mirket__memory_prepare: 'P2',
  mcp__mirket__memory_commit: 'M1',
  mcp__mirket__memory_index_refresh: 'I1',
  mcp__mirket__memory_status: 'O3',
  mcp__mirket__memory_forget_prepare: 'P3',
  mcp__mirket__memory_forget_commit: 'X1',
  mcp__mirket__memory_forget_reconcile: 'X2',
  mcp__mirket__memory_search: 'O4',
  mcp__mirket__memory_list: 'O3',
  mcp__mirket__memory_read: 'O3',
  mcp__mirket__memory_history: 'O3',
} as const);
export function classifyMirketTool(tool: string): (typeof MIRKET_TOOL_CLASSES)[keyof typeof MIRKET_TOOL_CLASSES] {
  if (!Object.prototype.hasOwnProperty.call(MIRKET_TOOL_CLASSES, tool)) throw new Error(`Unclassified Mirket tool: ${tool}`);
  return MIRKET_TOOL_CLASSES[tool as keyof typeof MIRKET_TOOL_CLASSES];
}
const restricted = new Set(['mcp__mirket__memory_forget_commit','mcp__mirket__memory_forget_reconcile']);
export function needsAuthorization(tool: string, args: Record<string, unknown>): boolean {
  classifyMirketTool(tool);
  const scope = args.scope as {kind?:string} | undefined;
  return restricted.has(tool) || (tool.startsWith('mcp__mirket__memory_') && scope?.kind === 'shared_preferences');
}
function canonical(value: unknown): string {
  if (Array.isArray(value)) return '['+value.map(canonical).join(',')+']';
  if (value && typeof value === 'object') return '{'+Object.keys(value).sort().map(k=>JSON.stringify(k)+':'+canonical((value as Record<string,unknown>)[k])).join(',')+'}';
  const result = JSON.stringify(value);if (result === undefined) throw new Error('Authorization needs JSON arguments');return result;
}
function key(runtime: VerifiedRuntime, tool: string, args: Record<string, unknown>): string {
  return createHash('sha256').update(canonical({release:runtime.binding.releaseId,project:runtime.cwd,tool,args})).digest('hex');
}
function read(home: string): Grants {
  let data: Grants;
  try { data = JSON.parse(regularBytes(join(home,'authorizations.json'),256*1024).toString()); }
  catch(error) { if ((error as NodeJS.ErrnoException).code === 'ENOENT') return {schema:1,grants:[]};throw error; }
  if (data.schema !== 1 || !Array.isArray(data.grants) || data.grants.length>256 || data.grants.some(g=>!g || !/^[a-f0-9]{64}$/.test(g.key) || !Number.isSafeInteger(g.expires))) throw new Error('Invalid authorization state');
  return data;
}
export function authorized(runtime: VerifiedRuntime, tool: string, args: Record<string, unknown>, now=Date.now()): boolean {
  const target=key(runtime,tool,args);return read(runtime.home).grants.some(g=>g.key===target && g.expires>now && g.expires<=now+300000);
}
export function grant(runtime: VerifiedRuntime, tool: string, args: Record<string, unknown>, now=Date.now()): void {
  if (!needsAuthorization(tool,args)) throw new Error('This operation needs no extra authorization');
  if (realpathSync(runtime.home)!==runtime.home || (lstatSync(runtime.home).mode&0o077)!==0) throw new Error('Authorization requires private runtime home');
  const lock=join(runtime.home,'authorizations.lock');mkdirSync(lock,{mode:0o700});
  const next=join(runtime.home,`authorizations-${randomUUID()}.next`);let created=false;
  try {
    const target=key(runtime,tool,args),data=read(runtime.home);data.grants=data.grants.filter(g=>g.expires>now && g.key!==target);
    if (data.grants.length>=256) throw new Error('Authorization limit reached');
    data.grants.push({key:target,expires:now+300000});writeFileSync(next,JSON.stringify(data)+'\n',{flag:'wx',mode:0o600});created=true;
    renameSync(next,join(runtime.home,'authorizations.json'));created=false;
  } finally { if(created)unlinkSync(next);rmdirSync(lock); }
}
