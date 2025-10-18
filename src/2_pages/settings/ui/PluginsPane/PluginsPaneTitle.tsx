import { splitProps, type Component, type ComponentProps } from 'solid-js';

import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';

import { OpenPluginsFolderButton } from './OpenPluginsFolderButton';
import { SyncPluginsButton } from './SyncPluginsButton';

export type PluginsPaneTitleProps = ComponentProps<'div'>;

export const PluginsPaneTitle: Component<PluginsPaneTitleProps> = (props) => {
  const [local, others] = splitProps(props, ['class']);

  const [{ t }] = useTranslation();

  return (
    <div
      class={cn('flex justify-between items-center', local.class)}
      {...others}
    >
      <h2>{t('plugins.title')}</h2>
      <div class='flex gap-2'>
        <OpenPluginsFolderButton />
        <SyncPluginsButton />
      </div>
    </div>
  );
};
