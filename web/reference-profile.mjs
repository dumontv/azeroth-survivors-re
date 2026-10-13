// A locally supplied class profile is available before the wasm snapshots its page environment.
export async function loadReferenceProfile(env, fetcher = globalThis.fetch) {
  if (typeof env.survivors_reference === 'string') return true;
  try {
    const response = await fetcher('./survivors-reference.toml', { cache: 'no-store' });
    if (!response.ok) return false;
    const text = await response.text();
    if (!text || text.length > 65536) return false;
    env.survivors_reference = text;
    return true;
  } catch {
    // The reference is optional; a plain fork build still boots without it.
    return false;
  }
}
