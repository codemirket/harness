import { mkdir, lstat, chmod, open } from 'node:fs/promises';
import { constants } from 'node:fs';

export async function privateDirectory(path) {
  try {
    const info = await lstat(path);
    if (info.isSymbolicLink()) throw new Error('Managed configuration directory must not be a symbolic link.');
    if (!info.isDirectory()) throw new Error('Managed configuration path must be a directory.');
  } catch (error) { if (error.code !== 'ENOENT') throw error; }
  await mkdir(path, { recursive: true, mode: 0o700 });
  if (process.platform !== 'win32') await chmod(path, 0o700);
}

export async function privateFile(path, content, { exclusive = false } = {}) {
  // Configuration is written only while no owned container is running. Check
  // reparse points explicitly on Windows, and additionally use O_NOFOLLOW on Unix.
  let present = false;
  try {
    const info = await lstat(path);
    present = true;
    if (info.isSymbolicLink() || !info.isFile() || info.nlink !== 1) throw new Error('Managed credential file must be a regular file without symbolic or hard links.');
  } catch (error) { if (error.code !== 'ENOENT') throw error; }
  const handle = await open(path, constants.O_WRONLY | constants.O_CREAT | (exclusive && !present ? constants.O_EXCL : 0) | (constants.O_NOFOLLOW ?? 0), 0o600);
  try {
    const info = await handle.stat();
    if (!info.isFile() || info.nlink !== 1) throw new Error('Managed credential file must not have hard links.');
    if (process.platform !== 'win32') await handle.chmod(0o600);
    if (exclusive && present) return;
    await handle.truncate(0);
    await handle.writeFile(content);
    await handle.sync();
  } finally { await handle.close(); }
}
