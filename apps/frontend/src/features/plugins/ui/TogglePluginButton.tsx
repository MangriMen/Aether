import {
  Match,
  splitProps,
  Switch,
  type Component,
  type ComponentProps,
} from 'solid-js';

import { useTranslation } from '@/shared/model';
import { Button } from '@/shared/ui';

import { useDisablePlugin } from '../api/pluginQueries';
import { useForceEnablePluginWithDialog } from '../lib/useForceEnablePluginWithDialog';
import { usePluginStates } from '../lib/usePluginStates';
import { type Plugin } from '../model/plugin';

export type TogglePluginButtonProps = ComponentProps<'div'> & {
  plugin: Plugin;
  disabled?: boolean;
};

export const TogglePluginButton: Component<TogglePluginButtonProps> = (
  props,
) => {
  const [local, others] = splitProps(props, ['plugin', 'class', 'disabled']);

  const [{ t }] = useTranslation();

  const { isDisabled, isLoading, isEnabled } = usePluginStates(
    () => local.plugin.state,
  );

  const { tryEnable } = useForceEnablePluginWithDialog();
  const { mutateAsync: disablePlugin } = useDisablePlugin();

  const togglePluginEnabled = async () => {
    if (isLoading()) {
      return;
    }

    try {
      const id = local.plugin.manifest.metadata.id;

      if (isEnabled()) {
        await disablePlugin(id);
      } else {
        await tryEnable(local.plugin);
      }
    } catch {
      /* empty */
    }
  };

  return (
    <Button
      size='sm'
      disabled={local.disabled}
      loading={isLoading()}
      onClick={togglePluginEnabled}
      {...others}
    >
      <Switch>
        <Match when={isDisabled() || isLoading()}>{t('common.enable')}</Match>
        <Match when={isEnabled()}>{t('common.disable')}</Match>
      </Switch>
    </Button>
  );
};
