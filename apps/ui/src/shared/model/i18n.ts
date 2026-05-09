import type { Flatten } from '@solid-primitives/i18n';

import { enDictionary, ruDictionary } from '../i18n';

export const LOCALES = {
  En: 'en',
  Ru: 'ru',
} as const;

export const LOCALE_RESOURCES = {
  [LOCALES.En]: enDictionary,
  [LOCALES.Ru]: ruDictionary,
};

export type Locale = (typeof LOCALES)[keyof typeof LOCALES];

export type RawDictionary = (typeof LOCALE_RESOURCES)[Locale];
export type Dictionary = Flatten<RawDictionary>;

export const getLocaleFromBrowser = () => {
  return navigator.language.split('-')[0];
};

export const getSystemLocale = () => getLocaleFromBrowser();
