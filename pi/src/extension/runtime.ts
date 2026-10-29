import { constants, lstatSync, openSync, closeSync, readFileSync, fstatSync, realpathSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { isAbsolute, join, resolve } from 'node:path';
export const HOST_VERSION = '1.4.0';
export interface Binding {
  schema: 1;
  runtimeSetupSha256?: string;
  packageVersion: string;
  hostVersion: string;
  hostInventorySha256: string;
  releaseId: string;
  coreSha256: string;
  nodeSha256: string;
  packageSha256: string;
}
export interface VerifiedRuntime { binding: Binding; home: string; command: string; cwd: string }
const hash = (bytes: Buffer) => createHash('sha256').update(bytes).digest('hex');
export function regularBytes(file: string, max: number): Buffer {
  const fd = openSync(file, constants.O_RDONLY | constants.O_NOFOLLOW);
  try {
    const stat = fstatSync(fd);
    if (!stat.isFile() || stat.nlink !== 1 || stat.size > max) throw new Error(`Invalid or aliased file: ${file}`);
    return readFileSync(fd);
  } finally { closeSync(fd); }
}
export function readBinding(packageRoot: string): Binding {
  const value = JSON.parse(regularBytes(join(packageRoot, 'native-binding.json'), 16384).toString()) as Binding;
  if (value.schema !== 1 || value.hostVersion !== HOST_VERSION || !/^sha256:[a-f0-9]{64}$/.test(value.releaseId)
      || ![value.coreSha256, value.nodeSha256, value.packageSha256, value.hostInventorySha256].every(x => /^[a-f0-9]{64}$/.test(x))) {
    throw new Error('Invalid native package binding');
  }
  const bytes = regularBytes(join(packageRoot, 'package.json'), 65536);
  const pkg = JSON.parse(bytes.toString());
  if (pkg.name !== '@codemirket/pi-harness' || pkg.version !== value.packageVersion || hash(bytes) !== value.packageSha256) {
    throw new Error('Package identity differs from native binding');
  }
  return value;
}
export async function verifyRuntime(packageRoot: string, agentDir: string, cwd: string, hostVersion: string): Promise<VerifiedRuntime> {
  if (hostVersion !== HOST_VERSION) throw new Error(`Unsupported pi host ${hostVersion}; expected ${HOST_VERSION}`);
  if (!isAbsolute(agentDir) || !isAbsolute(cwd)) throw new Error('Agent directory and project must be absolute');
  const binding = readBinding(packageRoot);
  const home = join(agentDir, 'mirket');
  const stat = lstatSync(home);
  if (!stat.isDirectory() || stat.isSymbolicLink() || (stat.mode & 0o077) !== 0) throw new Error('Runtime home must be a private real directory');
  if (realpathSync(home) !== resolve(home)) throw new Error('Runtime home has an aliased ancestor');
  const command = join(home, 'bin', 'mirket');
  if (hash(regularBytes(command, 256 * 1024 * 1024)) !== binding.coreSha256) throw new Error('Native launcher identity mismatch');
  // The pinned installed launcher binds the active release under its session
  // guard, then verifies the complete payload once before any MCP tool runs.
  // Running standalone doctor here would hash that same payload a second time.
  // This verifies the bootstrap tuple; actual readiness requires runtime_status.
  const node = join(home, 'releases', `r-${binding.releaseId.slice(7)}`, 'runtime/node/bin/node');
  if (hash(regularBytes(node, 256 * 1024 * 1024)) !== binding.nodeSha256) throw new Error('Private Node identity mismatch');
  return {binding, home, command, cwd: realpathSync(cwd)};
}
export function configuredOverride(agentDir: string, cwd: string): string | undefined {
  // Conservative refusal even for an untrusted project override. Never attest an
  // extension registration while a same-name file configuration may replace it.
  for (const file of [join(agentDir, 'mcp.json'), join(cwd, '.pi', 'mcp.json')]) {
    let bytes: Buffer;
    try { bytes = regularBytes(file, 1024 * 1024); }
    catch (error) { if ((error as NodeJS.ErrnoException).code === 'ENOENT') continue; throw error; }
    const config = JSON.parse(bytes.toString());
    if (Object.hasOwn(config.mcpServers ?? {}, 'mirket')) return file;
  }
  return undefined;
}
