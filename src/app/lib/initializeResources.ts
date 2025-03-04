import { initializeAccountsResource } from '@/entities/accounts';
import {
  initializeImportConfigs,
  initializeInstances,
  initializeRunningInstances,
} from '@/entities/instances';
import { initializePlugins } from '@/entities/plugins';

export const initializeResources = async () => {
  return Promise.all([
    initializeAccountsResource(),
    initializeImportConfigs(),
    initializeInstances(),
    initializeRunningInstances(),
    initializePlugins(),
  ]);
};
