import type { Component, ComponentProps } from 'solid-js';

import IconMdiFolder from '~icons/mdi/folder';

import { useOpenPluginFolder } from '@/features/plugins';
import { useTranslation } from '@/shared/i18n';
import { logError } from '@/shared/log';
import { CombinedTooltip, IconButton } from '@/shared/ui-kit';

export type OpenPluginsFolderButtonProps = ComponentProps<'button'>;

export const OpenPluginsFolderButton: Component<
  OpenPluginsFolderButtonProps
> = (props) => {
  const [{ t }] = useTranslation();

  const openPluginsFolder = useOpenPluginFolder();

  const handleOpenPluginsFolder = async () => {
    try {
      await openPluginsFolder.mutateAsync();
    } catch (error) {
      logError('Failed to open the plugins folder:', error);
    }
  };

  return (
    <CombinedTooltip
      label={t('plugins.openPluginsFolder')}
      as={IconButton}
      variant='secondary'
      icon={IconMdiFolder}
      onClick={handleOpenPluginsFolder}
      {...props}
    />
  );
};
