import * as i18n from '@solid-primitives/i18n';
import { makePersisted } from '@solid-primitives/storage';
import {
  createResource,
  createSignal,
  type Component,
  type JSX,
} from 'solid-js';

import enDictionary from '../i18n/en.json';
import type {
  I18nContextActions,
  I18nContextType,
  I18nContextValue,
  Locale,
} from '../model';
import { fetchDictionary, getLocaleFromBrowser, I18nContext } from '../model';

export type I18nProviderProps = { children?: JSX.Element };

const LOCALE_KEY = 'locale';

const I18nProvider: Component<I18nProviderProps> = (props) => {
  const [locale, setLocale_] = makePersisted(
    // eslint-disable-next-line solid/reactivity
    createSignal<Locale>(getLocaleFromBrowser()),
    { name: LOCALE_KEY },
  );

  const [dict] = createResource(locale, fetchDictionary, {
    initialValue: i18n.flatten(enDictionary),
  });

  const t = i18n.translator(dict);

  const contextValue: I18nContextValue = {
    locale,
    dict,
    t,
  };

  const setLocale = (locale: Locale) => {
    setLocale_(locale);
  };

  const contextActions: I18nContextActions = { setLocale };

  const context: I18nContextType = [contextValue, contextActions];

  return (
    <I18nContext.Provider value={context}>
      {props.children}
    </I18nContext.Provider>
  );
};

export default I18nProvider;
