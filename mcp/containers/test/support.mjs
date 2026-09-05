import { Client } from '@modelcontextprotocol/client';
import { StdioClientTransport } from '@modelcontextprotocol/client/stdio';
import { fileURLToPath } from 'node:url';

export async function connect(state, volumes) {
  const client = new Client({ name: 'development-test', version: '1.0.0' });
  const transport = new StdioClientTransport({ command: process.execPath, args: [fileURLToPath(new URL('../src/index.mjs', import.meta.url))], env: { ...process.env, DEVELOPMENT_MCP_HOME: state, DEVELOPMENT_MCP_VOLUMES: volumes }, stderr: 'pipe' });
  let stderr = '';
  transport.stderr.on('data', data => { stderr += data; });
  await client.connect(transport);
  return { client, stderr: () => stderr };
}

export async function call(client, name, args = {}) {
  const result = await client.callTool({ name, arguments: args }, { timeout: 900_000 });
  if (result.isError) throw new Error(result.content[0].text);
  return result.structuredContent ?? JSON.parse(result.content[0].text);
}

