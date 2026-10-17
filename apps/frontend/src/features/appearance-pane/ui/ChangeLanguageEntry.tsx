import { createMemo, type Component } from 'solid-js';

import type { Locale } from '@/shared/i18n';
import type { SettingsEntryProps } from '@/shared/settings-layout';
import type { Option } from '@/shared/utils';

import { useTranslation } from '@/shared/i18n';
import { SettingsEntry } from '@/shared/settings-layout';

import { LOCALE_OPTIONS } from '../model/localeOptions';
import SelectLanguage from './SelectLanguage';

export type ChangeLanguageEntryProps = SettingsEntryProps;

export const ChangeLanguageEntry: Component<ChangeLanguageEntryProps> = (
  props,
) => {
  const [{ locale, t }, { setLocale }] = useTranslation();

  const currentOption = createMemo(() =>
    LOCALE_OPTIONS.find((option) => option.value === locale()),
  );

  const handleChangeLocale = (locale: Option<Locale> | null) => {
    if (!locale) {
      return;
    }

    setLocale(locale.value);
  };

  return (
    <SettingsEntry
      title={t('settings.language')}
      description={t('settings.languageDescription')}
      {...props}
    >
      <SelectLanguage
        class='w-32'
        options={LOCALE_OPTIONS}
        value={currentOption()}
        onChange={handleChangeLocale}
      />
    </SettingsEntry>
  );
};
