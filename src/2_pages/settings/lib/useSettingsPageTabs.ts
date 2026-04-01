import { createMemo } from 'solid-js';

import { isDeveloperMode } from '@/shared/model';

import { SETTINGS_TABS_DEFINITION, SettingsTab } from '../model/settingsTabs';

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
