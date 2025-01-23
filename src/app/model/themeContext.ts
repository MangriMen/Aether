import { ColorMode } from '@kobalte/core';
import { createContext, useContext } from 'solid-js';

import { Theme, ThemeConfig } from '@/shared/model';

export type ThemeData = {
  theme: Theme;
  rawTheme: ThemeConfig;
  lightTheme: Theme;
  darkTheme: Theme;
};

export type ThemeContextValue = [
  ThemeData,
  {
    setTheme: (theme: ThemeConfig) => void;
    setThemeForColorMode: (colorMode: ColorMode, theme: Theme) => void;
  },
];

// const DEFAULT_VALUE: ThemeContextValue = [
//   { theme: 'aether-dark', themeRaw: 'aether-dark' },
//   { setTheme: () => {} },
// ];

export const DEFAULT_THEME: ThemeConfig = 'aether-dark';

export const ThemeContext = createContext<ThemeContextValue>();

export const useThemeContext = () => {
  const value = useContext(ThemeContext);

  if (!value) {
    throw new Error('Missing context Provider');
  }

  return value;
};
