import type { Accessor } from 'solid-js';

import { useSearchParams } from '@solidjs/router';
import { createMemo } from 'solid-js';

import type { SettingsSearchParams } from '../model/settingsSearchParams';

import {
  decodeSettingsSearchParams,
  encodeSettingsSearchParams,
} from './settingsSearchParams';

export const useSettingsSearchParams = (): [
  Accessor<SettingsSearchParams>,
  (value: SettingsSearchParams) => void,
] => {
  const [searchParams, setSearchParams] = useSearchParams();

  const currentParams = createMemo(() =>
    decodeSettingsSearchParams(searchParams),
  );

  const setParams = (params: SettingsSearchParams) => {
    setSearchParams(encodeSettingsSearchParams(params));
  };

  return [currentParams, setParams];
};
