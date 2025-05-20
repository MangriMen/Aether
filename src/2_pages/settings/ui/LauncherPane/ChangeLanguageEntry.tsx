import { createMemo, type Component, type ComponentProps } from 'solid-js';

import type { Locale, Option } from '@/shared/model';

import { useTranslate } from '@/shared/model';

import { LOCALE_OPTIONS } from '../../model';
import SelectLanguage from '../LauncherPane/SelectLanguage';
import { SettingsEntry } from '@/shared/ui';

export type ChangeLanguageEntryProps = ComponentProps<'div'>;

const ChangeLanguageEntry: Component<ChangeLanguageEntryProps> = (props) => {
  const [{ locale, t }, { setLocale }] = useTranslate();

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

export default ChangeLanguageEntry;
