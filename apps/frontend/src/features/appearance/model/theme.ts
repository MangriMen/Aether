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

const THEME_TO_COLOR_MODE_MAP = THEMES.reduce<
  Record<ThemeConfig, ConfigColorMode>
>((acc, theme) => {
  acc[theme.value] = theme.mode;
  return acc;
}, {});

export const THEME_TO_COLOR_MODE: Record<ThemeConfig, ConfigColorMode> = {
  ...THEME_TO_COLOR_MODE_MAP,
  system: 'system',
} as const;

export const THEME_BY_MODE: Record<ColorMode, ThemeObject[]> = {
  light: THEMES.filter((t) => t.mode === 'light'),
  dark: THEMES.filter((t) => t.mode === 'dark'),
};

export const isSystemTheme = (theme: ThemeConfig): theme is 'system' =>
  theme === 'system';
