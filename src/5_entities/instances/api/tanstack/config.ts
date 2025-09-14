import { useQuery } from '@tanstack/solid-query';

import { getImportConfigsRaw } from '../rawApi';
import { CONFIG_QUERY_KEYS } from './config_query_keys';

export const useImportConfigs = () => {
  return useQuery(() => ({
    queryFn: getImportConfigsRaw,
    queryKey: CONFIG_QUERY_KEYS.IMPORT(),
  }));
};
