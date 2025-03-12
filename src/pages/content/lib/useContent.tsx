import type { ContentResponse, ContentRequest } from '@/entities/instances';
import { getContentByProvider } from '@/entities/instances';

import type { ApiResult } from '@/shared/lib';
import { toResponseResult } from '@/shared/lib';
import { debounce } from '@solid-primitives/scheduled';
import type { Accessor, InitializedResource } from 'solid-js';
import { createEffect, createResource, createSignal } from 'solid-js';

export const useContent = (
  payload: Accessor<ContentRequest | null>,
): [
  InitializedResource<ApiResult<ContentResponse>>,
  { isLoading: Accessor<boolean> },
] => {
  const [contentRequest] = createResource(
    payload,
    async (payload) => toResponseResult(getContentByProvider, payload),
    { initialValue: { data: undefined, error: undefined } },
  );

  const [debouncedLoading, setDebouncedLoading] = createSignal(false);
  const setDebouncedLoadingDebounced = debounce(setDebouncedLoading, 300);

  createEffect(() => {
    setDebouncedLoadingDebounced(contentRequest.loading);
  });

  return [contentRequest, { isLoading: debouncedLoading }];
};
