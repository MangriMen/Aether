import { createMemo } from 'solid-js';

import { SettingsTab } from '@/entities/settings';
import { isDeveloperMode } from '@/shared/model';

import { SETTINGS_TABS_DEFINITION } from '../model';

export const useSettingsPageTabs = () => {
  const availableTabs = createMemo(() => {
    if (!isDeveloperMode()) {
      return SETTINGS_TABS_DEFINITION;
    }

    return SETTINGS_TABS_DEFINITION.filter(
      (tab) => tab.value !== SettingsTab.Experimental,
    );
  });

  return availableTabs;
};
