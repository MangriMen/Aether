import * as v from 'valibot';
import { describe, expect, it } from 'vitest';

import { CreateCustomInstanceSchema } from '../validation';

const base = { name: 'Pack', gameVersion: '1.21.1' };

describe('CreateCustomInstanceSchema', () => {
  it('ignores a leftover "other" loader version type for vanilla', () => {
    const result = v.safeParse(CreateCustomInstanceSchema, {
      ...base,
      loader: 'vanilla',
      loaderVersionType: 'other',
    });

    expect(result.success).toBe(true);
  });

  it('requires a loader version when a modded loader asks for a specific one', () => {
    const result = v.safeParse(CreateCustomInstanceSchema, {
      ...base,
      loader: 'fabric',
      loaderVersionType: 'other',
    });

    expect(result.success).toBe(false);
  });

  it('accepts a specific loader version', () => {
    const result = v.safeParse(CreateCustomInstanceSchema, {
      ...base,
      loader: 'fabric',
      loaderVersionType: 'other',
      loaderVersion: '0.16.0',
    });

    expect(result.success).toBe(true);
  });
});
