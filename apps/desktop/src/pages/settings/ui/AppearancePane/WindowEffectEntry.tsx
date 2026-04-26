import { type Component } from 'solid-js';

import { useTranslation } from '@/shared/model';
import { SettingsEntry } from '@/shared/ui';

import { SelectWindowEffect } from './SelectWindowEffect';

export type WindowEffectEntryProps = {
  class?: string;
};

export const WindowEffectEntry: Component<WindowEffectEntryProps> = (props) => {
  const [{ t }] = useTranslation();

  return (
    <SettingsEntry
      title={t('settings.windowEffect')}
      isTopLevel={false}
      {...props}
    >
      <SelectWindowEffect />
    </SettingsEntry>
  );
};
