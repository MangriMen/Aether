import { type ThemeConfig, type ThemeContextValue } from '@/shared/model';

export const THEME_ATTRIBUTE = 'data-theme';
export const TRANSPARENCY_PROPERTY = '--transparency';
export const DISABLE_ANIMATIONS_ATTRIBUTE = 'data-disable-animations';

export const THEME_STATE_LS_KEY = 'theme-state';

export const DEFAULT_THEME: ThemeConfig = 'aether-dark';
export const DEFAULT_LIGHT_THEME = 'light';
export const DEFAULT_DARK_THEME = 'dark';

export const DEFAULT_THEME_CONTEXT_VALUE: ThemeContextValue = {
  darkTheme: DEFAULT_DARK_THEME,
  disableAnimations: false,
  lightTheme: DEFAULT_LIGHT_THEME,
  prefersReducedMotion: false,
  rawTheme: DEFAULT_THEME,
  theme: DEFAULT_THEME,
  transparency: 1,
};
