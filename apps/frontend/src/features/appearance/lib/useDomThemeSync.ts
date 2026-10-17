import { createEffect, type Accessor } from 'solid-js';

import type { ThemeContextValue } from '../model/themeContext';

import {
  setAnimationDisabledToDocument,
  setThemeToDocument,
  setTransparencyToDocument,
} from '../model/themeDom';

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
    if (state.isActualTransparent) {
      setTransparencyToDocument(transparencyProperty(), state.transparency);
    } else {
      setTransparencyToDocument(transparencyProperty(), 1);
    }
  });

  createEffect(() => {
    setAnimationDisabledToDocument(
      disableAnimationsAttribute(),
      state.disableAnimations,
    );
  });
};
