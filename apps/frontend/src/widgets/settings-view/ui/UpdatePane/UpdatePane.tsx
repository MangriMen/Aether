import { type Component } from 'solid-js';

import type { SettingsPaneProps } from '@/shared/ui';

import { useTranslation } from '@/shared/model';
import { SettingsPane } from '@/shared/ui';

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
