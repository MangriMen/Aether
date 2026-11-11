import { createEffect, type Accessor } from 'solid-js';

import type { ThemeContextValue } from '../model/themeContext';

import { FONT_SANS_PROPERTY } from '../model/themeConstants';
import {
  setAnimationDisabledToDocument,
  setFontSansToDocument,
  setThemeToDocument,
  setTransparencyToDocument,
} from '../model/themeDom';
import { resolveFontSans } from '../model/uiFont';

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
    setFontSansToDocument(
      FONT_SANS_PROPERTY,
      resolveFontSans(state.uiFont, state.customFontFamily),
    );
  });

  createEffect(() => {
    setAnimationDisabledToDocument(
      disableAnimationsAttribute(),
      state.disableAnimations,
    );
  });
};
