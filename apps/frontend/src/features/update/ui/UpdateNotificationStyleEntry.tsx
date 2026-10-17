import { type Component } from 'solid-js';

import type { SettingsEntryProps } from '@/shared/settings-layout';

import { useTranslation } from '@/shared/i18n';
import { SettingsEntry } from '@/shared/settings-layout';

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
