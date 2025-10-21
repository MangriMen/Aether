import { useQuery } from '@tanstack/solid-query';

import { getImportConfigsRaw } from '../tauriApi';
import { CONFIG_QUERY_KEYS } from './configQueryKeys';

export const useImportConfigs = () => {
  return useQuery(() => ({
    queryKey: CONFIG_QUERY_KEYS.IMPORT(),
    queryFn: getImportConfigsRaw,
  }));
};
