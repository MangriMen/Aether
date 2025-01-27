import type { NullableTranslator } from '@solid-primitives/i18n';
import type { Accessor, Resource } from 'solid-js';
import { createContext, useContext } from 'solid-js';

import type { Dictionary, Locale } from './i18n';

export type I18nContextValue = {
  locale: Accessor<Locale>;
  dict: Resource<Dictionary>;
  t: NullableTranslator<Dictionary>;
};

export type I18nContextActions = {
  setLocale: (locale: Locale) => void;
};

export type I18nContextType = [I18nContextValue, I18nContextActions];

export const I18nContext = createContext<I18nContextType>();

export const useI18nContext = () => {
  const value = useContext(I18nContext);

  if (!value) {
    throw new Error(
      '`useI18nContext` must be used within a `I18nProvider` component',
    );
  }

  return value;
};
