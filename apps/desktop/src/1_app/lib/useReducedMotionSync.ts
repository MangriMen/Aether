import { batch, createEffect, on } from 'solid-js';

import type { ThemeContextValue } from '@/shared/model';

import { createPrefersReducedMotion } from '@/shared/lib';

import { showPrefersReducedMotionInfo } from './showPrefersReducedMotionInfo';

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
