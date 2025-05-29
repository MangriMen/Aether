const TRANSPARENCY_ATTRIBUTE_KEY = 'data-transparency';

export const getTransparencyAttribute = (): boolean =>
  Boolean(
    window.document.documentElement.getAttribute(TRANSPARENCY_ATTRIBUTE_KEY),
  );

export const setTransparencyAttribute = (transparency: boolean) => {
  if (transparency) {
    window.document.documentElement.setAttribute(
      TRANSPARENCY_ATTRIBUTE_KEY,
      'true',
    );
  } else {
    window.document.documentElement.removeAttribute(TRANSPARENCY_ATTRIBUTE_KEY);
  }
};
