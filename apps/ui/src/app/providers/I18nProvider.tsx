import * as i18n from '@solid-primitives/i18n';
import { makePersisted } from '@solid-primitives/storage';
import {
  createMemo,
  createResource,
  createSignal,
  onMount,
  splitProps,
  type JSX,
} from 'solid-js';

import type {
  Dictionary,
  I18nContextType,
  Locale,
  RawDictionary,
} from '@/shared/model';

import { dayjs } from '@/shared/lib';
import { getSystemLocale, I18nContext } from '@/shared/model';

import { LOCALE_LS_KEY } from '../config';

export type I18nProviderProps<
  Locale extends string,
  Dictionary extends i18n.BaseRecordDict,
> = {
  resources: Record<Locale, Dictionary>;
  fallbackLocale: Locale;
  children?: JSX.Element;
};

export const I18nProvider = (
  props: I18nProviderProps<Locale, RawDictionary>,
) => {
  const [local, others] = splitProps(props, ['resources', 'fallbackLocale']);

  const getLocale = (): Locale => {
    const locale = getSystemLocale();
    if (!(locale in local.resources)) {
      return local.fallbackLocale;
    }
    return locale as Locale;
  };

  const [locale, setLocale_] = makePersisted(
    // eslint-disable-next-line solid/reactivity
    createSignal<Locale>(getLocale()),
    { name: LOCALE_LS_KEY },
  );

  const combinedSignal = createMemo(() => [locale(), local.resources] as const);

  const [dict] = createResource(combinedSignal, ([locale, resources]) =>
    i18n.flatten(resources[locale]),
  );

  const t = i18n.translator(dict, i18n.resolveTemplate);

  const setLocale = (locale: Locale) => {
    dayjs.locale(locale);
    // eslint-disable-next-line @typescript-eslint/no-unsafe-function-type
    setLocale_(locale as Exclude<Locale, Function>);
  };

  const context: I18nContextType<Locale, Dictionary> = [
    {
      locale,
      dict,
      t,
    },
    {
      setLocale,
    },
  ];

  onMount(() => {
    dayjs.locale(locale());
  });

  return (
    <I18nContext.Provider value={context}>
      {others.children}
    </I18nContext.Provider>
  );
};
