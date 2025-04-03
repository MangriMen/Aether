import { initializeAccountsResource } from '@/entities/accounts';
import {
  initializeImportConfigs,
  initializeInstances,
  initializeRunningInstances,
} from '@/entities/instances';
import { initializePlugins } from '@/entities/plugins';
import { initializeSettings } from '@/entities/settings';

export const initializeResources = async () => {
  return Promise.all([
    initializeAccountsResource(),
    initializeImportConfigs(),
    initializeInstances(),
    initializeRunningInstances(),
    initializePlugins(),
    initializeSettings(),
  ]);
};
