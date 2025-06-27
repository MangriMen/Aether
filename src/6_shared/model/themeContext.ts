import type { ColorMode } from '@kobalte/core';
import { createContext, useContext } from 'solid-js';

import type { Theme, ThemeConfig } from '@/shared/model';

export type ThemeContextValue = {
  theme: Theme;
  rawTheme: ThemeConfig;
  lightTheme: Theme;
  darkTheme: Theme;
  transparency: number;
  disableAnimations: boolean;
  prefersReducedMotion: boolean;
};

export type ThemeContextActions = {
  setTheme: (theme: ThemeConfig) => void;
  setThemeByColorMode: (colorMode: ColorMode, theme: Theme) => void;
  setTransparency: (transparency: number) => void;
  setDisableAnimations: (disableAnimations: boolean) => void;
};

export type ThemeContextType = [ThemeContextValue, ThemeContextActions];

export const ThemeContext = createContext<ThemeContextType>();

export const useThemeContext = () => {
  const value = useContext(ThemeContext);

  if (!value) {
    throw new Error('Missing context Provider');
  }

  return value;
};

export const COLOR_MODE_TO_KEY: Record<
  ColorMode,
  Extract<keyof ThemeContextValue, 'lightTheme' | 'darkTheme'>
> = {
  light: 'lightTheme',
  dark: 'darkTheme',
};
