import { initializeAccountsResource } from '@/entities/accounts';
import {} from '@/entities/instances';
import { initializePlugins } from '@/entities/plugins';
import { initializeSettings } from '@/entities/settings';

export const initializeResources = async () => {
  return Promise.all([
    initializeAccountsResource(),
    initializePlugins(),
    initializeSettings(),
  ]);
};
