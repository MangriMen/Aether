import { createMemo, type Accessor } from 'solid-js';

import type { Instance } from '@/entities/instances';

import type { ContentPageSearchParams } from '../model/contentPageSearchParams';

import { getAvailableContentTypes } from './getAvailableContentTypes';
import { getFiltersFromInstance } from './getFiltersFromInstance';
import { getFiltersFromSearchParams } from './getFiltersFromSearchParams';
import { mergeContentFilters } from './mergeContentFilters';

export const useContentPageState = (
  instance: Accessor<Instance | undefined>,
  params: Accessor<ContentPageSearchParams>,
) => {
  const state = createMemo(() => {
    const instanceData = instance();
    const paramsData = params();

    const isInstanceContentPage = Boolean(paramsData.instanceId);

    const availableContentTypes = getAvailableContentTypes(
      instanceData?.loader,
      isInstanceContentPage,
    );

    const { filters, filtersLock } = mergeContentFilters(
      getFiltersFromInstance(instanceData),
      getFiltersFromSearchParams(paramsData),
    );

    return {
      isInstanceContentPage,
      availableContentTypes,
      filters,
      filtersLock,
    };
  });

  return state;
};
