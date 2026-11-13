import type { ColorMode, ConfigColorMode } from '@kobalte/core';

import type { TFunction } from '@/shared/i18n';

export type Theme = ColorMode | string;
export type ThemeObject = {
  value: Theme;
  /** The theme's own name, shown as is (a proper name, like a custom theme's). */
  name: string;
  /** Set only when the name is an ordinary word that has a translation. */
  labelKey?: 'light' | 'dark';
  mode: ColorMode;
};

export type ThemeConfig = ConfigColorMode | string;

export const THEMES: ThemeObject[] = [
  { value: 'light', name: 'Light', labelKey: 'light', mode: 'light' },
  { value: 'aether-light', name: 'Aether Light', mode: 'light' },
  { value: 'dark', name: 'Dark', labelKey: 'dark', mode: 'dark' },
  { value: 'aether-dark', name: 'Aether Dark', mode: 'dark' },
];

export const getThemeLabel = (theme: ThemeObject, t: TFunction): string =>
  (theme.labelKey ? t(`settings.themes.${theme.labelKey}`) : undefined) ??
  theme.name;

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
