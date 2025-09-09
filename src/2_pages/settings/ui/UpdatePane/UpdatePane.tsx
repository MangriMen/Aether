import { SettingsPane } from '@/shared/ui';
import { splitProps, type Component, type ComponentProps } from 'solid-js';
import { UpdateAppEntry } from './UpdateAppEntry';
import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';

export type UpdatePaneProps = ComponentProps<'div'>;

export const UpdatePane: Component<UpdatePaneProps> = (props) => {
  const [local, others] = splitProps(props, ['class']);

  const [{ t }] = useTranslation();

  return (
    <SettingsPane
      class={cn('container max-w-screen-lg', local.class)}
      label={t('settings.tab.update')}
      {...others}
    >
      <UpdateAppEntry />
    </SettingsPane>
  );
};
