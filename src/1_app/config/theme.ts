import { type ThemeConfig, type ThemeContextValue } from '@/shared/model';

export const THEME_ATTRIBUTE = 'data-theme';
export const TRANSPARENCY_PROPERTY = '--transparency';

export const THEME_STATE_LS_KEY = 'theme-state';

export const DEFAULT_THEME: ThemeConfig = 'aether-dark';
export const DEFAULT_LIGHT_THEME = 'light';
export const DEFAULT_DARK_THEME = 'dark';

export const DEFAULT_THEME_CONTEXT_VALUE: ThemeContextValue = {
  theme: DEFAULT_THEME,
  rawTheme: DEFAULT_THEME,
  lightTheme: DEFAULT_LIGHT_THEME,
  darkTheme: DEFAULT_DARK_THEME,
  transparency: 1,
};
