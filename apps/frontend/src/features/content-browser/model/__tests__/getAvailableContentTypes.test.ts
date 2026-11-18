import { describe, expect, it } from 'vitest';

import { ContentType } from '@/features/content';

import { getAvailableContentTypes } from '../getAvailableContentTypes';

describe('getAvailableContentTypes', () => {
  it('should_allow_every_type_when_browsing_outside_an_instance', () => {
    expect(getAvailableContentTypes(undefined, false)).toBeUndefined();
  });

  it('should_offer_only_resource_and_data_packs_when_the_instance_is_vanilla', () => {
    expect(getAvailableContentTypes('vanilla', true)).toEqual([
      ContentType.ResourcePack,
      ContentType.DataPack,
    ]);
  });

  it('should_offer_nothing_when_the_instance_has_no_loader', () => {
    expect(getAvailableContentTypes(undefined, true)).toEqual([]);
  });
});
