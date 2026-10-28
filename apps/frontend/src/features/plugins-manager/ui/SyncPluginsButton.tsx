import type { Component, ComponentProps } from 'solid-js';

import IconMdiReload from '~icons/mdi/reload';

import { useSyncPlugins } from '@/features/plugins';
import { useTranslation } from '@/shared/i18n';
import { logError } from '@/shared/log';
import { CombinedTooltip, IconButton } from '@/shared/ui-kit';

export type SyncPluginsButtonProps = ComponentProps<'button'>;

export const SyncPluginsButton: Component<SyncPluginsButtonProps> = (props) => {
  const [{ t }] = useTranslation();

  const syncPlugins = useSyncPlugins();

  const handleRefreshPlugins = async () => {
    try {
      await syncPlugins.mutateAsync();
    } catch (error) {
      logError('Failed to sync plugins:', error);
    }
  };

  return (
    <CombinedTooltip
      label={t('common.refresh')}
      as={IconButton}
      variant='secondary'
      icon={IconMdiReload}
      onClick={handleRefreshPlugins}
      {...props}
    />
  );
};
