import type { ExtensionCommandContext, ExtensionAPI } from '@earendil-works/pi-coding-agent';
import { readRuntimeSpecs, runtimeToolStatus, bindRuntime, installRuntime, skipRuntime, type ToolSpec } from './runtime-tools.js';
import type { VerifiedRuntime } from './runtime.js';
/** One Pi-native flow; only the user command can change runtime selections. */
export function createOnboarding(pi: ExtensionAPI, packageRoot: string, current: () => VerifiedRuntime | undefined) {
    let active: AbortController | undefined;
    async function setup(args: string, ctx: ExtensionCommandContext): Promise<void> {
        if (args.trim() === 'cancel') {
            active?.abort(new Error('Setup cancelled'));
            return;
        }
        const runtime = current();
        if (!runtime)
            throw new Error('Mirket core must be active before runtime setup');
        if (active)
            throw new Error('Runtime setup already active; use /mirket setup cancel');
        if (args.trim())
            throw new Error('Usage: /mirket setup');
        if (ctx.mode !== 'tui' || !ctx.hasUI)
            throw new Error('Run /mirket setup in the Pi terminal interface to select and authorize runtime tools');
        const specs = readRuntimeSpecs(packageRoot), controller = new AbortController();
        active = controller;
        const unsubscribe = ctx.ui.onTerminalInput(data => { if (data === '\x03') {
            controller.abort(new Error('Setup cancelled'));
            return { consume: true };
        } return undefined; });
        const progress = (message: string) => { ctx.ui.setStatus('mirket-setup', message); ctx.ui.setWidget('mirket-setup', [message, 'Ctrl+C cancels this setup operation.']); };
        try {
            for (const spec of specs) {
                let complete = false;
                while (!complete) {
                    controller.signal.throwIfAborted();
                    const status = runtimeToolStatus(runtime.home, runtime.binding.releaseId, [spec])[0]!;
                    ctx.ui.setWidget('mirket-setup', [`${spec.title}: ${String(status.status)}`, spec.terms, 'Select a choice. Existing tools and unrelated Pi settings are preserved.']);
                    const install = `Install pinned ${spec.version}${spec.archive ? '' : ' (unavailable)'}`;
                    const choice = await ctx.ui.select(spec.title, [install, 'Use an installed executable', 'Skip', 'Cancel setup'], { signal: controller.signal });
                    if (choice === undefined || choice === 'Cancel setup') {
                        controller.abort(new Error('Setup cancelled'));
                        controller.signal.throwIfAborted();
                    }
                    if (choice === 'Skip') {
                        skipRuntime(runtime.home, runtime.binding.releaseId, spec.id);
                        complete = true;
                        continue;
                    }
                    try {
                        if (choice === install) {
                            if (!spec.archive)
                                throw new Error(spec.unavailable ?? 'No verified publisher archive is configured. Select an installed executable or Skip.');
                            const accepted = await ctx.ui.confirm(`Install ${spec.title}?`, `${spec.terms}\n\nDownload: ${spec.archive.url}\nBytes: ${spec.archive.bytes}\nSHA-256: ${spec.archive.sha256}\nScope: private runtime files under ${runtime.home}; then an isolated local capability test. No upstream installer or hook runs.`, { signal: controller.signal });
                            if (!accepted)
                                continue;
                            await installRuntime(runtime.home, runtime.binding.releaseId, spec, controller.signal, progress);
                        }
                        else {
                            const file = await ctx.ui.input(`Absolute path to ${spec.title}`, undefined, { signal: controller.signal });
                            if (!file)
                                continue;
                            const accepted = await ctx.ui.confirm(`Bind ${spec.title}?`, `${spec.terms}\n\nSelected executable: ${file}\nMirket will execute this tool in a disposable private test directory, record its bytes and recheck them before jobs. The tool is not sandboxed. This grants no project output operation.`, { signal: controller.signal });
                            if (!accepted)
                                continue;
                            await bindRuntime(runtime.home, runtime.binding.releaseId, spec, file, controller.signal, progress);
                        }
                        complete = true;
                        ctx.ui.notify(`${spec.title} is ready. Reload Pi to reconnect the runtime.`, 'info');
                    }
                    catch (error) {
                        if (controller.signal.aborted)
                            throw error;
                        ctx.ui.notify(`${spec.title}: ${error instanceof Error ? error.message : String(error)}`, 'error');
                        const next = await ctx.ui.select('Runtime setup failed', ['Retry', 'Skip', 'Cancel setup'], { signal: controller.signal });
                        if (next === 'Skip') {
                            skipRuntime(runtime.home, runtime.binding.releaseId, spec.id);
                            complete = true;
                        }
                        else if (next !== 'Retry') {
                            controller.abort(new Error('Setup cancelled'));
                            throw controller.signal.reason;
                        }
                    }
                }
            }
            pi.sendMessage({ customType: 'mirket-setup', content: JSON.stringify({ tools: runtimeToolStatus(runtime.home, runtime.binding.releaseId, specs), reloadRequired: true }, null, 2), display: true });
        }
        catch (error) {
            if (controller.signal.aborted)
                ctx.ui.notify('Mirket runtime setup cancelled. Completed choices were retained; unfinished downloads were removed.', 'warning');
            else
                throw error;
        }
        finally {
            unsubscribe();
            ctx.ui.setStatus('mirket-setup', undefined);
            ctx.ui.setWidget('mirket-setup', undefined);
            active = undefined;
        }
    }
    return { setup, cancel: () => active?.abort(new Error('Session ended')), status: () => { const runtime = current(); if (!runtime)
            return []; try {
            return runtimeToolStatus(runtime.home, runtime.binding.releaseId, readRuntimeSpecs(packageRoot));
        }
        catch (error) {
            return [{ status: 'changed', error: error instanceof Error ? error.message : String(error) }];
        } } };
}
