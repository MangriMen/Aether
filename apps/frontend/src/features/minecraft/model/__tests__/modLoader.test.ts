import { describe, expect, it } from 'vitest';

import { isModLoader, MOD_LOADERS } from '../modLoader';

describe('isModLoader', () => {
  it.each(MOD_LOADERS)('accepts the canonical name %s', (loader) => {
    expect(isModLoader(loader)).toBe(true);
  });

  it.each(['neo', 'Fabric', 'bukkit', ''])(
    'rejects the string %j that is not a canonical name',
    (value) => {
      expect(isModLoader(value)).toBe(false);
    },
  );

  it.each([undefined, null, 42, {}])('rejects the non-string %j', (value) => {
    expect(isModLoader(value)).toBe(false);
  });
});
