import * as i18n from '@solid-primitives/i18n';

export const LOCALES = ['en', 'ru'] as const;
export const LOCALES_NAMES = ['English', 'Русский'] as const;
export const LOCALES_OPTIONS = LOCALES.map((locale, index) => ({
  name: LOCALES_NAMES[index],
  value: locale,
}));

export type Locale = (typeof LOCALES)[number];
export type RawDictionary = Record<string, string>;
export type Dictionary = i18n.Flatten<RawDictionary>;

export const getLocaleFromBrowser = () => {
  const browserLocale = navigator.language.split('-')[0] as Locale;
  return LOCALES.includes(browserLocale) ? browserLocale : 'en';
};

export const fetchDictionary = async (locale: Locale): Promise<Dictionary> => {
  const dict: RawDictionary = await import(`../i18n/${locale}.json`);
  return i18n.flatten(dict);
};
