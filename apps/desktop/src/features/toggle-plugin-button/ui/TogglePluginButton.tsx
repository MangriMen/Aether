import {
  Match,
  splitProps,
  Switch,
  type Component,
  type ComponentProps,
} from 'solid-js';

import {
  useDisablePlugin,
  useEnablePlugin,
  usePluginStates,
  type Plugin,
} from '@/entities/plugins';
import { useTranslation } from '@/shared/model';
import { Button } from '@/shared/ui';

export type TogglePluginButtonProps = ComponentProps<'div'> & {
  plugin: Plugin;
};

export const TogglePluginButton: Component<TogglePluginButtonProps> = (
  props,
) => {
  const [local, others] = splitProps(props, ['plugin', 'class']);

  const [{ t }] = useTranslation();

  const { isDisabled, isLoading, isEnabled } = usePluginStates(
    () => local.plugin.state,
  );

  const { mutateAsync: enablePlugin } = useEnablePlugin();
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
        await enablePlugin(id);
      }
    } catch {
      /* empty */
    }
  };

  return (
    <Button
      size='sm'
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
