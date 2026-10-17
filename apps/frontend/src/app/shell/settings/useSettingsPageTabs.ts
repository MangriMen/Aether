import { createMemo } from 'solid-js';

import { isDeveloperMode, SettingsTab } from '@/features/settings';

import { SETTINGS_TABS_DEFINITION } from './settingsTabs';

export const useSettingsPageTabs = () => {
  const availableTabs = createMemo(() => {
    if (isDeveloperMode()) {
      return SETTINGS_TABS_DEFINITION;
    }

    return SETTINGS_TABS_DEFINITION.filter(
      (tab) => tab.value !== SettingsTab.Experimental,
    );
  });

  return availableTabs;
};
