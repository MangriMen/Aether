import { useAppSettings, useUpdateAppSettings } from '@/pages/settings/api';
import { debounce } from '@/shared/lib';
import { isSystemTheme, useThemeContext } from '@/shared/model';
import type { ColorMode } from '@kobalte/core';
import { useColorMode } from '@kobalte/core';
import { createEffect, on } from 'solid-js';

export const useMicaUpdateListener = () => {
  const { colorMode } = useColorMode();

  const [theme] = useThemeContext();

  const appSettings = useAppSettings();
  const updateAppSettings = useUpdateAppSettings();

  const updateMicaDebounced = debounce(
    (rawTheme: string, colorMode: ColorMode) => {
      const window_effect = appSettings.data?.windowEffect;
      if (!window_effect || window_effect === 'off') {
        return;
      }

      updateAppSettings.mutateAsync({
        windowEffect: isSystemTheme(rawTheme) ? 'mica' : `mica_${colorMode}`,
      });
    },
    50,
  );

  createEffect(
    on([() => theme.rawTheme, colorMode], ([rawTheme, colorMode]) => {
      updateMicaDebounced(rawTheme, colorMode);
    }),
  );
};
