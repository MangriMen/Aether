import type { ColorMode, ConfigColorMode } from '@kobalte/core';
import type { Accessor } from 'solid-js';

import { makePersisted } from '@solid-primitives/storage';
import { batch, createEffect, on, onMount } from 'solid-js';
import { createStore } from 'solid-js/store';

import type {
  Theme,
  ThemeConfig,
  ThemeContextActions,
  ThemeContextType,
  ThemeContextValue,
} from '@/shared/model';

import {
  COLOR_MODE_TO_THEME_KEY,
  isSystemTheme,
  THEME_TO_COLOR_MODE,
} from '@/shared/model';

import {
  COLOR_MODE_CHANGE_DEBOUNCE_DELAY,
  DEFAULT_THEME_CONTEXT_VALUE,
} from '../config';
import { useThemeContextDomSync } from './useDomThemeSync';
import { useGlobalColorMode } from './useGlobalColorMode';
import { useReducedMotionSync } from './useReducedMotionSync';

export const createThemeContext = (
  themeStateKey: Accessor<string>,
  themeAttribute: Accessor<string>,
  disableAnimationsAttribute: Accessor<string>,
  transparencyProperty: Accessor<string>,
): ThemeContextType => {
  const {
    state,
    setRawTheme,
    setTheme,
    setThemeByColorMode,
    setTransparency,
    setDisableAnimations,
    setPrefersReducedMotion,
  } = createContext(themeStateKey);

  useThemeContextDomSync(
    state,
    themeAttribute,
    disableAnimationsAttribute,
    transparencyProperty,
  );

  useReducedMotionSync(state, setPrefersReducedMotion, setDisableAnimations);

  const { colorMode, setColorMode } = useGlobalColorMode();

  const resolveColorMode = (rawTheme: ThemeConfig): ConfigColorMode =>
    THEME_TO_COLOR_MODE[rawTheme];

  const resolveTheme = (rawTheme: ThemeConfig): Theme => {
    if (isSystemTheme(rawTheme)) {
      return state[COLOR_MODE_TO_THEME_KEY[colorMode()]];
    }

    return rawTheme;
  };

  createEffect(
    on(
      [() => state.rawTheme, () => state.lightTheme, () => state.darkTheme],
      ([rawTheme]) => {
        batch(() => {
          const newColorMode = resolveColorMode(rawTheme);
          const newTheme = resolveTheme(rawTheme);

          setColorMode(newColorMode);
          setTheme(newTheme);
        });
      },
    ),
  );

  let prevThemeSet = 0;
  const setThemeLeadingEdge = (theme: Theme) => {
    const currentTime = performance.now();

    const setWithTime = () => {
      setTheme(theme);
      prevThemeSet = performance.now();
    };

    if (currentTime - prevThemeSet > COLOR_MODE_CHANGE_DEBOUNCE_DELAY) {
      setWithTime();
    }
  };

  createEffect(
    on([colorMode], () => {
      const rawTheme = state.rawTheme;

      if (!isSystemTheme(rawTheme)) {
        return;
      }

      setThemeLeadingEdge(resolveTheme(rawTheme));
    }),
  );

  onMount(() => {
    setRawTheme(state.rawTheme);
  });

  const actions: ThemeContextActions = {
    setTheme: setRawTheme,
    setThemeByColorMode: setThemeByColorMode,
    setTransparency: setTransparency,
    setDisableAnimations: setDisableAnimations,
  };

  return [state, actions];
};

const createContext = (themeStateKey: Accessor<string>) => {
  const [state, setState] = makePersisted(
    // eslint-disable-next-line solid/reactivity
    createStore<ThemeContextValue>(DEFAULT_THEME_CONTEXT_VALUE),
    { name: themeStateKey() },
  );

  const setRawTheme = (theme: ThemeConfig) => {
    setState('rawTheme', theme);
  };

  const setTheme = (theme: Theme) => {
    setState('theme', theme);
  };

  const setThemeByColorMode = (colorMode: ColorMode, theme: Theme) => {
    setState(COLOR_MODE_TO_THEME_KEY[colorMode], theme);
  };

  const setTransparency = (transparency: number) => {
    setState('transparency', transparency);
  };

  const setDisableAnimations = (disableAnimations: boolean) => {
    setState('disableAnimations', disableAnimations);
  };

  const setPrefersReducedMotion = (prefersReducedMotion: boolean) => {
    setState('prefersReducedMotion', prefersReducedMotion);
  };

  return {
    state,
    setRawTheme,
    setTheme,
    setThemeByColorMode,
    setTransparency,
    setDisableAnimations,
    setPrefersReducedMotion,
  };
};
