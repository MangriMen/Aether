import {
  createMemo,
  createSignal,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

import {
  useDisablePlugin,
  useEnablePlugin,
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

  const isPluginEnabled = createMemo(() => local.plugin.state === 'Loaded');

  const { mutateAsync: enablePlugin } = useEnablePlugin();
  const { mutateAsync: disablePlugin } = useDisablePlugin();

  const [isLoading, setIsLoading] = createSignal(false);

  const togglePluginEnabled = async () => {
    if (isLoading()) {
      return;
    }

    setIsLoading(true);
    try {
      if (isPluginEnabled()) {
        await disablePlugin(local.plugin.manifest.metadata.id);
      } else {
        await enablePlugin(local.plugin.manifest.metadata.id);
      }
    } catch {
      /* empty */
    }
    setIsLoading(false);
  };

  return (
    <Button
      size='sm'
      onClick={togglePluginEnabled}
      loading={isLoading()}
      {...others}
    >
      {isPluginEnabled() ? t('common.disable') : t('common.enable')}
    </Button>
  );
};
