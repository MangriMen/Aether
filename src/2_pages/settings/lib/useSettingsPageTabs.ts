import type { Accessor } from 'solid-js';

import { createMemo } from 'solid-js';

import { isDeveloperMode } from '@/shared/model';

import {
  SETTINGS_TABS_CONTENT,
  SETTINGS_TABS_TRIGGER,
  SettingsTab,
} from '../model/settingsTabs';

export const useSettingsPageTabs = (): [
  Accessor<typeof SETTINGS_TABS_TRIGGER>,
  Accessor<typeof SETTINGS_TABS_CONTENT>,
] => {
  const tabsTriggers = createMemo(() => {
    if (!isDeveloperMode()) {
      return SETTINGS_TABS_TRIGGER.filter(
        (trigger) => trigger.value !== SettingsTab.Experimental,
      );
    }

    return SETTINGS_TABS_TRIGGER;
  });

  const tabsContents = createMemo(() => {
    if (!isDeveloperMode()) {
      return SETTINGS_TABS_CONTENT.filter(
        (trigger) => trigger.value !== SettingsTab.Experimental,
      );
    }

    return SETTINGS_TABS_CONTENT;
  });

  return [tabsTriggers, tabsContents];
};
