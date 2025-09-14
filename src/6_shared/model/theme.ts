import type { ColorMode, ConfigColorMode } from '@kobalte/core';

export type Theme = ColorMode | string;
export type ThemeConfig = ConfigColorMode | string;

export type ThemeObject = { mode: ColorMode; name: string; value: Theme };

export const THEMES: ThemeObject[] = [
  { mode: 'light', name: 'Light', value: 'light' },
  { mode: 'light', name: 'Aether Light', value: 'aether-light' },
  { mode: 'dark', name: 'Dark', value: 'dark' },
  { mode: 'dark', name: 'Aether Dark', value: 'aether-dark' },
];

export const THEMES_MAP = THEMES.reduce<Record<Theme, ThemeObject>>(
  (acc, theme) => {
    acc[theme.value] = theme;
    return acc;
  },
  {},
);

export const THEME_TO_MODE: Record<ThemeConfig, ConfigColorMode> = {
  'aether-dark': 'dark',
  'aether-light': 'light',
  dark: 'dark',
  light: 'light',
  system: 'system',
};

export const THEME_BY_MODE: Record<ColorMode, ThemeObject[]> = {
  dark: THEMES.filter((t) => t.mode === 'dark'),
  light: THEMES.filter((t) => t.mode === 'light'),
};

export const isSystemTheme = (theme: ThemeConfig): theme is 'system' =>
  theme === 'system';
