import type { ColorMode } from '@kobalte/core';
import { useColorMode } from '@kobalte/core';
import type { Component, JSX } from 'solid-js';
import { onMount } from 'solid-js';
import { createStore } from 'solid-js/store';

import type { Theme, ThemeConfig } from '@/shared/model';
import { THEMES_MAP } from '@/shared/model';

import type { ThemeContextValue } from '../model';
import { DEFAULT_THEME, ThemeContext } from '../model';

export type ThemeObserverProps = { children?: JSX.Element };

const THEME_KEY = 'theme';
const RAW_THEME_KEY = 'theme-raw';

const THEME_ATTRIBUTE = 'data-theme';

const COLOR_MODES = ['light', 'dark'] as ColorMode[];

const setThemeToWindow = (theme: Theme) => {
  window.document.documentElement.setAttribute(THEME_ATTRIBUTE, theme);
};

const ThemeObserver: Component<ThemeObserverProps> = (props) => {
  const colorModeContext = useColorMode();

  const [contextValue, setContextValue] = createStore<ThemeContextValue[0]>({
    theme: DEFAULT_THEME,
    rawTheme: DEFAULT_THEME,
    lightTheme: 'light',
    darkTheme: 'dark',
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
    localStorage.setItem(RAW_THEME_KEY, theme);
    setContextValue('rawTheme', theme);
  };

  const setThemeField = (theme: Theme) => {
    localStorage.setItem(THEME_KEY, theme);
    setContextValue('theme', theme);
    setThemeToWindow(theme);
  };

  const setThemeForColorMode = (colorMode: ColorMode, theme: Theme) => {
    localStorage.setItem(`${colorMode}-${THEME_KEY}`, theme);
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

  const context: ThemeContextValue = [
    contextValue,
    {
      setTheme,
      setThemeForColorMode,
    },
  ];

  const initializeContext = () => {
    const rawTheme = localStorage.getItem(RAW_THEME_KEY) as ThemeConfig | null;
    setTheme(rawTheme ?? DEFAULT_THEME);

    const colorModeThemes = COLOR_MODES.map(
      (colorMode) =>
        localStorage.getItem(`${colorMode}-${THEME_KEY}`) as Theme | null,
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
      {props.children}
    </ThemeContext.Provider>
  );
};

export default ThemeObserver;
