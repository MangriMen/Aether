import { ConfigColorMode } from '@kobalte/core';

export type Theme = ConfigColorMode | 'aether-light' | 'aether-dark';

export const THEME_TO_MODE: Record<Theme, ConfigColorMode> = {
  light: 'light',
  dark: 'dark',
  'aether-light': 'light',
  'aether-dark': 'dark',
  system: 'system',
};

const THEME_ATTRIBUTE = 'data-theme';

export const getTheme = (): Theme | null => {
  return (localStorage.getItem(THEME_ATTRIBUTE) ??
    window.document.documentElement.getAttribute(
      THEME_ATTRIBUTE,
    )) as Theme | null;
};

export const setTheme = (theme: Theme) => {
  localStorage.setItem(THEME_ATTRIBUTE, theme);
  window.document.documentElement.setAttribute(THEME_ATTRIBUTE, theme);
};
