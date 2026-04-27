import { splitProps, type Component, type ComponentProps } from 'solid-js';

import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { SettingsPane } from '@/shared/ui';

import { GoToPlaygroundEntry } from './GoToPlaygroundEntry';

export type ExperimentalPaneProps = ComponentProps<'div'>;

export const ExperimentalPane: Component<ExperimentalPaneProps> = (props) => {
  const [local, others] = splitProps(props, ['class']);

  const [{ t }] = useTranslation();

  return (
    <SettingsPane
      class={cn('container max-w-screen-lg', local.class)}
      label={t('settings.tab.experimental')}
      {...others}
    >
      <GoToPlaygroundEntry variant='card' />
    </SettingsPane>
  );
};
