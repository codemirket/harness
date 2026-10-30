import type {ExtensionAPI, ExtensionContext} from '@earendil-works/pi-coding-agent';
import {createHash} from 'node:crypto';
import {realpathSync} from 'node:fs';
import {homedir} from 'node:os';
import {dirname, isAbsolute, join, resolve} from 'node:path';
import {fileURLToPath} from 'node:url';
import {MIRKET_TOOL_CLASSES} from './authorization.js';
import {configuredOverride, regularBytes, type VerifiedRuntime} from './runtime.js';

type Use = {kind:'skill'|'tool'; id:string};
interface Selection {id:string; path:string; body:string}
const hash = (text:string|Buffer) => createHash('sha256').update(text).digest('hex');
const administrative = new Set(['mcp__mirket__runtime_status', 'mcp__mirket__memory_status', 'mcp__mirket__job_status', 'mcp__mirket__job_cancel', 'mcp__mirket__video_status', 'mcp__mirket__video_cancel']);

// Pi has no skill-activation event. Correlate its raw slash input with the exact
// expanded user message, including queued steering/follow-up messages. A pasted
// skill block, discovery result or failed read is not activation evidence.
export function registerProjectObservation(pi:ExtensionAPI, packageRoot:string, currentRuntime:()=>VerifiedRuntime|undefined, record:(runtime:VerifiedRuntime,use:Use)=>boolean):void {
  if (process.env.MIRKET_MANAGED_CHILD) return;
  const root = resolve(packageRoot);
  const pending = new Map<string,{runtime:VerifiedRuntime; selection:Selection}>();
  const seen = new Set<string>();
  let active = true, observedRuntime:VerifiedRuntime|undefined;
  function runtime(ctx:ExtensionContext):VerifiedRuntime|undefined {
    if (!active || ctx.signal?.aborted) return;
    const selected = currentRuntime();
    if (!selected || realpathSync(ctx.cwd) !== selected.cwd) return;
    if (observedRuntime !== selected) {seen.clear(); observedRuntime = selected;}
    return selected;
  }
  function selectedSkill(id:string):Selection|undefined {
    if (!/^[a-z0-9]+(?:-[a-z0-9]+)*$/u.test(id)) return;
    const commands = pi.getCommands().filter(command => command.name === `skill:${id}`);
    const path = join(root,'skills',id,'SKILL.md');
    if (commands.length !== 1 || commands[0]!.source !== 'skill' || commands[0]!.sourceInfo.path !== path
        || realpathSync(root) !== root || realpathSync(path) !== path) return;
    const catalog = JSON.parse(regularBytes(join(root,'skills/catalog.json'),2*1024*1024).toString()) as {global:unknown[];project:unknown[]};
    if (!Array.isArray(catalog.global) || !Array.isArray(catalog.project) || catalog.global.length+catalog.project.length > 256) return;
    const entries = [...catalog.global,...catalog.project].filter(value => value && typeof value === 'object' && (value as {id?:unknown}).id === id) as {path?:unknown;source_sha256?:unknown;source_bytes?:unknown}[];
    const bytes = regularBytes(path,256*1024);
    if (entries.length !== 1 || entries[0]!.path !== `skills/${id}/SKILL.md` || entries[0]!.source_bytes !== bytes.length || entries[0]!.source_sha256 !== hash(bytes)) return;
    // Match the admitted Pi 1.4.0 expansion's BOM/newline/frontmatter handling.
    const source = bytes.toString('utf8').replace(/^\uFEFF/u,'').replace(/\r\n?/gu,'\n');
    const end = source.startsWith('---') ? source.indexOf('\n---',3) : -1;
    const body = (end < 0 ? source : source.slice(end+4)).trim();
    return {id,path,body};
  }
  function observe(selected:VerifiedRuntime,use:Use,ctx:ExtensionContext):void {
    if (currentRuntime() !== selected || !runtime(ctx)) return;
    const key = `${use.kind}:${use.id}`;
    if (seen.has(key)) return;
    try {if (record(selected,use)) seen.add(key);}
    catch (error) {
      // Guidance maintenance must not turn an already successful tool into an error.
      try {ctx.ui.notify(`Project guidance sync failed: ${error instanceof Error ? error.message : String(error)}`,'warning');} catch { /* No UI must not alter the result either. */ }
    }
  }
  pi.on('session_start',() => {active=true;pending.clear();seen.clear();observedRuntime=undefined;});
  pi.on('session_shutdown',() => {active=false;pending.clear();seen.clear();observedRuntime=undefined;});
  pi.on('input',(event,ctx) => {
    try {
      if (ctx.isIdle()) pending.clear();
      // A literal pasted block cannot consume a previous failed/queued invocation.
      pending.delete(hash(event.text));
      const selected = runtime(ctx);
      if (!selected || event.text.length > 65536 || !event.text.startsWith('/skill:')) return;
      const space = event.text.indexOf(' '), id = event.text.slice(7,space < 0 ? undefined : space);
      const selection = selectedSkill(id);
      if (!selection) return;
      const args = space < 0 ? '' : event.text.slice(space+1).trim();
      const block = `<skill name="${id}" location="${selection.path}">\nReferences are relative to ${dirname(selection.path)}.\n\n${selection.body}\n</skill>`;
      const expected = args ? `${block}\n\n${args}` : block;
      if (pending.size >= 16) pending.delete(pending.keys().next().value!);
      pending.set(hash(expected),{runtime:selected,selection});
    } catch { /* Missing, changed or ambiguous package resources are not evidence. */ }
  });
  pi.on('message_end',(event,ctx) => {
    try {
      if (event.message.role !== 'user' || !pending.size) return;
      const content = event.message.content;
      const text = typeof content === 'string' ? content : content.filter(part => part.type === 'text').map(part => part.text).join('');
      const key = hash(text), invocation = pending.get(key);
      if (!invocation) return;
      pending.delete(key);
      const selected = runtime(ctx);
      if (selected !== invocation.runtime || !selectedSkill(invocation.selection.id)) return;
      observe(selected,{kind:'skill',id:invocation.selection.id},ctx);
    } catch { /* Only an exact, still-selected expansion counts. */ }
  });
  pi.on('tool_result',(event,ctx) => {
    try {
      if (event.isError) return;
      const selected = runtime(ctx);
      if (!selected) return;
      const owners = pi.getAllTools().filter(tool => tool.name === event.toolName);
      if (owners.length !== 1) return;
      const owner = owners[0]!.sourceInfo.path;
      if (event.toolName === 'read' && owner === 'builtin:read' && typeof event.input.path === 'string') {
        let path = event.input.path.replace(/^@/u,'');
        if (path === '~' || path.startsWith('~/')) path = join(homedir(),path.slice(2));
        if (path.startsWith('file://')) path = fileURLToPath(path);
        path = isAbsolute(path) ? resolve(path) : resolve(ctx.cwd,path);
        for (const command of pi.getCommands()) {
          if (command.source !== 'skill' || command.sourceInfo.path !== path || !command.name.startsWith('skill:')) continue;
          const skill = selectedSkill(command.name.slice(6));
          if (skill) observe(selected,{kind:'skill',id:skill.id},ctx);
        }
        return;
      }
      if (event.toolName === 'mirket_delegate' && owner === join(root,'dist/extension/index.js') && realpathSync(owner) === owner) {
        observe(selected,{kind:'tool',id:event.toolName},ctx);return;
      }
      if (administrative.has(event.toolName) || !Object.hasOwn(MIRKET_TOOL_CLASSES,event.toolName) || owner !== 'builtin:mcp') return;
      if (configuredOverride(dirname(selected.home),selected.cwd)) return;
      const servers = pi.getMcpServers().filter(server => server.name === 'mirket');
      if (servers.length !== 1) return;
      const config = servers[0]!.config;
      if (!('command' in config) || config.command !== selected.command || config.cwd !== selected.cwd
          || JSON.stringify(config.args) !== JSON.stringify(['serve','--home',selected.home,'--expected-release-id',selected.binding.releaseId])) return;
      if (event.input.expected_release_id !== undefined && event.input.expected_release_id !== selected.binding.releaseId) return;
      const scope = event.input.scope as {kind?:unknown;project_root?:unknown}|undefined;
      const project = event.input.project_root ?? (scope?.kind === 'project' ? scope.project_root : undefined);
      if (project !== undefined && (typeof project !== 'string' || realpathSync(project) !== selected.cwd)) return;
      observe(selected,{kind:'tool',id:event.toolName},ctx);
    } catch { /* Unverified ownership/scope and stale results cannot update guidance. */ }
  });
}
