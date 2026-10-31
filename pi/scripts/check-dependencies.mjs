// Check required direct build/runtime inputs before importing the compiler.
// This is an offline version/lock guard, not a package-content integrity audit.
import fs from 'node:fs';
import path from 'node:path';
export function checkDependencies(root) {
  const read = file => JSON.parse(fs.readFileSync(file, 'utf8'));
  const manifest = read(path.join(root, 'package.json'));
  const lock = read(path.join(root, 'package-lock.json'));
  if (lock.lockfileVersion !== 3 || !lock.packages?.['']) throw new Error('Expected the locked npm v3 dependency tree');
  for (const group of ['dependencies', 'devDependencies']) {
    const requested = manifest[group] ?? {};
    const locked = lock.packages[''][group] ?? {};
    if (Object.keys(requested).length !== Object.keys(locked).length) throw new Error(`Package lock differs from ${group}`);
    for (const [name, version] of Object.entries(requested)) {
      const entry = lock.packages[`node_modules/${name}`];
      if (locked[name] !== version || entry?.version !== version) throw new Error(`Dependency is not exactly locked: ${name}`);
      const installed = read(path.join(root, 'node_modules', name, 'package.json'));
      if (installed.version !== version || installed.name !== name) throw new Error(`Installed dependency differs from lock: ${name}`);
    }
  }
}
