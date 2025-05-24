import {} from '@/entities/instances';
import { initializePlugins } from '@/entities/plugins';

export const initializeResources = async () => {
  return Promise.all([initializePlugins()]);
};
