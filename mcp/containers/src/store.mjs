import { DatabaseSync } from 'node:sqlite';
import { mkdirSync, chmodSync } from 'node:fs';
import { join } from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';
import { randomUUID } from 'node:crypto';
import { seeds } from './catalog.mjs';

// One SQLite database holds durable state. A second is solely an OS-managed
// cross-process mutex, so state can be committed before external Docker actions.
export class Store {
  #tail = Promise.resolve();
  constructor(directory) {
    mkdirSync(directory, { recursive: true, mode: 0o700 });
    this.directory = directory;
    this.db = new DatabaseSync(join(directory, 'catalog.sqlite'));
    this.lock = new DatabaseSync(join(directory, 'operations.sqlite'));
    this.db.exec('PRAGMA busy_timeout=5000; CREATE TABLE IF NOT EXISTS records (collection TEXT NOT NULL, id TEXT NOT NULL, value TEXT NOT NULL, PRIMARY KEY(collection,id))');
    this.lock.exec('PRAGMA busy_timeout=0');
    this.db.exec('BEGIN IMMEDIATE');
    try {
      if (!this.get('meta', 'initialized')) {
        for (const seed of seeds) this.put('templates', seed.id, seed);
        this.put('meta', 'initialized', { version: 1, owner: randomUUID() });
      }
      this.db.exec('COMMIT');
    } catch (error) { this.db.exec('ROLLBACK'); throw error; }
    if (process.platform !== 'win32') {
      chmodSync(directory, 0o700);
      for (const file of ['catalog.sqlite', 'operations.sqlite']) chmodSync(join(directory, file), 0o600);
    }
  }
  get(collection, id) {
    const row = this.db.prepare('SELECT value FROM records WHERE collection=? AND id=?').get(collection, id);
    return row ? JSON.parse(row.value) : undefined;
  }
  all(collection) { return this.db.prepare('SELECT value FROM records WHERE collection=? ORDER BY id').all(collection).map(row => JSON.parse(row.value)); }
  put(collection, id, value) { this.db.prepare('INSERT INTO records VALUES (?,?,?) ON CONFLICT(collection,id) DO UPDATE SET value=excluded.value').run(collection, id, JSON.stringify(value)); }
  insertOnce(collection, id, value) { this.db.prepare('INSERT INTO records VALUES (?,?,?) ON CONFLICT(collection,id) DO NOTHING').run(collection, id, JSON.stringify(value)); }
  delete(collection, id) { this.db.prepare('DELETE FROM records WHERE collection=? AND id=?').run(collection, id); }
  exclusive(action) {
    const run = this.#tail.then(async () => {
      const deadline = Date.now() + 900_000;
      for (;;) {
        try { this.lock.exec('BEGIN IMMEDIATE'); break; }
        catch (error) {
          if (!/locked|busy/i.test(error.message) || Date.now() > deadline) throw new Error('Catalog is busy; retry after the current container operation finishes');
          await delay(100);
        }
      }
      try { return await action(); }
      finally { this.lock.exec('ROLLBACK'); }
    });
    this.#tail = run.catch(() => {});
    return run;
  }
  close() { this.db.close(); this.lock.close(); }
}
