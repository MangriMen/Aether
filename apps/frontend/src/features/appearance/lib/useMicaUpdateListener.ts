import type { ColorMode } from '@kobalte/core';

import { useColorMode } from '@kobalte/core';
import { createEffect, on } from 'solid-js';

import { useAppSettings, useEditAppSettings } from '@/features/settings';
import { debounce, logError } from '@/shared/utils';

import { isSystemTheme } from '../model/theme';
import { useThemeContext } from '../model/themeContext';

export const useMicaUpdateListener = () => {
  const { colorMode } = useColorMode();

  const [theme] = useThemeContext();

  const appSettings = useAppSettings();
  const updateAppSettings = useEditAppSettings();

  const updateMicaDebounced = debounce(
    (rawTheme: string, colorMode: ColorMode) => {
      const window_effect = appSettings.data?.windowEffect;

      switch (window_effect) {
        case 'mica':
        case 'mica_dark':
        case 'mica_light':
          updateAppSettings.mutate(
            {
              windowEffect: isSystemTheme(rawTheme)
                ? 'mica'
                : `mica_${colorMode}`,
            },
            { onError: (err) => logError('Failed to update mica', err) },
          );
          break;

        default:
          break;
      }
    },
    50,
  );

  createEffect(
    on([() => theme.rawTheme, colorMode], ([rawTheme, colorMode]) => {
      updateMicaDebounced(rawTheme, colorMode);
    }),
  );
};
