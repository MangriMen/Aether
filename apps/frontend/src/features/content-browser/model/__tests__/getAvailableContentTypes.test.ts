import { describe, expect, it } from 'vitest';

import { ATOMIC_CONTENT_TYPES, ContentType } from '@/features/content';

import { getAvailableContentTypes } from '../getAvailableContentTypes';

describe('getAvailableContentTypes', () => {
  it('allows every type outside an instance', () => {
    expect(getAvailableContentTypes(undefined, false)).toBeUndefined();
  });

  it('offers only resource and data packs for a vanilla instance', () => {
    expect(getAvailableContentTypes('vanilla', true)).toEqual([
      ContentType.ResourcePack,
      ContentType.DataPack,
    ]);
  });

  it('offers nothing for an instance without a loader', () => {
    expect(getAvailableContentTypes(undefined, true)).toEqual([]);
  });

  it('offers atomic types without modpacks for a modded instance', () => {
    expect(getAvailableContentTypes('fabric', true)).toEqual(
      ATOMIC_CONTENT_TYPES,
    );
  });
});
