import { KB_THEME_ATTRIBUTE, THEME_ATTRIBUTE } from '../config';
import { setThemeToDocument } from './theme';

export const setFallbackAttributes = () => {
  if (typeof document === 'undefined') {
    return;
  }

  const theme: 'light' | 'dark' = 'dark';

  setThemeToDocument(THEME_ATTRIBUTE, theme);
  document.documentElement.setAttribute(KB_THEME_ATTRIBUTE, theme);
  document.documentElement.style.colorScheme = theme;
};
