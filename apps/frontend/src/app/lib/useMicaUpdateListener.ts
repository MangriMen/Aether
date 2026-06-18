import type { ColorMode } from '@kobalte/core';

import { useColorMode } from '@kobalte/core';
import { createEffect, on } from 'solid-js';

import { useAppSettings, useEditAppSettings } from '@/entities/settings';
import { debounce } from '@/shared/lib';
import { isSystemTheme, useThemeContext } from '@/shared/model';

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
          updateAppSettings.mutateAsync({
            windowEffect: isSystemTheme(rawTheme)
              ? 'mica'
              : `mica_${colorMode}`,
          });
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
