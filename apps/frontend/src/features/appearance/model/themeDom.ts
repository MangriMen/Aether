import type { Theme } from './theme';

export const setThemeToDocument = (attribute: string, theme: Theme): void => {
  document.documentElement.setAttribute(attribute, theme);
};

export const setTransparencyToDocument = (
  property: string,
  transparency: number,
) => {
  document.documentElement.style.setProperty(property, transparency.toString());
};

export const setFontSansToDocument = (
  property: string,
  fontSans: string | undefined,
) => {
  if (fontSans === undefined) {
    document.documentElement.style.removeProperty(property);
  } else {
    document.documentElement.style.setProperty(property, fontSans);
  }
};

export const setAnimationDisabledToDocument = (
  attribute: string,
  disableAnimations: boolean,
) => {
  if (disableAnimations) {
    document.documentElement.setAttribute(attribute, 'true');
  } else {
    document.documentElement.removeAttribute(attribute);
  }
};
