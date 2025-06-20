import { cn } from '@/shared/lib';
import { CombinedTooltip, IconButton, showToast } from '@/shared/ui';
import { splitProps, type Component, type ComponentProps } from 'solid-js';

import MdiFolderIcon from '@iconify/icons-mdi/folder';
import MdiReloadIcon from '@iconify/icons-mdi/reload';
import { openPluginsFolderRaw, syncPluginsRaw } from '@/entities/plugins';
import { isLauncherError, useTranslate } from '@/shared/model';

export type PluginsPaneTitleProps = ComponentProps<'div'>;

export const PluginsPaneTitle: Component<PluginsPaneTitleProps> = (props) => {
  const [local, others] = splitProps(props, ['class']);

  const [{ t }] = useTranslate();

  const handleOpenPluginsFolder = async () => {
    try {
      await openPluginsFolderRaw();
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
    await syncPluginsRaw();
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
          icon={MdiFolderIcon}
          onClick={handleOpenPluginsFolder}
        />
        <CombinedTooltip
          label={t('common.refresh')}
          as={IconButton}
          icon={MdiReloadIcon}
          onClick={handleRefreshPlugins}
        />
      </div>
    </div>
  );
};
