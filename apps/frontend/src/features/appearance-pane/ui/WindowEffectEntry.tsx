import { type Component } from 'solid-js';

import { useTranslation } from '@/shared/i18n';
import { SettingsEntry } from '@/shared/settings-layout';

import { SelectWindowEffect } from './SelectWindowEffect';

export type WindowEffectEntryProps = {
  class?: string;
};

export const WindowEffectEntry: Component<WindowEffectEntryProps> = (props) => {
  const [{ t }] = useTranslation();

  return (
    <SettingsEntry title={t('settings.windowEffect')} {...props}>
      <SelectWindowEffect />
    </SettingsEntry>
  );
};
