import { type Component, type ComponentProps, splitProps } from 'solid-js';

import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { SettingsPane } from '@/shared/ui';

import { UpdateAppEntry } from './UpdateAppEntry';

export type UpdatePaneProps = ComponentProps<'div'>;

export const UpdatePane: Component<UpdatePaneProps> = (props) => {
  const [local, others] = splitProps(props, ['class']);

  const [{ t }] = useTranslation();

  return (
    <SettingsPane
      class={cn('container max-w-screen-lg', local.class)}
      label={t('settings.update')}
      {...others}
    >
      <UpdateAppEntry />
    </SettingsPane>
  );
};
