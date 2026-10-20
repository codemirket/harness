// Pin the official file-mode CLI; no desktop discovery or remote asset downloads.
globalThis.fetch = async () => { throw Error('OpenPencil network access is disabled; embed assets and use bundled/local fonts'); };
if (process.argv[2] === 'tool' && process.argv[3] === 'call') {
  const {ALL_TOOLS} = await import('@open-pencil/core/tools');
  const {parse} = await import('valibot');
  const tool = ALL_TOOLS.find(tool => tool.name === process.argv[4]);
  const index = process.argv.indexOf('--args');
  if (!tool || index < 0) throw Error('Missing validated OpenPencil tool arguments');
  // The CLI's file-mode path calls execute directly. Apply the upstream input
  // validator (including transformations) before it reaches that path.
  process.argv[index+1] = JSON.stringify(parse(tool.input, JSON.parse(process.argv[index+1])));
}
const {COMMANDS} = await import('@open-pencil/cli/commands');
const {runMain} = await import('citty');
await runMain({meta:{name:'openpencil',version:'0.16.0'},subCommands:COMMANDS});
