import type { Component, ComponentProps } from 'solid-js';

import { useTranslation } from '@/shared/model';
import { SettingsEntry } from '@/shared/ui';

import { SelectTheme } from './SelectTheme';

export type SelectThemeEntryProps = ComponentProps<'div'>;

export const SelectThemeEntry: Component<SelectThemeEntryProps> = (props) => {
  const [{ t }] = useTranslation();

  return (
    <SettingsEntry
      title={t('settings.colorScheme')}
      description={t('settings.colorSchemeDescription')}
      {...props}
    >
      <SelectTheme />
    </SettingsEntry>
  );
};
