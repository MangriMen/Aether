import type { Theme } from '@/shared/model';

export const setThemeToDocument = (attribute: string, theme: Theme): void => {
  document.documentElement.setAttribute(attribute, theme);
};

export const setTransparencyToDocument = (
  property: string,
  transparency: number,
) => {
  document.documentElement.style.setProperty(property, transparency.toString());
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
