import { type Component } from 'solid-js';

import type { SettingsEntryProps } from '@/shared/ui';

import { useTranslation } from '@/shared/model';
import { SettingsEntry } from '@/shared/ui';

import { SelectNotificationStyle } from './SelectNotificationStyle';

export type UpdateNotificationStyleEntryProps = SettingsEntryProps;

export const UpdateNotificationStyleEntry: Component<
  UpdateNotificationStyleEntryProps
> = (props) => {
  const [{ t }] = useTranslation();

  return (
    <SettingsEntry title={t('settings.updateNotificationStyle')} {...props}>
      <SelectNotificationStyle />
    </SettingsEntry>
  );
};
