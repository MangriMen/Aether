import type { ColorMode } from '@kobalte/core';
import { useColorMode } from '@kobalte/core';
import type { Accessor } from 'solid-js';
import { createEffect, on, onMount } from 'solid-js';
import { createStore } from 'solid-js/store';

import type {
  Theme,
  ThemeConfig,
  ThemeContextActions,
  ThemeContextType,
  ThemeContextValue,
} from '@/shared/model';
import { THEMES_MAP, COLOR_MODE_TO_KEY, isSystemTheme } from '@/shared/model';
import { DEFAULT_THEME_CONTEXT_VALUE } from '../config';
import { makePersisted } from '@solid-primitives/storage';
import {
  applyDisableAnimationsToDocument,
  applyThemeToDocument,
  applyTransparencyToDocument,
} from './theme';
import { makeMediaQueryListener } from '@solid-primitives/media';

import { showPrefersReducedMotionInfo } from './showPrefersReducedMotionInfo';

export const useCreateThemeContext = (
  themeStateKey: Accessor<string>,
  themeAttribute: Accessor<string>,
  disableAnimationsAttribute: Accessor<string>,
  transparencyProperty: Accessor<string>,
): ThemeContextType => {
  const { colorMode: currentColorMode, setColorMode } = useColorMode();

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
    setState(COLOR_MODE_TO_KEY[colorMode], theme);
  };

  const setTransparency = (transparency: number) => {
    setState('transparency', transparency);
  };

  const setDisableAnimations = (disableAnimations: boolean) => {
    setState('disableAnimations', disableAnimations);
  };

  const actions: ThemeContextActions = {
    setTheme: setRawTheme,
    setThemeByColorMode,
    setTransparency,
    setDisableAnimations,
  };

  const setColorModeByTheme = (theme: ThemeConfig): void => {
    const mode = isSystemTheme(theme) ? theme : THEMES_MAP[theme].mode;
    setColorMode(mode);
  };

  const updateThemeByColorMode = () => {
    setTheme(state[COLOR_MODE_TO_KEY[currentColorMode()]]);
  };

  const setPrefersReducedMotion = (prefersReducedMotion: boolean) => {
    setState('prefersReducedMotion', prefersReducedMotion);
  };

  createEffect(
    on(
      [
        () => state.rawTheme,
        () => state.lightTheme,
        () => state.darkTheme,
        currentColorMode,
      ],
      ([rawTheme]) => {
        setColorModeByTheme(rawTheme);

        if (isSystemTheme(rawTheme)) updateThemeByColorMode();
        else setTheme(rawTheme);
      },
    ),
  );

  makeMediaQueryListener('(prefers-reduced-motion: reduce)', (e) => {
    setPrefersReducedMotion(e.matches);
    setDisableAnimations(e.matches);

    if (e.matches && state.disableAnimations) {
      showPrefersReducedMotionInfo(() => setDisableAnimations(false));
    }
  });

  createEffect(() => {
    applyThemeToDocument(themeAttribute(), state.theme);
  });

  createEffect(() => {
    applyTransparencyToDocument(transparencyProperty(), state.transparency);
  });

  createEffect(() => {
    applyDisableAnimationsToDocument(
      disableAnimationsAttribute(),
      state.disableAnimations,
    );
  });

  onMount(() => {
    setRawTheme(state.rawTheme);
  });

  return [state, actions];
};
