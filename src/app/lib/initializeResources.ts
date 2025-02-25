import { initializeAccountsResource } from '@/entities/accounts';
import {
  initializeImportHandlers,
  initializeInstances,
  initializeRunningInstances,
} from '@/entities/instances';
import { initializePlugins } from '@/entities/plugins';

export const initializeResources = async () => {
  return Promise.all([
    initializeAccountsResource(),
    initializeImportHandlers(),
    initializeInstances(),
    initializeRunningInstances(),
    initializePlugins(),
  ]);
};
