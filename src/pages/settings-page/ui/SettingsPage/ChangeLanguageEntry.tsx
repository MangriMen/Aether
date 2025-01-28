import { createMemo, type Component, type ComponentProps } from 'solid-js';

import type { Option } from '@/shared/model';

import { SelectLanguage } from '@/features/select-language';


import type { Locale } from '@/app/model';

import { LOCALES_OPTIONS, useTranslate } from '@/app/model';

import { SettingsEntry } from '../SettingsEntry';

export type ChangeLanguageEntryProps = ComponentProps<'div'>;

const ChangeLanguageEntry: Component<ChangeLanguageEntryProps> = (props) => {
  const [{ locale, t }, { setLocale }] = useTranslate();

  const currentOption = createMemo(() =>
    LOCALES_OPTIONS.find((option) => option.value === locale()),
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
        options={LOCALES_OPTIONS}
        value={currentOption()}
        onChange={handleChangeLocale}
      />
    </SettingsEntry>
  );
};

export default ChangeLanguageEntry;
