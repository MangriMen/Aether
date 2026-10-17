import type { ColorMode, ConfigColorMode } from '@kobalte/core';
import type { Accessor } from 'solid-js';

import { makePersisted } from '@solid-primitives/storage';
import { batch, createEffect, on, onCleanup, onMount } from 'solid-js';
import { createStore } from 'solid-js/store';

import { logError } from '@/shared/log';

import type { Theme, ThemeConfig } from '../model/theme';
import type {
  ThemeContextActions,
  ThemeContextType,
  ThemeContextValue,
} from '../model/themeContext';

import { migrateThemeContext } from '../model/migrateThemeContext';
import { isSystemTheme, THEME_TO_COLOR_MODE } from '../model/theme';
import {
  COLOR_MODE_CHANGE_DEBOUNCE_DELAY,
  DEFAULT_THEME_CONTEXT_VALUE,
} from '../model/themeConstants';
import { COLOR_MODE_TO_THEME_KEY } from '../model/themeContext';
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
    setIsActualTransparent,
    setTransparencyEnabled,
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

  const resolveColorMode = (
    rawTheme: ThemeConfig,
  ): ConfigColorMode | undefined => THEME_TO_COLOR_MODE[rawTheme];

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

          if (newColorMode === undefined) {
            logError(`Can't resolve color mode ${rawTheme}`);
            return;
          }

          setColorMode(newColorMode);
          setTheme(newTheme);
        });
      },
    ),
  );

  let prevThemeSet = 0;
  let trailingTimeout: ReturnType<typeof setTimeout> | undefined;

  onCleanup(() => clearTimeout(trailingTimeout));

  const setSystemTheme = () => {
    const rawTheme = state.rawTheme;

    if (isSystemTheme(rawTheme)) {
      setTheme(resolveTheme(rawTheme));
      prevThemeSet = performance.now();
    }
  };

  // Applies the first change at once and the last one of a burst after the delay,
  // so a quick back-and-forth of the system color mode still ends on the right theme.
  const setSystemThemeThrottled = () => {
    clearTimeout(trailingTimeout);

    const elapsed = performance.now() - prevThemeSet;

    if (elapsed > COLOR_MODE_CHANGE_DEBOUNCE_DELAY) {
      setSystemTheme();
      return;
    }

    trailingTimeout = setTimeout(
      setSystemTheme,
      COLOR_MODE_CHANGE_DEBOUNCE_DELAY - elapsed,
    );
  };

  createEffect(
    on([colorMode], () => {
      if (!isSystemTheme(state.rawTheme)) {
        return;
      }

      setSystemThemeThrottled();
    }),
  );

  onMount(() => {
    setRawTheme(state.rawTheme);
  });

  const actions: ThemeContextActions = {
    setTheme: setRawTheme,
    setThemeByColorMode,
    setIsActualTransparent,
    setTransparencyEnabled,
    setTransparency,
    setDisableAnimations,
  };

  return [state, actions];
};

const createContext = (themeStateKey: Accessor<string>) => {
  const [state, setState] = makePersisted(
    createStore<ThemeContextValue>(DEFAULT_THEME_CONTEXT_VALUE),
    { name: themeStateKey() },
  );

  onMount(() => {
    setState(migrateThemeContext(state));
  });

  const setRawTheme = (theme: ThemeConfig) => {
    setState('rawTheme', theme);
  };

  const setTheme = (theme: Theme) => {
    setState('theme', theme);
  };

  const setThemeByColorMode = (colorMode: ColorMode, theme: Theme) => {
    setState(COLOR_MODE_TO_THEME_KEY[colorMode], theme);
  };

  const setIsActualTransparent = (isActualTransparent: boolean) => {
    setState('isActualTransparent', isActualTransparent);
  };

  const setTransparencyEnabled = (isEnabled: boolean) => {
    setState('transparencyEnabled', isEnabled);
  };

  const setTransparency = (transparency: number) => {
    const clampedTransparency = Math.max(Math.min(transparency, 1), 0);
    setState('transparency', clampedTransparency);
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
    setIsActualTransparent,
    setTransparencyEnabled,
    setTransparency,
    setDisableAnimations,
    setPrefersReducedMotion,
  };
};
