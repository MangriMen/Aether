import type { ColorMode } from '@kobalte/core';

import { createContext, useContext } from 'solid-js';

import type { Theme, ThemeConfig } from '@/shared/model';

export type ThemeContextActions = {
  setDisableAnimations: (disableAnimations: boolean) => void;
  setTheme: (theme: ThemeConfig) => void;
  setThemeByColorMode: (colorMode: ColorMode, theme: Theme) => void;
  setTransparency: (transparency: number) => void;
};

export type ThemeContextType = [ThemeContextValue, ThemeContextActions];

export type ThemeContextValue = {
  darkTheme: Theme;
  disableAnimations: boolean;
  lightTheme: Theme;
  prefersReducedMotion: boolean;
  rawTheme: ThemeConfig;
  theme: Theme;
  transparency: number;
};

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
  Extract<keyof ThemeContextValue, 'darkTheme' | 'lightTheme'>
> = {
  dark: 'darkTheme',
  light: 'lightTheme',
};
