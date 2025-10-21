import type { QueryClient } from '@tanstack/solid-query';

import type { Instance } from '../../model';

import { CONTENT_QUERY_KEYS } from './contentQueryKeys';
import { INSTANCE_QUERY_KEYS } from './instanceQueryKeys';

export const invalidateInstanceData = (
  queryClient: QueryClient,
  id: Instance['id'],
) => {
  queryClient.invalidateQueries({
    queryKey: INSTANCE_QUERY_KEYS.LIST(),
  });
  queryClient.invalidateQueries({
    queryKey: INSTANCE_QUERY_KEYS.GET(id),
  });
  queryClient.invalidateQueries({
    queryKey: INSTANCE_QUERY_KEYS.DIR(id),
  });
  invalidateInstanceContent(queryClient, id);
};

export const invalidateInstanceContent = (
  queryClient: QueryClient,
  id: Instance['id'],
) => {
  queryClient.invalidateQueries({
    queryKey: CONTENT_QUERY_KEYS.BY_INSTANCE(id),
  });
};
