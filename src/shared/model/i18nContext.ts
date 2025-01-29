import type {
  BaseRecordDict,
  NullableTranslator,
} from '@solid-primitives/i18n';
import type { Accessor, Resource } from 'solid-js';
import { createContext, useContext } from 'solid-js';

export type I18nContextValue<Locale, Dictionary extends BaseRecordDict> = {
  locale: Accessor<Locale>;
  dict: Resource<Dictionary>;
  t: NullableTranslator<Dictionary>;
};

export type I18nContextActions<Locale> = {
  setLocale: (locale: Locale) => void;
};

export type I18nContextType<Locale, Dictionary extends BaseRecordDict> = [
  I18nContextValue<Locale, Dictionary>,
  I18nContextActions<Locale>,
];

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export const I18nContext = createContext<I18nContextType<any, any>>();

export const useI18nContext = () => {
  const value = useContext(I18nContext);

  if (!value) {
    throw new Error(
      '`useTranslate` must be used within a `I18nProvider` component',
    );
  }

  return value;
};

export const useTranslate = useI18nContext;
