import type { SearchParams } from '@solidjs/router';

import { parseSearchParamToString } from '@/shared/lib';

import type { SettingsSearchParams } from '../model';

import { resolveSettingsTab } from '../model';

export const decodeSettingsSearchParams = ({
  tab,
}: Partial<SearchParams>): SettingsSearchParams => {
  const parsedTab = parseSearchParamToString(tab);

  return {
    tab: resolveSettingsTab(parsedTab),
  };
};

export const encodeSettingsSearchParams = ({
  modal,
  tab,
}: SettingsSearchParams): SearchParams => {
  const params: SearchParams = {};

  if (modal !== undefined) {
    params.modal = modal;
  }

  if (tab !== undefined) {
    params.tab = tab;
  }

  return params;
};
