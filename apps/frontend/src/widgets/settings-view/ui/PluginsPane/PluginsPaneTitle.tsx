import { splitProps, type Component, type ComponentProps } from 'solid-js';

import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';

import { AddPluginIconButton } from './AddPluginIconButton';
import { InstallFromPluginPackageButton } from './InstallFromPluginPackageButton';
import { OpenPluginsFolderButton } from './OpenPluginsFolderButton';
import { SyncPluginsButton } from './SyncPluginsButton';

export type PluginsPaneTitleProps = ComponentProps<'div'> & {
  onPluginAddClick: () => void;
};

export const PluginsPaneTitle: Component<PluginsPaneTitleProps> = (props) => {
  const [local, others] = splitProps(props, ['onPluginAddClick', 'class']);

  const [{ t }] = useTranslation();

  return (
    <div
      class={cn('pt-0.5 flex items-center justify-between', local.class)}
      {...others}
    >
      <h2>{t('plugins.title')}</h2>
      <div class='gap-2 flex'>
        <AddPluginIconButton onClick={local.onPluginAddClick} />
        <InstallFromPluginPackageButton />
        <OpenPluginsFolderButton />
        <SyncPluginsButton />
      </div>
    </div>
  );
};
