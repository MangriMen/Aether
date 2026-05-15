import type { SearchParams } from '@solidjs/router';

import { parseSearchParamToString } from '@/shared/lib';

import type { SettingsSearchParams } from '../model/settingsSearchParams';

import { DEFAULT_TAB, isSettingsTab, type SettingsTab } from '../model';

export const decodeSettingsSearchParams = ({
  tab,
}: Partial<SearchParams>): SettingsSearchParams => {
  const parsedTab = parseSearchParamToString(tab);

  return {
    tab: resolveSettingsTab(parsedTab),
  };
};

export const encodeSettingsSearchParams = ({
  tab,
}: SettingsSearchParams): SearchParams => {
  return {
    tab,
  };
};

const resolveSettingsTab = (
  tab: string | undefined,
): SettingsTab | undefined => {
  if (tab === undefined) {
    return;
  }

  return isSettingsTab(tab) ? tab : DEFAULT_TAB;
};
