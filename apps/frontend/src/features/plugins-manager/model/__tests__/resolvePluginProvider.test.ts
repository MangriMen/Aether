import { describe, expect, it } from 'vitest';

import { resolvePluginProvider } from '../resolvePluginProvider';

describe('resolvePluginProvider', () => {
  it('keeps the picked provider when the backend offers it', () => {
    expect(resolvePluginProvider(['git_hub', 'local'], 'local')).toBe('local');
  });

  it('falls back to the first provider when nothing is picked', () => {
    expect(resolvePluginProvider(['git_hub', 'local'], null)).toBe('git_hub');
  });

  it('drops a pick that the backend no longer offers', () => {
    expect(resolvePluginProvider(['git_hub'], 'local')).toBe('git_hub');
  });

  it('shows none when there are no providers', () => {
    expect(resolvePluginProvider([], 'git_hub')).toBeNull();
  });

  it('gives the pick back when its provider returns to the list', () => {
    expect(resolvePluginProvider(['git_hub'], 'local')).toBe('git_hub');
    expect(resolvePluginProvider(['git_hub', 'local'], 'local')).toBe('local');
  });
});
