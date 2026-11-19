import { describe, expect, it } from 'vitest';

import { resolvePluginProvider } from '../resolvePluginProvider';

describe('resolvePluginProvider', () => {
  it('keeps the picked provider when the backend offers it', () => {
    expect(resolvePluginProvider(['github', 'local'], 'local')).toBe('local');
  });

  it('falls back to the first provider when nothing is picked', () => {
    expect(resolvePluginProvider(['github', 'local'], null)).toBe('github');
  });

  it('drops a pick that the backend no longer offers', () => {
    expect(resolvePluginProvider(['github'], 'local')).toBe('github');
  });

  it('shows none when there are no providers', () => {
    expect(resolvePluginProvider([], 'github')).toBeNull();
  });

  it('gives the pick back when its provider returns to the list', () => {
    expect(resolvePluginProvider(['github'], 'local')).toBe('github');
    expect(resolvePluginProvider(['github', 'local'], 'local')).toBe('local');
  });
});
