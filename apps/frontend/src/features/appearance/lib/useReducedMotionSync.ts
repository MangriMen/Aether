import { batch, createEffect, on } from 'solid-js';

import type { ThemeContextValue } from '../model/themeContext';

import { showPrefersReducedMotionInfo } from '../ui/PrefersReducedMotionToast';
import { createPrefersReducedMotion } from './createPrefersReducedMotion';

export const useReducedMotionSync = (
  state: ThemeContextValue,
  setPrefersReducedMotion: (value: boolean) => void,
  setDisableAnimations: (value: boolean) => void,
) => {
  const isPrefersReducedMotion = createPrefersReducedMotion();

  const handleEnableAnimations = () => {
    setDisableAnimations(false);
  };

  createEffect(
    on(isPrefersReducedMotion, (prefersReducedMotion) => {
      batch(() => {
        setPrefersReducedMotion(prefersReducedMotion);
        setDisableAnimations(prefersReducedMotion);
      });

      if (prefersReducedMotion && state.disableAnimations) {
        showPrefersReducedMotionInfo(handleEnableAnimations);
      }
    }),
  );
};
