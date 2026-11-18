import type { ColorMode } from '@kobalte/core';

import { useColorMode } from '@kobalte/core';
import { createEffect, on } from 'solid-js';

import {
  useAppSettings,
  useEditAppSettings,
  type WindowEffect,
} from '@/features/settings';
import { logError } from '@/shared/log';
import { debounce } from '@/shared/utils';

import { isSystemTheme } from '../model/theme';
import { useThemeContext } from '../model/themeContext';

export const useMicaUpdateListener = () => {
  const { colorMode } = useColorMode();

  const [theme] = useThemeContext();

  const appSettings = useAppSettings();
  const updateAppSettings = useEditAppSettings({ silent: true });

  const updateMicaDebounced = debounce(
    (rawTheme: string, colorMode: ColorMode) => {
      const window_effect = appSettings.data?.windowEffect;

      if (
        window_effect !== 'mica' &&
        window_effect !== 'mica_dark' &&
        window_effect !== 'mica_light'
      ) {
        return;
      }

      const next: WindowEffect = isSystemTheme(rawTheme)
        ? 'mica'
        : `mica_${colorMode}`;

      // Mutating on every start would rewrite the settings file for nothing.
      if (next === window_effect) {
        return;
      }

      updateAppSettings.mutate(
        { windowEffect: next },
        { onError: (err) => logError('Failed to update mica', err) },
      );
    },
    50,
  );

  createEffect(
    on([() => theme.rawTheme, colorMode], ([rawTheme, colorMode]) => {
      updateMicaDebounced(rawTheme, colorMode);
    }),
  );
};
