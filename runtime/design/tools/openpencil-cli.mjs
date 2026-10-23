// Pin the official file-mode CLI; no desktop discovery or remote asset downloads.
import {createRequire} from 'node:module';
import fs from 'node:fs/promises';
globalThis.fetch = async () => { throw Error('OpenPencil network access is disabled; embed assets and use bundled/local fonts'); };
const transaction = process.argv[2] === 'transaction';
if (transaction) {
  // Reuse the official file-mode lifecycle: it loads once, invokes this
  // transaction in memory, and saves once. The parent reopens before publish.
  const {validateOpenPencilBatch, assertOpenPencilSuccess} = await import('./openpencil.mjs');
  const {ALL_TOOLS} = await import('@open-pencil/core/tools');
  const {parse} = await import('valibot');
  const [input, output, request] = process.argv.slice(3);
  const operations = validateOpenPencilBatch(JSON.parse(await fs.readFile(request, 'utf8')));
  const prepared = operations.map(operation => {
    const tool = ALL_TOOLS.find(tool => tool.name === operation.tool);
    if (!tool || !tool.capabilities.every(cap => ['document:read','document:write'].includes(cap))) throw Error('Unsupported OpenPencil transaction capability');
    return {tool, input:parse(tool.input, operation.arguments), execute:tool.execute};
  });
  const first = prepared[0];
  const signature = JSON.stringify(first.input);
  first.tool.execute = async (...parameters) => {
    // The pinned CLI supplies the validated input and its file-mode context.
    // Locate the input by exact value rather than assuming an undocumented
    // argument order. Fail closed if the upstream calling convention changes.
    const matches = parameters.flatMap((value, index) => {
      try { return JSON.stringify(value) === signature ? [index] : []; } catch { return []; }
    });
    if (matches.length !== 1) throw Error('Unsupported OpenPencil execute calling convention');
    const results = [];
    for (const operation of prepared) {
      const invocation = [...parameters];
      invocation[matches[0]] = operation.input;
      const result = await operation.execute.apply(operation.tool, invocation);
      assertOpenPencilSuccess(result);
      results.push(result);
    }
    return {operation_results:results, operation_count:operations.length};
  };
  process.argv.splice(2, process.argv.length - 2, 'tool', 'call', first.tool.name, input, '--args', signature, '--output', output, '--json');
}
if (process.argv[2] === 'tool' && process.argv[3] === 'call') {
  const {ALL_TOOLS} = await import('@open-pencil/core/tools');
  const {parse} = await import('valibot');
  const tool = ALL_TOOLS.find(tool => tool.name === process.argv[4]);
  const index = process.argv.indexOf('--args');
  if (!tool || index < 0) throw Error('Missing validated OpenPencil tool arguments');
  process.argv[index+1] = JSON.stringify(parse(tool.input, JSON.parse(process.argv[index+1])));
}
const {COMMANDS} = await import('@open-pencil/cli/commands');
const fromCli = createRequire(import.meta.resolve('@open-pencil/cli/commands'));
const {runMain} = fromCli('citty');
await runMain({meta:{name:'openpencil',version:'0.16.0'},subCommands:COMMANDS});
