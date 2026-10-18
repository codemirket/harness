import {Server} from '@modelcontextprotocol/sdk/server/index.js';
import {StdioServerTransport} from '@modelcontextprotocol/sdk/server/stdio.js';
import {Client} from '@modelcontextprotocol/sdk/client/index.js';
import {StreamableHTTPClientTransport} from '@modelcontextprotocol/sdk/client/streamableHttp.js';
import {ListToolsRequestSchema, CallToolRequestSchema} from '@modelcontextprotocol/sdk/types.js';
import {compileTokens} from './tokens.mjs';
import Ajv from 'ajv';
import {withRequest, checkRequest, redactSecrets} from './request.mjs';
import {synchronizePenpotTokens} from './penpot-tokens.mjs';
import {readFileSync} from 'node:fs';
import {shutdownChildren, withRenderSlot} from './paths.mjs';

const toolPolicies = JSON.parse(readFileSync(new URL('./tool-policy.json', import.meta.url), 'utf8'));
function withPolicy(tool) {
  const policy = toolPolicies[tool.name];
  if (!policy) throw Error(`Design tool lacks a capability floor: ${tool.name}`);
  return {...tool, _meta: {...tool._meta, 'mirket/policy': policy}};
}

const file = {type: 'string', description: 'Workspace-relative file path'};
const output = {type: 'string', description: 'Workspace-relative output path; parent directory must exist'};
const overwrite = {type: 'boolean', default: false};
const common = {tokens: file, overwrite, expected_sha256: {type: 'string', pattern: '^[0-9a-f]{64}$'}, baseline: file};
function schema(required, properties) { return {type: 'object', additionalProperties: false, required, properties}; }
const definitions = [
  {name: 'tokens_compile', description: 'Compile one canonical DTCG token JSON into CSS, JavaScript, TypeScript, Typst and Penpot outputs with Style Dictionary.', inputSchema: schema([], {source: file})},
  {name: 'static_render', description: 'Render static Flexbox composition through Satori and resvg-js; optimize generated SVG with SVGO; return native raster pixels for inspection.', inputSchema: schema(['tree', 'fonts', 'width', 'height', 'output'], {...common, tree: {type: 'object'}, fonts: {type: 'array', items: schema(['path'], {path: file, name: {type: 'string'}, weight: {type: 'integer'}, style: {type: 'string'}})}, width: {type: 'integer'}, height: {type: 'integer'}, format: {type: 'string', enum: ['png', 'svg']}, output})},
  {name: 'browser_screenshot', description: 'Capture a local workspace or Penpot page using local Playwright/Chromium at verified 2× device scale, including DOM/layout/console evidence, bounded local UI actions and animation-state sampling; optional Odiff baseline comparison.', inputSchema: schema(['output'], {...common, source: file, url: {type: 'string'}, width: {type: 'integer'}, height: {type: 'integer'}, wait_for: {type: 'string'}, inspect_selectors: {type: 'array', maxItems: 16, items: {type: 'string', maxLength: 256}}, actions: {type:'array',maxItems:8,items:schema(['kind','selector'],{kind:{type:'string',enum:['click','hover','focus','press']},selector:{type:'string',minLength:1,maxLength:256},key:{type:'string',enum:['Enter','Space','Tab','Escape','ArrowLeft','ArrowRight','ArrowUp','ArrowDown']}})}, animation_time_ms:{type:'integer',minimum:0,maximum:60000,description:'Seek CSS/WAAPI animations; authored JS timelines may implement window.__mirketSeek(milliseconds).'}, reduced_motion:{type:'string',enum:['reduce','no-preference']}, full_page: {type: 'boolean'}, output})},
  {name: 'kroki_render', description: 'POST a diagram to Kroki and inspect its optimized SVG or PNG output.', inputSchema: schema(['source', 'diagram_type', 'output'], {...common, source: file, diagram_type: {type: 'string', enum: ['plantuml', 'c4plantuml', 'vegalite', 'excalidraw', 'd2', 'graphviz']}, format: {type: 'string', enum: ['svg', 'png']}, output})},
  {name: 'hyperframes_render', description: 'Run real HyperFrames CLI lint, snapshot and render in order, after canonical tokens and raster preprocessing.', inputSchema: schema(['project', 'output'], {...common, project: file, snapshot_times:{type:'array',minItems:1,maxItems:8,uniqueItems:true,items:{type:'number',minimum:0,maximum:3600}}, format: {type: 'string', enum: ['mp4', 'webm']}, output})},
  {name: 'typst_compile', description: 'Compile a paged document or deck with Typst, save PDF, render every page PNG and return native page PNGs for inspection.', inputSchema: schema(['source', 'output'], {...common, source: file, output})},
  {name: 'raster_transform', description: 'Preprocess a workspace raster through an isolated, bounded Sharp worker.', inputSchema: schema(['source', 'width', 'height', 'output'], {...common, source: file, width: {type: 'integer'}, height: {type: 'integer'}, mode: {type: 'string', enum: ['fit', 'fill', 'force']}, gravity: {type: 'string', enum: ['sm', 'ce', 'no', 'ea', 'so', 'we']}, quality: {type: 'integer', minimum: 1, maximum: 100}, enlarge: {type:'boolean',default:false}, format: {type: 'string', enum: ['png', 'jpeg', 'webp', 'avif']}, output})},
  {name: 'vector_render', description: 'Render authored vector artwork: preserve semantic SVG groups and IDs, optimize with SVGO and produce a bounded native resvg PNG preview for visual inspection. Graph layouts belong to Kroki.', inputSchema: schema(['source', 'output'], {...common, source: file, output, width: {type: 'integer', minimum:16, maximum:4096}, height:{type:'integer',minimum:16,maximum:4096}, fonts:{type:'array',maxItems:8,items:file}})},
  {name: 'svg_optimize', description: 'Optimize a generated or imported SVG with SVGO before publication.', inputSchema: schema(['source', 'output'], {...common, source: file, output})},
  {name: 'odiff_compare', description: 'Compare two saved PNG screenshots (at most 16 Mi pixels each) with the actual Odiff CLI; retain diff PNG when pixels differ.', inputSchema: schema(['baseline', 'candidate'], {baseline: file, candidate: file, output, overwrite, expected_sha256: common.expected_sha256})},
];
// Hosts keep a worker per connection. Discovery and cached tokens must not load
// every renderer into each idle process; import only when a tool needs it.
const operation = name => async args => (await import('./operations.mjs'))[name](args);
const handlers = {
  tokens_compile: args => compileTokens(args.source),
  static_render: operation('staticRender'),
  browser_screenshot: operation('browserScreenshot'),
  kroki_render: operation('krokiRender'),
  hyperframes_render: operation('hyperframesRender'),
  typst_compile: operation('typstCompile'),
  raster_transform: operation('rasterTransform'),
  svg_optimize: operation('svgOptimize'),
  vector_render: operation('vectorRender'),
  odiff_compare: operation('odiffCompare'),
};
const ajv = new Ajv({allErrors: true, strict: false});
const validators = Object.fromEntries(definitions.map(tool => [tool.name, ajv.compile(tool.inputSchema)]));
let serial = Promise.resolve();
async function exclusive(fn, signal) {
  let release;
  const prior = serial;
  serial = new Promise(resolve => { release = resolve; });
  try {
    await prior;
    if (signal?.aborted) throw signal.reason || Error('Design tool request was cancelled');
    return await withRequest(signal, fn);
  } finally { release(); }
}

let penpot;
let penpotOpening;
let penpotPromise;
let closing = false;
async function penpotClient() {
  if (closing) throw Error('Design tools session is closing');
  if (penpot) return penpot;
  if (!penpotPromise) {
    const client = new Client({name: 'mirket-design-penpot-forwarder', version: '1.0.0'});
    const transport = new StreamableHTTPClientTransport(new URL(process.env.MIRKET_PENPOT_MCP_URL || 'http://penpot-mcp:4401/mcp'));
    penpotOpening = client;
    penpotPromise = client.connect(transport, {timeout: 5000}).then(() => {
      if (closing) throw Error('Design tools session closed during Penpot connection');
      penpot = client;
      return client;
    }).catch(async error => {
      await client.close().catch(() => {});
      throw error;
    }).finally(() => { penpotOpening = undefined; penpotPromise = undefined; });
  }
  return penpotPromise;
}
const groups = new Set((process.env.MIRKET_DESIGN_GROUPS || 'core').split(','));
async function upstreamTools({optional = false} = {}) {
  if (!groups.has('penpot')) return [];
  try {
    const client = await penpotClient();
    const result = await client.listTools({}, {timeout: 5000});
    return result.tools || [];
  } catch (error) {
    if (!optional) throw Error(`Penpot dependency unavailable: ${redactSecrets(error.message)}`);
    process.stderr.write(`Penpot discovery unavailable: ${redactSecrets(error.message).slice(0, 500)}\n`);
    return [];
  }
}
const server = new Server({name: 'mirket-design-tools', version: '1.0.0'}, {capabilities: {tools: {listChanged: false}}});
function shutdown() {
  if (closing) return;
  closing = true;
  void (async () => {
    // SDK v1.32.1 does not close its stdio transport when stdin reaches EOF.
    // A bounded exit also releases a remote podman-exec process if an upstream
    // HTTP close stalls after its caller has gone away.
    const deadline = setTimeout(() => process.exit(1), 6000);
    let code = 0;
    try {
      const results = await Promise.allSettled([shutdownChildren(), penpot?.close(), penpotOpening?.close(), server.close(), serial]);
      for (const result of results) if (result.status === 'rejected') {
        code = 1;
        process.stderr.write(`Design shutdown failed: ${redactSecrets(String(result.reason)).slice(0, 500)}\n`);
      }
    } finally {
      clearTimeout(deadline);
      process.exit(code);
    }
  })();
}
for (const event of ['end', 'close']) process.stdin.once(event, () => { void shutdown(); });
for (const signal of ['SIGINT', 'SIGTERM']) process.once(signal, () => { void shutdown(); });
server.onclose = () => { void shutdown(); };
server.setRequestHandler(ListToolsRequestSchema, async () => {
  const remote = await upstreamTools({optional: true});
  const names = new Set(definitions.map(tool => tool.name));
  for (const tool of remote) if (names.has(tool.name)) throw Error(`Penpot tool name collides with a direct design tool: ${tool.name}`);
  return {tools: [...definitions, ...remote].map(tool => withPolicy(tool.name === 'kroki_render' ? {...tool, _meta: {'mirket/availability': {enabled: groups.has('diagrams'), group:'diagrams', start:'mirket package start design --group diagrams'}}} : tool))};
});
server.setRequestHandler(CallToolRequestSchema, async (request, extra) => {
  const signal = extra.signal;
  return exclusive(async () => {
  if (closing) throw Error('Design tools session is closing');
  const name = request.params.name;
  const args = request.params.arguments || {};
  try {
    checkRequest();
    if (name === 'kroki_render' && !groups.has('diagrams')) throw Error('Diagrams group is disabled; explicitly stop then start design with --group diagrams');
    if (handlers[name]) {
      if (!validators[name](args)) throw Error(`Invalid ${name} arguments: ${ajv.errorsText(validators[name].errors)}`);
      const result = name === 'tokens_compile' ? await handlers[name](args) : await withRenderSlot(() => handlers[name](args));
      return {content: [{type: 'text', text: JSON.stringify(result)}], structuredContent: result, isError: ['failed', 'changed_pixels'].includes(result.qa?.status)};
    }
    if (!groups.has('penpot')) throw Error('Penpot group is disabled; explicitly stop then start design with --group penpot');
    const available = await upstreamTools();
    if (!available.some(tool => tool.name === name)) throw Error(`Unknown design tool: ${name}`);
    if (['execute_code', 'export_shape', 'import_image'].includes(name)) {
      const compiled = await compileTokens(process.env.MIRKET_DESIGN_TOKENS || 'design/tokens.json');
      await synchronizePenpotTokens(await penpotClient(), compiled, args.sessionId, signal);
    }
    if (name === 'export_shape') return await withRenderSlot(async () => (await import('./penpot-export.mjs')).exportPenpotShape(await penpotClient(), args, signal));
    if (name === 'import_image') return await withRenderSlot(async () => (await import('./penpot-export.mjs')).importPenpotImage(await penpotClient(), args, signal));
    return await (await penpotClient()).callTool({name, arguments: args}, undefined, {signal});
  } catch (error) {
    return {isError: true, content: [{type: 'text', text: redactSecrets(error.stack || error.message || error)}]};
  }
  }, signal);
});
await server.connect(new StdioServerTransport());
