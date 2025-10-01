import { useQuery } from '@tanstack/solid-query';

import { getImportConfigsRaw } from '../tauriApi';
import { CONFIG_QUERY_KEYS } from './config_query_keys';

export const useImportConfigs = () => {
  return useQuery(() => ({
    queryKey: CONFIG_QUERY_KEYS.IMPORT(),
    queryFn: getImportConfigsRaw,
  }));
};
