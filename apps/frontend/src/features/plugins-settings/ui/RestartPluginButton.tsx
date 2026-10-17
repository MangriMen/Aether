import IconMdiRestart from '~icons/mdi/restart';
import { type Component } from 'solid-js';

import { useReloadPlugin, type PluginId } from '@/features/plugins';
import { useTranslation } from '@/shared/i18n';
import { Button } from '@/shared/ui-kit';

export type RestartPluginButtonProps = {
  pluginId: PluginId;
  disabled?: boolean;
  class?: string;
};

/**
 * Restarts a running plugin on the user's request. A refusal (`Busy`) comes up as a toast, and
 * the plugin goes on as it was.
 */
export const RestartPluginButton: Component<RestartPluginButtonProps> = (
  props,
) => {
  const [{ t }] = useTranslation();

  const reloadPlugin = useReloadPlugin();

  return (
    <Button
      size='sm'
      variant='secondary'
      class={props.class}
      leadingIcon={IconMdiRestart}
      loading={reloadPlugin.isPending}
      disabled={props.disabled || reloadPlugin.isPending}
      onClick={() => reloadPlugin.mutate(props.pluginId)}
    >
      {t('plugin.restart')}
    </Button>
  );
};
