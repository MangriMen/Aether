import { createEffect, type Accessor } from 'solid-js';

import type { ThemeContextValue } from '@/shared/model';

import {
  setAnimationDisabledToDocument,
  setThemeToDocument,
  setTransparencyToDocument,
} from './theme';

export const useThemeContextDomSync = (
  state: ThemeContextValue,
  themeAttribute: Accessor<string>,
  disableAnimationsAttribute: Accessor<string>,
  transparencyProperty: Accessor<string>,
) => {
  createEffect(() => {
    setThemeToDocument(themeAttribute(), state.theme);
  });

  createEffect(() => {
    setTransparencyToDocument(transparencyProperty(), state.transparency);
  });

  createEffect(() => {
    setAnimationDisabledToDocument(
      disableAnimationsAttribute(),
      state.disableAnimations,
    );
  });
};
