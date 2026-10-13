import assert from 'node:assert/strict';
import test from 'node:test';
import { loadReferenceProfile } from './reference-profile.mjs';

test('loads a local profile before the caller initializes wasm', async () => {
  const env = { dev_query_creds: '1' };
  let completed = false;
  const fetcher = async (url, options) => {
    assert.equal(url, './survivors-reference.toml');
    assert.equal(options.cache, 'no-store');
    return { ok: true, text: async () => { completed = true; return 'schema = 1'; } };
  };
  assert.equal(await loadReferenceProfile(env, fetcher), true);
  assert.equal(completed, true);
  assert.equal(env.survivors_reference, 'schema = 1');
  assert.equal(env.dev_query_creds, '1');
});

test('an absent or failed optional profile leaves the default game environment', async () => {
  for (const fetcher of [
    async () => ({ ok: false }),
    async () => { throw new Error('offline'); },
    async () => ({ ok: true, text: async () => '' }),
    async () => ({ ok: true, text: async () => 'x'.repeat(65537) }),
  ]) {
    const env = { dev_query_creds: '1' };
    assert.equal(await loadReferenceProfile(env, fetcher), false);
    assert.deepEqual(env, { dev_query_creds: '1' });
  }
});

test('an explicitly supplied profile takes precedence', async () => {
  const env = { survivors_reference: 'custom' };
  assert.equal(await loadReferenceProfile(env, () => { throw new Error('must not fetch'); }), true);
  assert.equal(env.survivors_reference, 'custom');
});
