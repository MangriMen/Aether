import { type Component } from 'solid-js';

import type { SettingsPaneProps } from '@/shared/settings-layout';

import { useTranslation } from '@/shared/i18n';
import { SettingsPane } from '@/shared/settings-layout';

import { UpdateAppEntry } from './UpdateAppEntry';
import { UpdateNotificationStyleEntry } from './UpdateNotificationStyleEntry';

export type UpdatePaneProps = SettingsPaneProps;

export const UpdatePane: Component<UpdatePaneProps> = (props) => {
  const [{ t }] = useTranslation();

  return (
    <SettingsPane label={t('settings.tab.update')} {...props}>
      <UpdateNotificationStyleEntry variant='card' />
      <UpdateAppEntry variant='card' />
    </SettingsPane>
  );
};
