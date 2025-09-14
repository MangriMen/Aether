import type {
  BaseRecordDict,
  NullableTranslator,
} from '@solid-primitives/i18n';
import type { Accessor, Resource } from 'solid-js';

import { createContext, useContext } from 'solid-js';

import type { Dictionary, Locale } from './i18n';

export type I18nContextActions<Locale> = {
  setLocale: (locale: Locale) => void;
};

export type I18nContextType<Locale, Dictionary extends BaseRecordDict> = [
  I18nContextValue<Locale, Dictionary>,
  I18nContextActions<Locale>,
];

export type I18nContextValue<Locale, Dictionary extends BaseRecordDict> = {
  dict: Resource<Dictionary>;
  locale: Accessor<Locale>;
  t: NullableTranslator<Dictionary>;
};

export const I18nContext = createContext<I18nContextType<Locale, Dictionary>>();

export type TFunction = NullableTranslator<Dictionary>;

const useI18nContext = () => {
  const value = useContext(I18nContext);

  if (!value) {
    throw new Error(
      '`useTranslate` must be used within a `I18nProvider` component',
    );
  }

  return value;
};

export const useTranslation = useI18nContext;
