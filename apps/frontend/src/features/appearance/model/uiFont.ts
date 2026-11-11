export const UI_FONTS = {
  System: 'system',
  Inter: 'inter',
  Custom: 'custom',
} as const;

export type UiFont = (typeof UI_FONTS)[keyof typeof UI_FONTS];

export const isUiFont = (value: unknown): value is UiFont =>
  Object.values<unknown>(UI_FONTS).includes(value);

export const MAX_CUSTOM_FONT_FAMILY_LENGTH = 100;

/** The family of the bundled Inter (`@fontsource-variable/inter`). */
const INTER_FAMILY = 'Inter Variable';

/** Tailwind's own (the installed version's) `--font-sans`: what the fonts after the chosen one fall back to. */
const DEFAULT_SANS_STACK =
  '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", "Noto Sans", Arial, sans-serif, "Apple Color Emoji", "Segoe UI Emoji", "Segoe UI Symbol", "Noto Color Emoji"';

/** A family name from the user, safe to put between quotes of a `font-family` value. */
export const sanitizeFontFamily = (value: string): string =>
  value
    .replace(/["'\\;{}<>:]|\p{Cc}/gu, '')
    .trim()
    .slice(0, MAX_CUSTOM_FONT_FAMILY_LENGTH)
    .trim();

/** The `--font-sans` value for the choice, or `undefined` to keep the default stack. */
export const resolveFontSans = (
  font: UiFont,
  customFontFamily: string,
): string | undefined => {
  switch (font) {
    case UI_FONTS.Inter:
      return `'${INTER_FAMILY}', ${DEFAULT_SANS_STACK}`;

    case UI_FONTS.Custom: {
      const family = sanitizeFontFamily(customFontFamily);

      return family ? `'${family}', ${DEFAULT_SANS_STACK}` : undefined;
    }

    case UI_FONTS.System:
      return undefined;
  }
};
