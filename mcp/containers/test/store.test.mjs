import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { Store } from '../src/store.mjs';

async function fixture(t) {
  const dir = await mkdtemp(join(tmpdir(), 'development-store-'));
  const store = new Store(dir);
  t.after(async () => { store.close(); await rm(dir, { recursive: true, force: true }); });
  return { dir, store };
}

test('catalog is seeded once and deletions survive reopening', async t => {
  const { store, dir } = await fixture(t);
  assert.deepEqual(store.all('templates').map(x => x.id).sort(), ['minio', 'postgres', 'redis']);
  await store.exclusive(async () => store.delete('templates', 'redis'));
  const other = new Store(dir);
  t.after(() => other.close());
  assert.equal(other.get('templates', 'redis'), undefined);
});

test('independent clients serialize mutations and persist every write', async t => {
  const { store, dir } = await fixture(t);
  const other = new Store(dir);
  t.after(() => other.close());
  const update = s => s.exclusive(async () => {
    const before = s.get('meta', 'counter')?.value ?? 0;
    await new Promise(resolve => setTimeout(resolve, 10));
    s.put('meta', 'counter', { value: before + 1 });
  });
  await Promise.all(Array.from({ length: 6 }, (_, i) => update(i % 2 ? store : other)));
  assert.equal(store.get('meta', 'counter').value, 6);
});

test('failed operations release locks and already saved credentials survive', async t => {
  const { store } = await fixture(t);
  await assert.rejects(store.exclusive(async () => {
    store.put('services', 'postgres', { secret: 'fixture-secret' });
    throw new Error('Docker unavailable');
  }), /Docker unavailable/);
  await store.exclusive(async () => assert.equal(store.get('services', 'postgres').secret, 'fixture-secret'));
});

test('Docker endpoint binding is insert-only and cannot overwrite another client', async t => {
  const { store, dir } = await fixture(t);
  const other = new Store(dir);
  try {
    store.insertOnce('meta', 'docker', { endpoint: 'unix:///a' });
    other.insertOnce('meta', 'docker', { endpoint: 'unix:///b' });
    assert.equal(store.get('meta', 'docker').endpoint, 'unix:///a');
  } finally { other.close(); }
});
