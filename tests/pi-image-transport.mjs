// Exact saved-PNG delivery to an offline image-capable Pi provider fixture.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {join, isAbsolute} from 'node:path';
import {pathToFileURL} from 'node:url';

const [peers, root, imageFile] = process.argv.slice(2);
assert.ok([peers, root, imageFile].every(isAbsolute));
assert.match(root, /^\/private\/tmp\/mirket-pi-package-/u);
assert.ok(imageFile.startsWith(join(root, 'project') + '/'));
const packageRoot = join(root, 'package'), agentDir = join(root, 'agent'), project = join(root, 'project');
Object.assign(process.env, {HOME: join(root, 'home'), PI_CODING_AGENT_DIR: agentDir,
  PI_CODING_AGENT_SESSION_DIR: join(root, 'sessions'), PI_SKIP_UPDATE_CHECK: '1', PI_TELEMETRY_DISABLED: '1'});
const png = await fs.readFile(imageFile);
assert.equal(png.toString('hex', 0, 8), '89504e470d0a1a0a');
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const expected = sha(png), sdk = await import(pathToFileURL(join(peers, '@earendil-works/pi-coding-agent/dist/index.js')).href);
const ai = await import(pathToFileURL(join(peers, '@earendil-works/pi-ai/dist/index.js')).href);
let delivered;
const provider = pi => pi.registerProvider('offline-image-fixture', {api: 'offline-image-fixture',
  baseUrl: 'http://invalid.local', apiKey: 'fixture',
  models: [{id: 'vision', name: 'Offline image fixture', reasoning: false, input: ['text', 'image'],
    cost: {input: 0, output: 0, cacheRead: 0, cacheWrite: 0}, contextWindow: 200000, maxTokens: 20000}],
  streamSimple(model, context) {
    const stream = ai.createAssistantMessageEventStream();
    const issued = context.messages.some(message => message.role === 'assistant');
    if (issued) {
      const parts = context.messages.flatMap(message => Array.isArray(message.content) ? message.content : []);
      delivered = parts.find(part => part.type === 'image');
    }
    const output = {role: 'assistant', api: model.api, provider: model.provider, model: model.id,
      content: issued ? [{type: 'text', text: 'Image fixture complete'}]
        : [{type: 'toolCall', id: 'read-image', name: 'read', arguments: {path: imageFile}}],
      usage: {input: 0, output: 0, cacheRead: 0, cacheWrite: 0, totalTokens: 0,
        cost: {input: 0, output: 0, cacheRead: 0, cacheWrite: 0, total: 0}},
      stopReason: issued ? 'stop' : 'toolUse', timestamp: Date.now()};
    queueMicrotask(() => {stream.push({type: 'start', partial: output});
      stream.push({type: 'done', reason: output.stopReason, message: output}); stream.end(output);});
    return stream;
  },
});
const settingsManager = sdk.SettingsManager.create(project, agentDir);
const loader = new sdk.DefaultResourceLoader({cwd: project, agentDir, settingsManager, noContextFiles: true,
  extensionFactories: [{name: 'codemode', factory: sdk.createCodemodeExtension(), builtin: true, replaceable: true},
    {name: 'tool-search', factory: sdk.createToolSearchExtension(), builtin: true, replaceable: true},
    {name: 'mcp', factory: sdk.createMcpExtension(), builtin: true, replaceable: true}, provider]});
await loader.reload(); assert.deepEqual(loader.getExtensions().errors, []);
assert.ok(loader.getExtensions().extensions.some(extension => extension.path === join(packageRoot, 'dist/extension/index.js')));
const {session} = await sdk.createAgentSession({cwd: project, agentDir, settingsManager, resourceLoader: loader,
  sessionManager: sdk.SessionManager.create(project, join(root, 'sessions'))});
const errors = [];
try {
  await session.bindExtensions({mode: 'print', uiContext: {notify: () => {}}, onError: error => errors.push(error)});
  await session.setModel(session.modelRuntime.getModel('offline-image-fixture', 'vision'));
  await session.prompt('Read the saved image for the offline transport check.');
  assert.deepEqual(errors, []);
  assert.ok(delivered, 'Pi did not deliver image content to the model context');
  assert.equal(delivered.mimeType, 'image/png');
  assert.equal(sha(Buffer.from(delivered.data, 'base64')), expected);
  const binding = JSON.parse(await fs.readFile(join(packageRoot, 'native-binding.json')));
  const report = {root, imageFile, imageSha256: expected, deliveredSha256: sha(Buffer.from(delivered.data, 'base64')),
    platform: process.platform, architecture: process.arch, node: process.version,
    pi: JSON.parse(await fs.readFile(join(peers, '@earendil-works/pi-coding-agent/package.json'))).version,
    releaseId: binding.releaseId, coreSha256: binding.coreSha256,
    packageInventorySha256: sha(await fs.readFile(join(packageRoot, 'package-inventory.json'))),
    result: 'byte-exact PNG delivered to offline image-capable Pi model context; no visual interpretation claimed'};
  await fs.writeFile(join(root, 'image-transport-report.json'), JSON.stringify(report, null, 2) + '\n');
  console.log('PASS byte-exact saved PNG delivered to offline image-capable Pi model context');
} finally {
  await session.extensionRunner.emit({type: 'session_shutdown', reason: 'quit'});
  session.dispose();
}
