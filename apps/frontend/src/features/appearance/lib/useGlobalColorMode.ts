import type { ConfigColorMode } from '@kobalte/core';
import type { Theme as WindowTheme } from '@tauri-apps/api/window';

import { useColorMode } from '@kobalte/core';
import { getCurrentWindow } from '@tauri-apps/api/window';

import { logError } from '@/shared/log';

const configColorModeToWindowThemeMap: Record<
  ConfigColorMode,
  WindowTheme | null
> = {
  light: 'light',
  dark: 'dark',
  system: null,
} as const;

export const useGlobalColorMode = () => {
  const { colorMode: browserColorMode, setColorMode: setBrowserColorMode } =
    useColorMode();

  const setColorMode = (value: ConfigColorMode) => {
    setBrowserColorMode(value);
    getCurrentWindow()
      .setTheme(configColorModeToWindowThemeMap[value])
      .catch((err: unknown) => {
        logError('Failed to set the window theme:', err);
      });
  };

  return { colorMode: browserColorMode, setColorMode };
};
