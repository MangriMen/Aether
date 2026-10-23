import type { ThemeContextValue } from './themeContext';

import { DEFAULT_THEME_CONTEXT_VALUE } from './themeConstants';

export const migrateThemeContext = (value: unknown): ThemeContextValue => {
  const defaults = DEFAULT_THEME_CONTEXT_VALUE;

  if (value === null || value === undefined || typeof value !== 'object') {
    return DEFAULT_THEME_CONTEXT_VALUE;
  }

  const stored = value as Record<string, unknown>;

  // A hand-edited or damaged entry must not put a value of the wrong type into the theme.
  const pick = <K extends keyof ThemeContextValue>(key: K) => {
    const candidate = stored[key];

    return typeof candidate === typeof defaults[key]
      ? (candidate as ThemeContextValue[K])
      : defaults[key];
  };

  return {
    theme: pick('theme'),
    rawTheme: pick('rawTheme'),
    lightTheme: pick('lightTheme'),
    darkTheme: pick('darkTheme'),
    isActualTransparent: pick('isActualTransparent'),
    transparencyEnabled: pick('transparencyEnabled'),
    transparency: pick('transparency'),
    disableAnimations: pick('disableAnimations'),
    prefersReducedMotion: pick('prefersReducedMotion'),
  };
};
