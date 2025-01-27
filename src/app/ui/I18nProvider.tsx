import * as i18n from '@solid-primitives/i18n';
import { makePersisted } from '@solid-primitives/storage';
import {
  createResource,
  createSignal,
  onMount,
  type Component,
  type JSX,
} from 'solid-js';

import { dayjs } from '@/shared/lib';

import enDictionary from '../i18n/en.json';
import type { I18nContextType, Locale } from '../model';
import {
  fetchDictionary,
  getSystemLocale,
  I18nContext,
  LOCALE_KEY,
} from '../model';

export type I18nProviderProps = { children?: JSX.Element };

const I18nProvider: Component<I18nProviderProps> = (props) => {
  const [locale, setLocale_] = makePersisted(
    // eslint-disable-next-line solid/reactivity
    createSignal<Locale>(getSystemLocale()),
    { name: LOCALE_KEY },
  );

  const [dict] = createResource(locale, fetchDictionary, {
    initialValue: i18n.flatten(enDictionary),
  });

  const t = i18n.translator(dict, i18n.resolveTemplate);

  const setLocale = (locale: Locale) => {
    dayjs.locale(locale);
    setLocale_(locale);
  };

  const context: I18nContextType = [
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
      {props.children}
    </I18nContext.Provider>
  );
};

export default I18nProvider;
