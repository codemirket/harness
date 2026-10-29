import { getAgentDir, getPackageDir, VERSION, type ExtensionAPI } from '@earendil-works/pi-coding-agent';
import { Type } from 'typebox';
import { fileURLToPath } from 'node:url';
import { realpathSync } from 'node:fs';
import { join } from 'node:path';
import { prepareProfile, applyProfile } from './profiles.js';
import { verifyHost } from './host.js';
import { readBinding, configuredOverride, verifyRuntime, type VerifiedRuntime } from './runtime.js';
import { classifyMirketTool, needsAuthorization, authorized, grant } from './authorization.js';
import { registerResearch } from './research.js';
import { registerWorkflow } from './workflow.js';
import {createOnboarding} from './onboarding.js';
import {assertRuntimeTools, runtimeBrowserBindings} from './runtime-tools.js';
import {createBrowserController} from './browser.js';
import {browserOrigin} from './browser-policy.js';

const packageRoot = fileURLToPath(new URL('../../', import.meta.url));
export default function mirket(pi: ExtensionAPI): void {
  let generation = 0;
  let runtime: VerifiedRuntime | undefined;
  let failure = 'No active session';
  let observedRelease: string | undefined;
  let nativeFailure: string | undefined;
  const onboarding=createOnboarding(pi,packageRoot,()=>runtime);
  const browser=createBrowserController(pi,()=>runtime,getAgentDir);
  registerResearch(pi,()=>runtime);
  registerWorkflow(pi,packageRoot,()=>runtime);
  function report() {
    const commands = pi.getCommands();
    const mcp = commands.find(command => command.name === 'mcp');
    const tool = pi.getAllTools().find(tool => tool.name === 'mcp__mirket__runtime_status');
    const registration = pi.getMcpServers().find(server => server.name === 'mirket');
    const issues: string[] = [];
    if (!runtime) issues.push(failure);
    if (nativeFailure) issues.push(nativeFailure);
    if (!mcp) issues.push('MCP disabled or missing');
    else if (mcp.sourceInfo.path !== 'builtin:mcp') issues.push('MCP command replaced');
    if (runtime) {
      const override = configuredOverride(getAgentDir(), runtime.cwd);
      if (override) issues.push(`Same-name MCP configuration: ${override}`);
      if (!registration || !('command' in registration.config) || registration.config.command !== runtime.command
          || registration.config.cwd !== runtime.cwd) issues.push('MCP registration missing or changed');
      if (!tool || !pi.getActiveTools().includes(tool.name)) issues.push('Runtime status unavailable or filtered');
      else if (tool.sourceInfo.path !== 'builtin:mcp') issues.push('Runtime status has unexpected owner');
      if (observedRelease !== runtime.binding.releaseId) issues.push('Awaiting actual runtime_status identity');
    }
    return {status: issues.length ? 'not_ready' : 'ready', issues, releaseId: runtime?.binding.releaseId, project: runtime?.cwd,
      runtimeTools:onboarding.status(),
      browser:browser.status(),
      selectedSkills: pi.getCommands().filter(command => command.source === 'skill' && command.sourceInfo.path.startsWith(packageRoot)).map(command => command.name), model: 'host-owned', nativeOnly: false};
  }
  pi.on('session_start', async (_event, ctx) => {
    onboarding.cancel();
    const current = ++generation;
    runtime = undefined; observedRelease = undefined; nativeFailure = undefined;
    await browser.revoke('Pi session changed').catch(error => { ctx.ui.notify(`Browser cleanup failed: ${String(error)}`, 'warning'); });
    pi.unregisterMcpServer('mirket');
    try {
      const binding = readBinding(packageRoot);
      const [,verified] = await Promise.all([
        verifyHost(packageRoot,getPackageDir(),binding.hostInventorySha256,binding.nodeSha256,join(getAgentDir(),'mirket/bin/mirket'),binding.coreSha256),
        verifyRuntime(packageRoot,getAgentDir(),ctx.cwd,VERSION),
      ]);
      const override = configuredOverride(getAgentDir(), verified.cwd);
      if (override) throw new Error(`Same-name MCP configuration: ${override}`);
      if (current !== generation) return;
      runtime = verified;
      pi.registerMcpServer('mirket', {command: verified.command,
        args: ['serve', '--home', verified.home, '--expected-release-id', verified.binding.releaseId],
        cwd: verified.cwd, exposure: 'deferred', toolExposure: {runtime_status: 'direct'}});
      failure = '';
    } catch (error) {
      if (current !== generation) return;
      failure = error instanceof Error ? error.message : String(error);
      ctx.ui.notify(`Mirket unavailable: ${failure}`, 'warning');
    }
  });
  pi.on('session_shutdown', async () => {
    onboarding.cancel();
    ++generation; runtime = undefined; observedRelease = undefined; nativeFailure = undefined;
    try {await browser.revoke('Pi session ended');}
    finally {pi.unregisterMcpServer('mirket');}
  });
  pi.on('tool_call', async (event, ctx) => {
    if (!event.toolName.startsWith('mcp__mirket__')) return;
    try {
      classifyMirketTool(event.toolName);
      if (!runtime || realpathSync(ctx.cwd) !== runtime.cwd) throw new Error('No runtime bound to this project');
      const override = configuredOverride(getAgentDir(), runtime.cwd);
      if (override) throw new Error(`Same-name MCP configuration: ${override}`);
      const input = event.input as Record<string, unknown>;
      if (['mcp__mirket__video_prepare','mcp__mirket__video_start'].includes(event.toolName)) assertRuntimeTools(runtime,['chromium']);
      const scope = input.scope as {kind?: string; project_root?: string} | undefined;
      const selected = input.project_root ?? (scope?.kind === 'project' ? scope.project_root : undefined);
      if (selected !== undefined && (typeof selected !== 'string' || realpathSync(selected) !== runtime.cwd)) {
        throw new Error('Operation project differs from the active pi project');
      }
      if (input.expected_release_id !== undefined && input.expected_release_id !== runtime.binding.releaseId) {
        throw new Error('Operation release differs from the active pi package');
      }
      if (needsAuthorization(event.toolName,input) && !authorized(runtime,event.toolName,input)) {
        // RPC can expose a dialog bridge, but it is not implicit consent. Its
        // controller must issue the explicit authorization command first.
        if (ctx.mode !== 'tui' || !ctx.hasUI) throw new Error('Explicit authorization required: /mirket-authorize {"tool":"'+event.toolName+'","args":'+JSON.stringify(input)+'}');
        const selectedRuntime=runtime,current=generation;
        const accepted=await ctx.ui.confirm('Authorize Mirket operation?',JSON.stringify({project:runtime.cwd,tool:event.toolName,args:input,expiresInSeconds:300},null,2),{timeout:60000});
        if (!accepted) throw new Error('Mirket operation authorization denied');
        if (generation!==current || runtime!==selectedRuntime) throw new Error('Session changed during authorization');
        grant(runtime,event.toolName,input);
      }
    } catch (error) { return {block: true, reason: error instanceof Error ? error.message : String(error)}; }
  });
  pi.registerCommand('mirket-authorize', {description:'Authorize one exact restricted operation for five minutes: {"tool":...,"args":...}',handler:async(args,ctx)=>{
    if (!runtime || realpathSync(ctx.cwd)!==runtime.cwd) throw new Error('No runtime bound to this project');
    const request=JSON.parse(args);
    if (!request || typeof request.tool!=='string' || !request.args || typeof request.args!=='object' || Array.isArray(request.args) || Object.keys(request).some(k=>k!=='tool'&&k!=='args')) throw new Error('Expected exact tool and args');
    grant(runtime,request.tool,request.args);
    pi.sendMessage({customType:'mirket-authorization',content:JSON.stringify({tool:request.tool,project:runtime.cwd,expiresInSeconds:300}),display:true});
  }});
  pi.registerCommand('mirket', {description: 'Mirket runtime setup and session doctor: setup | doctor', handler: async (args,ctx) => {
    if(args.trim()==='setup'||args.trim().startsWith('setup ')){await onboarding.setup(args.trim().slice(5).trim(),ctx);return;}
    if (args.trim() !== 'doctor') throw new Error('Usage: /mirket setup | /mirket doctor');
    pi.sendMessage({customType: 'mirket-doctor', content: JSON.stringify(report(), null, 2), display: true});
  }});
  pi.registerCommand('mirket-browser', {description: 'Authorize one local browser origin: open http://127.0.0.1:PORT | revoke | status', handler: async (args,ctx) => {
    const command=args.trim();
    if(command==='status') {pi.sendMessage({customType:'mirket-browser',content:JSON.stringify(browser.status(),null,2),display:true});return;}
    if(command==='revoke') {await browser.revoke('User revoked browser authorization');pi.sendMessage({customType:'mirket-browser',content:JSON.stringify(browser.status(),null,2),display:true});return;}
    if(!command.startsWith('open ')||command.slice(5).trim()!==command.slice(5)) throw new Error('Usage: /mirket-browser open http://127.0.0.1:PORT | revoke | status');
    if(ctx.mode!=='tui'||!ctx.hasUI) throw new Error('Open the scoped browser session from Pi TUI');
    const selected=runtime, current=generation;
    if(!selected||realpathSync(ctx.cwd)!==selected.cwd) throw new Error('No verified runtime bound to this project');
    const origin=browserOrigin(command.slice(5));
    const bindings=runtimeBrowserBindings(selected);
    const accepted=await ctx.ui.confirm('Authorize local browser session?',JSON.stringify({project:selected.cwd,origin,peerPath:bindings.peerPath,peerIdentity:bindings.peerIdentity,browserPath:bindings.browserPath,browserIdentity:bindings.browserIdentity,expiresInSeconds:300,maxCalls:64,network:'One literal loopback HTTP origin, GET/HEAD only. Opening the page requests its subresources; GET may change the local application. This is not an OS sandbox.'},null,2),{timeout:60000});
    if(!accepted) throw new Error('Browser authorization denied');
    if(current!==generation||runtime!==selected) throw new Error('Pi session changed during browser authorization');
    const result=await browser.open(origin,ctx,{expected:{peerPath:bindings.peerPath,peerIdentity:bindings.peerIdentity,browserPath:bindings.browserPath,browserIdentity:bindings.browserIdentity}});
    pi.sendMessage({customType:'mirket-browser',content:JSON.stringify(result,null,2),display:true});
  }});
  pi.registerCommand('mirket-profile', {description: 'Select Mirket package skills: global|project profile names; +skill selects an exact ID', handler: async (args, ctx) => {
    const [scope, ...selections] = args.trim().split(/\s+/);
    if ((scope !== 'global' && scope !== 'project') || !selections.length) throw new Error('Usage: /mirket-profile global|project PROFILE ... [+SKILL ...]');
    const file = scope === 'global' ? join(getAgentDir(), 'settings.json') : join(ctx.cwd, '.pi/settings.json');
    const edit = prepareProfile(packageRoot, file, selections.filter(s => !s.startsWith('+')), selections.filter(s => s.startsWith('+')).map(s => s.slice(1)));
    applyProfile(edit);
    pi.sendMessage({customType:'mirket-profile',content:JSON.stringify({settings:file,skills:edit.skills,reloadRequired:true}),display:true});
  }});
  pi.registerTool({name: 'mirket_doctor', label: 'Mirket doctor', description: 'Verify native payload and actual pi MCP runtime identity in this session.',
    parameters: Type.Object({}), async execute(_id, _args, signal, _onUpdate, ctx) {
      const current = generation;
      nativeFailure = undefined;
      const issues = report().issues.filter(issue => issue !== 'Awaiting actual runtime_status identity');
      observedRelease = undefined;
      if (runtime && !issues.length) {
        const result = await ctx.executeTool('mcp__mirket__runtime_status', {}, signal ? {signal} : {});
        if (result.isError && current === generation) nativeFailure = 'Native verification failed: '+result.result.content.filter(part=>part.type==='text').map(part=>part.text).join(' ').slice(0,1024);
        if (!result.isError && current === generation) {
          for (const part of result.result.content) if (part.type === 'text') {
            try { const status = JSON.parse(part.text); if (status.release_id === runtime?.binding.releaseId) observedRelease = status.release_id; } catch { /* Non-JSON is not attestation. */ }
          }
        }
      }
      const details = report();
      return {content: [{type: 'text' as const, text: JSON.stringify(details)}], details};
    }});
}
