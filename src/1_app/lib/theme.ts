import type { Theme } from '@/shared/model';

export const applyThemeToDocument = (attribute: string, theme: Theme): void => {
  document.documentElement.setAttribute(attribute, theme);
};

export const applyTransparencyToDocument = (
  property: string,
  transparency: number,
) => {
  document.documentElement.style.setProperty(property, transparency.toString());
};

export const applyDisableAnimationsToDocument = (
  attribute: string,
  disableAnimations: boolean,
) => {
  if (disableAnimations) {
    document.documentElement.setAttribute(attribute, 'true');
  } else {
    document.documentElement.removeAttribute(attribute);
  }
};
