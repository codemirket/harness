import {Server} from '@modelcontextprotocol/sdk/server/index.js';
import {StdioServerTransport} from '@modelcontextprotocol/sdk/server/stdio.js';
import {ListToolsRequestSchema, CallToolRequestSchema} from '@modelcontextprotocol/sdk/types.js';
import {compileTokens} from './tokens.mjs';
import Ajv from 'ajv';
import {withRequest, checkRequest, redactSecrets} from './request.mjs';
import {readFileSync, readdirSync} from 'node:fs';
import {createHash} from 'node:crypto';

// Captured once by the connected process, including every owned runtime module.
const runtimeFiles = readdirSync(new URL('./', import.meta.url)).filter(name => name.endsWith('.mjs') || ['package-lock.json', 'tool-policy.json', 'tool-budgets.json', 'openpencil-tools.json'].includes(name)).sort();
const runtimeHashes = Object.fromEntries(runtimeFiles.map(name => [name, createHash('sha256').update(readFileSync(new URL(name, import.meta.url))).digest('hex')]));
const runtimeIdentity = Object.freeze({sha256:createHash('sha256').update(JSON.stringify(runtimeHashes)).digest('hex'), files:runtimeHashes, node:process.version, workspace, scope:workspacePrefix ? 'project' : 'host'});
import {shutdownChildren, withRenderSlot, assertOutputReady, workspace, workspacePrefix} from './paths.mjs';

const toolPolicies = JSON.parse(readFileSync(new URL('./tool-policy.json', import.meta.url), 'utf8'));
function withPolicy(tool) {
  const policy = toolPolicies[tool.name];
  if (!policy) throw Error(`Design tool lacks a capability floor: ${tool.name}`);
  return {...tool, _meta: {...tool._meta, 'mirket/policy': policy}};
}

const file = {type: 'string', description: 'Workspace-relative file path'};
const output = {type: 'string', description: 'Workspace-relative output path; parent directory must exist'};
const svgOutput = {...output, pattern: '\\.svg$', description: 'Workspace-relative editable SVG output path ending in .svg; parent directory must exist. vector_render returns its separate PNG preview path in the receipt.'};
const overwrite = {type: 'boolean', default: false, description:'Replace an existing output only with its current expected_sha256. Omit for a new output.'};
const common = {tokens: file, overwrite, expected_sha256: {type: 'string', pattern: '^[0-9a-f]{64}$'}, baseline: file};
function schema(required, properties) {
  return {type: 'object', additionalProperties: false, required, properties,
    ...(properties.overwrite ? {allOf:[{if:{required:['overwrite'],properties:{overwrite:{const:true}}},then:{required:['expected_sha256']}}]} : {})};
}
const definitions = [
  {name:'openpencil_inspect',description:'Inspect an editable OpenPencil .fig/.pen file with an upstream read tool. Omit source to list supported tools; provide tool without source for its exact upstream argument schema.',inputSchema:schema([], {source:file,tool:{type:'string'},arguments:{type:'object'}})},
  {name:'openpencil_document',description:'Create an editable OpenPencil .fig document, or import a .fig/.pen document; compile canonical tokens and import them as variables/styles.',inputSchema:schema(['output'],{...common,source:file,output})},
  {name:'openpencil_edit',description:'Apply one validated upstream OpenPencil write tool to a file and atomically save an editable .fig revision. Discover tool argument schemas with openpencil_inspect. Canonical tokens are compiled before editing. Use IDs from the saved document; serialization can remap them.',inputSchema:schema(['source','tool','arguments','output'],{...common,source:file,tool:{type:'string'},arguments:{type:'object'},output})},
  {name:'openpencil_export',description:'Export the saved OpenPencil document or selected node to PNG at 2x or SVG. Inspect PNG pixels independently; export is not design acceptance.',inputSchema:schema(['source','output'],{...common,source:file,format:{type:'string',enum:['png','svg']},node:{type:'string'},page:{type:'string'},output})},
  {name: 'tokens_compile', description: 'Compile one canonical DTCG token JSON into CSS, JavaScript, TypeScript, Typst and DTCG outputs with Style Dictionary.', inputSchema: schema([], {source: file})},
  {name: 'static_render', description: 'Render static Flexbox composition through Satori and resvg-js; optimize generated SVG with SVGO; return native raster pixels for inspection.', inputSchema: schema(['tree', 'fonts', 'width', 'height', 'output'], {...common, tree: {type: 'object'}, fonts: {type: 'array', items: schema(['path'], {path: file, name: {type: 'string'}, weight: {type: 'integer'}, style: {type: 'string'}})}, width: {type: 'integer'}, height: {type: 'integer'}, format: {type: 'string', enum: ['png', 'svg']}, output})},
  {name: 'browser_screenshot', description: 'Capture a local workspace page using local Playwright/Chromium at verified 2× device scale, including DOM/layout/console evidence, bounded local UI actions and animation-state sampling; optional Odiff baseline comparison.', inputSchema: schema(['output'], {...common, source: file, url: {type: 'string'}, width: {type: 'integer'}, height: {type: 'integer'}, wait_for: {type: 'string'}, inspect_selectors: {type: 'array', maxItems: 16, items: {type: 'string', maxLength: 256}}, actions: {type:'array',maxItems:8,items:schema(['kind','selector'],{kind:{type:'string',enum:['click','hover','focus','press']},selector:{type:'string',minLength:1,maxLength:256},key:{type:'string',enum:['Enter','Space','Tab','Escape','ArrowLeft','ArrowRight','ArrowUp','ArrowDown']}})}, animation_time_ms:{type:'integer',minimum:0,maximum:60000,description:'Seek CSS/WAAPI animations; authored JS timelines may implement window.__mirketSeek(milliseconds).'}, reduced_motion:{type:'string',enum:['reduce','no-preference']}, full_page: {type: 'boolean'}, output})},
  {name: 'kroki_render', description: 'POST a diagram to Kroki and inspect its optimized SVG or PNG output.', inputSchema: schema(['source', 'diagram_type', 'output'], {...common, source: file, diagram_type: {type: 'string', enum: ['plantuml', 'c4plantuml', 'vegalite', 'excalidraw', 'd2', 'graphviz']}, format: {type: 'string', enum: ['svg', 'png']}, output})},
  {name: 'hyperframes_render', description: 'Run real HyperFrames CLI lint, snapshot and render in order, after canonical tokens and raster preprocessing.', inputSchema: schema(['project', 'output'], {...common, project: file, snapshot_times:{type:'array',minItems:1,maxItems:8,uniqueItems:true,items:{type:'number',minimum:0,maximum:3600}}, format: {type: 'string', enum: ['mp4', 'webm']}, output})},
  {name: 'typst_compile', description: 'Compile a paged document or deck with Typst, save PDF, render every page PNG and return native page PNGs for inspection.', inputSchema: schema(['source', 'output'], {...common, source: file, output})},
  {name: 'raster_transform', description: 'Preprocess a workspace raster through an isolated, bounded Sharp worker.', inputSchema: schema(['source', 'width', 'height', 'output'], {...common, source: file, width: {type: 'integer'}, height: {type: 'integer'}, mode: {type: 'string', enum: ['fit', 'fill', 'force']}, gravity: {type: 'string', enum: ['sm', 'ce', 'no', 'ea', 'so', 'we']}, quality: {type: 'integer', minimum: 1, maximum: 100}, enlarge: {type:'boolean',default:false}, format: {type: 'string', enum: ['png', 'jpeg', 'webp', 'avif']}, output})},
  {name: 'vector_render', description: 'Render authored vector artwork: preserve semantic SVG groups and IDs, optimize with SVGO and produce a bounded native resvg PNG preview for visual inspection. Previews use pinned DejaVu font fallbacks; supply local fonts for exact typography. Graph layouts belong to Kroki.', inputSchema: schema(['source', 'output'], {...common, source: file, output: svgOutput, width: {type: 'integer', minimum:16, maximum:4096}, height:{type:'integer',minimum:16,maximum:4096}, fonts:{type:'array',maxItems:8,items:file}})},
  {name: 'svg_optimize', description: 'Optimize a generated or imported SVG with SVGO before publication.', inputSchema: schema(['source', 'output'], {...common, source: file, output: svgOutput})},
  {name: 'odiff_compare', description: 'Compare two saved PNG screenshots (at most 16 Mi pixels each) with the actual Odiff CLI; retain diff PNG when pixels differ.', inputSchema: schema(['baseline', 'candidate'], {baseline: file, candidate: file, output, overwrite, expected_sha256: common.expected_sha256})},
];
// Hosts keep a worker per connection. Discovery and cached tokens must not load
// every renderer into each idle process; import only when a tool needs it.
const operation = name => async args => (await import('./operations.mjs'))[name](args);
const pencilOperation = name => async args => (await import('./openpencil.mjs'))[name](args);
const handlers = {
  openpencil_inspect:pencilOperation('openpencilInspect'),
  openpencil_document:pencilOperation('openpencilDocument'),
  openpencil_edit:pencilOperation('openpencilEdit'),
  openpencil_export:pencilOperation('openpencilExport'),
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

let closing = false;
const groups = new Set((process.env.MIRKET_DESIGN_GROUPS || 'core').split(','));
const server = new Server({name: 'mirket-design-tools', version: `1.0.0+${runtimeIdentity.sha256.slice(0,12)}`}, {capabilities: {tools: {listChanged: false}}});
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
      const results = await Promise.allSettled([shutdownChildren(), server.close(), serial]);
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
  return {tools: definitions.map(tool => withPolicy(tool.name === 'kroki_render' ? {...tool, _meta: {'mirket/availability': {enabled: groups.has('diagrams'), group:'diagrams', start:'mirket package start design --group diagrams'}}} : tool))};
});
server.setRequestHandler(CallToolRequestSchema, async (request, extra) => {
  const signal = extra.signal;
  return exclusive(async () => {
  if (closing) throw Error('Design tools session is closing');
  const name = request.params.name;
  const args = request.params.arguments || {};
  try {
    checkRequest();
    if (handlers[name]) {
      // Only this owned boundary can attest that the operation did not start.
      // Never attach this receipt to a handler or upstream operation error.
      const rejected = (code, message) => ({isError:true, content:[{type:'text', text:message}], structuredContent:{execution:{tool:name, state:'not_started', code}, message}});
      if (name === 'kroki_render' && !groups.has('diagrams')) return rejected('disabled_group', 'Diagrams group is disabled; explicitly stop then start design with --group diagrams');
      if (!validators[name](args)) return rejected('invalid_arguments', `Invalid ${name} arguments: ${ajv.errorsText(validators[name].errors)}`);
      if (typeof args.output === 'string') {
        try { await assertOutputReady(args); }
        catch (error) { return rejected('output_preflight', redactSecrets(String(error.message || error))); }
      }
      let started = false;
      try {
        const invoke = () => { started = true; return handlers[name](args); };
        const result = name === 'tokens_compile' ? await invoke() : await withRenderSlot(invoke);
        // Some desktop hosts omit MCP _meta from model-visible responses.
        // Keep the exact loaded identity in ordinary content as well.
        const visible = {...result, runtime: runtimeIdentity};
        return {_meta:{'mirket/runtime':runtimeIdentity}, content: [{type: 'text', text: JSON.stringify(visible)}], structuredContent: visible, isError: ['failed', 'changed_pixels'].includes(result.qa?.status)};
      } catch (error) {
        if (!started) return rejected('render_admission', redactSecrets(String(error.message || error)));
        throw error;
      }
    }
    throw Error(`Unknown design tool: ${name}`);
  } catch (error) {
    return {isError: true, content: [{type: 'text', text: redactSecrets(error.stack || error.message || error)}]};
  }
  }, signal);
});
await server.connect(new StdioServerTransport());
