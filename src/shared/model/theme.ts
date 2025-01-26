import type { ColorMode, ConfigColorMode } from '@kobalte/core';

export type Theme = ColorMode | string;
export type ThemeObject = { value: Theme; name: string; mode: ColorMode };

export type ThemeConfig = ConfigColorMode | string;

export const THEMES: ThemeObject[] = [
  { value: 'light', name: 'Light', mode: 'light' },
  { value: 'aether-light', name: 'Aether Light', mode: 'light' },
  { value: 'dark', name: 'Dark', mode: 'dark' },
  { value: 'aether-dark', name: 'Aether Dark', mode: 'dark' },
];

export const THEMES_MAP = THEMES.reduce<Record<Theme, ThemeObject>>(
  (acc, theme) => {
    acc[theme.value] = theme;
    return acc;
  },
  {},
);

export const THEME_TO_MODE: Record<ThemeConfig, ConfigColorMode> = {
  light: 'light',
  'aether-light': 'light',
  dark: 'dark',
  'aether-dark': 'dark',
  system: 'system',
};

export const THEME_BY_MODE: Record<ColorMode, ThemeObject[]> = {
  light: THEMES.filter((t) => t.mode === 'light'),
  dark: THEMES.filter((t) => t.mode === 'dark'),
};
