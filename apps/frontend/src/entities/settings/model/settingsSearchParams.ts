import type { GlobalSearchParams } from '@/shared/config';

import type { SettingsTab } from './settingsTabs';

export interface SettingsSearchParams extends GlobalSearchParams {
  modal?: 'settings';
  tab?: SettingsTab;
}
