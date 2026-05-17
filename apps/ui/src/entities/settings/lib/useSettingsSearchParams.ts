import type { Accessor } from 'solid-js';

import { useSearchParams } from '@solidjs/router';
import { createMemo } from 'solid-js';

import type { SettingsSearchParams } from '../model/settingsSearchParams';

import { DEFAULT_SETTINGS_TAB, type SettingsTab } from '../model';
import {
  decodeSettingsSearchParams,
  encodeSettingsSearchParams,
} from './settingsSearchParams';

export const useSettingsSearchParams = (): {
  params: Accessor<SettingsSearchParams>;
  setParams: (params: SettingsSearchParams) => void;
  open: (tab?: SettingsTab) => void;
  close: () => void;
} => {
  const [searchParams, setSearchParams] = useSearchParams();

  const params = createMemo(() => decodeSettingsSearchParams(searchParams));

  const isInitialSetup = createMemo(() => !params().tab);

  const setParams = (params: SettingsSearchParams) => {
    setSearchParams(encodeSettingsSearchParams(params), {
      replace: isInitialSetup(),
    });
  };

  const open = (tab?: SettingsTab) => {
    setParams({ modal: 'settings', tab: tab ?? DEFAULT_SETTINGS_TAB });
  };

  const close = () => {
    setSearchParams({ modal: undefined, tab: undefined });
  };

  return { params, setParams, open, close };
};
