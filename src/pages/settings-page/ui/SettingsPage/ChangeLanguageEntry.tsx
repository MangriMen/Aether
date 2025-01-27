import { createMemo, type Component, type ComponentProps } from 'solid-js';

import type { Option } from '@/shared/model';

import { SelectLanguage } from '@/features/select-language';

// eslint-disable-next-line boundaries/element-types
import type { Locale } from '@/app/model';
// eslint-disable-next-line boundaries/element-types
import { LOCALES_OPTIONS, useI18nContext } from '@/app/model';

import { SettingsEntry } from '../SettingsEntry';

export type ChangeLanguageEntryProps = ComponentProps<'div'>;

const ChangeLanguageEntry: Component<ChangeLanguageEntryProps> = (props) => {
  const [{ locale, t }, { setLocale }] = useI18nContext();

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
      title={t('language')}
      description={t('languageDescription')}
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
