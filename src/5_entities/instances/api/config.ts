import { useQuery } from '@tanstack/solid-query';
import { getImportConfigsRaw } from './rawApi';

export const configKeys = {
  all: ['configs'] as const,
  importConfigs: () => [...configKeys.all, 'import'] as const,
};

export const useImportConfigs = () => {
  return useQuery(() => ({
    queryKey: configKeys.importConfigs(),
    queryFn: getImportConfigsRaw,
  }));
};
