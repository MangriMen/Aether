import type { Component, ComponentProps } from 'solid-js';

import IconMdiReload from '~icons/mdi/reload';

import { useSyncPlugins } from '@/entities/plugins';
import { useTranslation } from '@/shared/model';
import { CombinedTooltip, IconButton } from '@/shared/ui';

export type SyncPluginsButtonProps = ComponentProps<'button'>;

export const SyncPluginsButton: Component<SyncPluginsButtonProps> = (props) => {
  const [{ t }] = useTranslation();

  const syncPlugins = useSyncPlugins();
  const handleRefreshPlugins = async () => {
    await syncPlugins.mutateAsync();
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
