import type { ThemeContextValue } from '@/shared/model';

import { DEFAULT_THEME_CONTEXT_VALUE } from '../config';

export const migrateThemeContext = (value: unknown): ThemeContextValue => {
  const defaults = DEFAULT_THEME_CONTEXT_VALUE;

  if (value === null || value === undefined || typeof value !== 'object') {
    return DEFAULT_THEME_CONTEXT_VALUE;
  }

  const uValue = value as Partial<ThemeContextValue>;

  return {
    theme: uValue['theme'] ?? defaults['theme'],
    rawTheme: uValue['rawTheme'] ?? defaults['rawTheme'],
    lightTheme: uValue['lightTheme'] ?? defaults['lightTheme'],
    darkTheme: uValue['darkTheme'] ?? defaults['darkTheme'],
    transparencyEnabled:
      uValue['transparencyEnabled'] ?? defaults['transparencyEnabled'],
    transparency: uValue['transparency'] ?? defaults['transparency'],
    disableAnimations:
      uValue['disableAnimations'] ?? defaults['disableAnimations'],
    prefersReducedMotion:
      uValue['prefersReducedMotion'] ?? defaults['prefersReducedMotion'],
  };
};
