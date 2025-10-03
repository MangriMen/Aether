import IconMdiFolder from '~icons/mdi/folder';
import IconMdiReload from '~icons/mdi/reload';
import { splitProps, type Component, type ComponentProps } from 'solid-js';

import { useOpenPluginFolder, useSyncPlugins } from '@/entities/plugins';
import { cn } from '@/shared/lib';
import { isLauncherError, useTranslation } from '@/shared/model';
import { CombinedTooltip, IconButton, showToast } from '@/shared/ui';

export type PluginsPaneTitleProps = ComponentProps<'div'>;

export const PluginsPaneTitle: Component<PluginsPaneTitleProps> = (props) => {
  const [local, others] = splitProps(props, ['class']);

  const [{ t }] = useTranslation();

  const openPluginsFolder = useOpenPluginFolder();
  const syncPlugins = useSyncPlugins();

  const handleOpenPluginsFolder = async () => {
    try {
      await openPluginsFolder.mutateAsync();
    } catch (e) {
      if (isLauncherError(e)) {
        showToast({
          title: 'Failed to open plugins folder',
          variant: 'destructive',
          description: e.message,
        });
      }
    }
  };

  const handleRefreshPlugins = async () => {
    await syncPlugins.mutateAsync();
  };

  return (
    <div
      class={cn('flex justify-between items-center', local.class)}
      {...others}
    >
      {t('plugins.title')}
      <div class='flex gap-2'>
        <CombinedTooltip
          label={t('plugins.openPluginsFolder')}
          as={IconButton}
          icon={IconMdiFolder}
          onClick={handleOpenPluginsFolder}
        />
        <CombinedTooltip
          label={t('common.refresh')}
          as={IconButton}
          icon={IconMdiReload}
          onClick={handleRefreshPlugins}
        />
      </div>
    </div>
  );
};
