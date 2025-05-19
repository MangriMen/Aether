import type { ColorMode } from '@kobalte/core';
import { useColorMode } from '@kobalte/core';
import type { Component, JSX } from 'solid-js';
import { onMount, splitProps } from 'solid-js';
import { createStore } from 'solid-js/store';

import type {
  Theme,
  ThemeConfig,
  ThemeContextActions,
  ThemeContextType,
  ThemeContextValue,
} from '@/shared/model';
import { THEMES_MAP, THEME_BY_MODE, ThemeContext } from '@/shared/model';
import { DEFAULT_THEME } from '../config';

export type ThemeObserverProps = {
  themeLsKey: string;
  rawThemeLsKey: string;
  themeAttribute: string;
  children?: JSX.Element;
};

const COLOR_MODES = ['light', 'dark'] as ColorMode[];

const setThemeToWindow = (theme: Theme, themeAttribute: string) => {
  window.document.documentElement.setAttribute(themeAttribute, theme);
};

export const ThemeProvider: Component<ThemeObserverProps> = (props) => {
  const [local, others] = splitProps(props, [
    'themeLsKey',
    'rawThemeLsKey',
    'themeAttribute',
  ]);

  const colorModeContext = useColorMode();

  const [contextValue, setContextValue] = createStore<ThemeContextValue>({
    theme: DEFAULT_THEME,
    rawTheme: DEFAULT_THEME,
    lightTheme: THEME_BY_MODE['light'][0].value,
    darkTheme: THEME_BY_MODE['dark'][0].value,
  });

  const getThemeVariableNameByColorMode = (colorMode: ColorMode) =>
    colorMode === 'light' ? 'lightTheme' : 'darkTheme';

  const getThemeByColorMode = (colorMode: ColorMode) =>
    contextValue[getThemeVariableNameByColorMode(colorMode)];

  const updateColorModeByTheme = (theme: ThemeConfig) => {
    const colorMode = theme === 'system' ? theme : THEMES_MAP[theme].mode;
    colorModeContext.setColorMode(colorMode);
    return colorModeContext.colorMode();
  };

  const setRawThemeField = (theme: ThemeConfig) => {
    localStorage.setItem(local.rawThemeLsKey, theme);
    setContextValue('rawTheme', theme);
  };

  const setThemeField = (theme: Theme) => {
    localStorage.setItem(local.themeLsKey, theme);
    setContextValue('theme', theme);
    setThemeToWindow(theme, local.themeAttribute);
  };

  const setThemeForColorMode = (colorMode: ColorMode, theme: Theme) => {
    localStorage.setItem(`${colorMode}-${local.themeLsKey}`, theme);
    setContextValue(getThemeVariableNameByColorMode(colorMode), theme);

    const isSystemTheme = contextValue.rawTheme === 'system';
    const isCurrentColorMode = colorMode === colorModeContext.colorMode();

    if (isSystemTheme && isCurrentColorMode) {
      setThemeField(getThemeByColorMode(colorMode));
    }
  };

  const setTheme = (theme: ThemeConfig) => {
    setRawThemeField(theme);
    setThemeField(
      theme === 'system'
        ? getThemeByColorMode(updateColorModeByTheme(theme))
        : theme,
    );
  };

  const contextActions: ThemeContextActions = {
    setTheme,
    setThemeForColorMode,
  };

  const context: ThemeContextType = [contextValue, contextActions];

  const initializeContext = () => {
    const rawTheme = localStorage.getItem(
      local.rawThemeLsKey,
    ) as ThemeConfig | null;
    setTheme(rawTheme ?? DEFAULT_THEME);

    const colorModeThemes = COLOR_MODES.map(
      (colorMode) =>
        localStorage.getItem(
          `${colorMode}-${local.themeLsKey}`,
        ) as Theme | null,
    );
    colorModeThemes.map((theme, index) => {
      setThemeForColorMode(COLOR_MODES[index], theme ?? COLOR_MODES[index]);
    });
  };
  onMount(() => {
    initializeContext();
  });

  return (
    <ThemeContext.Provider value={context}>
      {others.children}
    </ThemeContext.Provider>
  );
};
