import { useAppSettings, useUpdateAppSettings } from '@/2_pages/settings/api';
import { debounce } from '@/6_shared/lib';
import { isSystemTheme, useThemeContext } from '@/6_shared/model';
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
      const mode = appSettings.data?.mica;
      if (!mode || mode === 'off') {
        return;
      }

      updateAppSettings.mutateAsync({
        mica: isSystemTheme(rawTheme) ? 'system' : colorMode,
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
