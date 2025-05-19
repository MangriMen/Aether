import { SettingsPane } from '@/shared/ui';
import { splitProps, type Component, type ComponentProps } from 'solid-js';
import { UpdateAppEntry } from './UpdateAppEntry';
import { cn } from '@/shared/lib';
import { useTranslate } from '@/shared/model';

export type UpdatePaneProps = ComponentProps<'div'>;

export const UpdatePane: Component<UpdatePaneProps> = (props) => {
  const [local, others] = splitProps(props, ['class']);

  const [{ t }] = useTranslate();

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
