import {readJson} from './paths.mjs';

// TokenType names are from the official Penpot Plugin API TokenType declaration.
const types = Object.freeze({
  color: 'color', dimension: 'dimension', spacing: 'spacing', opacity: 'opacity', number: 'number', rotation: 'rotation', sizing: 'sizing',
  borderRadius: 'borderRadius', borderWidth: 'borderWidth', letterSpacing: 'letterSpacing', textCase: 'textCase', textDecoration: 'textDecoration',
  fontFamily: 'fontFamilies', fontFamilies: 'fontFamilies', fontSize: 'fontSizes', fontSizes: 'fontSizes', fontWeight: 'fontWeights', fontWeights: 'fontWeights',
});

function records(tree, prefix = '', output = []) {
  for (const [segment, entry] of Object.entries(tree)) {
    const name = prefix ? `${prefix}.${segment}` : segment;
    if (entry && typeof entry === 'object' && '$value' in entry) {
      const type = types[entry.$type];
      if (!type) throw Error(`Penpot cannot map DTCG token type ${entry.$type} at ${name}`);
      if (typeof entry.$value !== 'string' && typeof entry.$value !== 'number') throw Error(`Penpot token ${name} needs a scalar value for ${type}`);
      output.push({type, name, value: String(entry.$value)});
    } else if (entry && typeof entry === 'object') records(entry, name, output);
    else throw Error(`Invalid DTCG token node: ${name}`);
  }
  return output;
}
export {records as penpotTokenRecords};

function findAck(value, digest, depth = 0) {
  if (depth > 8 || value == null) return false;
  if (typeof value === 'string') {
    try { return findAck(JSON.parse(value), digest, depth + 1); } catch { return false; }
  }
  if (Array.isArray(value)) return value.some(item => findAck(item, digest, depth + 1));
  if (typeof value === 'object') return value.marker === 'mirket-token-sync' && value.source_sha256 === digest || Object.values(value).some(item => findAck(item, digest, depth + 1));
  return false;
}

export async function synchronizePenpotTokens(client, compilation, sessionId, signal) {
  const data = await readJson(compilation.outputs['penpot.tokens.json'], 512 * 1024);
  const specs = records(data);
  if (!specs.length || specs.length > 512) throw Error('Penpot canonical token set needs 1..512 supported tokens');
  const code = `
    const specs = ${JSON.stringify(specs)};
    const catalog = penpot.library?.local?.tokens;
    if (!catalog) throw new Error('Penpot token catalog is unavailable');
    const ownedName = 'Mirket Canonical';
    let set = catalog.sets.find(item => item.name === ownedName);
    if (!set) set = catalog.addSet({name: ownedName});
    const required = new Map(specs.map(item => [item.name, item]));
    for (const token of [...set.tokens]) {
      const next = required.get(token.name);
      if (!next || next.type !== token.type) token.remove();
    }
    await penpot.waitForLayoutUpdate();
    for (const item of specs) {
      const token = set.tokens.find(existing => existing.name === item.name && existing.type === item.type);
      if (token) { if (token.value !== item.value) token.value = item.value; }
      else set.addToken(item);
    }
    if (!set.active) set.toggleActive();
    await penpot.waitForLayoutUpdate();
    return {marker: 'mirket-token-sync', source_sha256: ${JSON.stringify(compilation.source_sha256)}, set_id: set.id, count: specs.length};
  `;
  const args = {code};
  if (sessionId) args.sessionId = sessionId;
  const result = await client.callTool({name: 'execute_code', arguments: args}, undefined, {signal});
  if (result.isError || !findAck(result, compilation.source_sha256)) throw Error(`Penpot canonical token sync failed: ${JSON.stringify(result.content || []).slice(0, 1000)}`);
  return {source_sha256: compilation.source_sha256, count: specs.length, set: 'Mirket Canonical'};
}
