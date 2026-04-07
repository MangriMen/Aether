import IconMdiAlertOutline from '~icons/mdi/alert-outline';
import {
  createMemo,
  Show,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

import {
  checkIsApiCompatible,
  useApiVersion,
  type Plugin,
} from '@/entities/plugins';
import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { CombinedTooltip } from '@/shared/ui';

export type PluginDetailsInfoProps = ComponentProps<'div'> & {
  plugin: Plugin;
};

export const PluginDetailsInfo: Component<PluginDetailsInfoProps> = (props) => {
  const [local, others] = splitProps(props, ['plugin', 'class']);

  const [{ t }] = useTranslation();

  const metadata = createMemo(() => local.plugin.manifest.metadata);
  const authorsStr = createMemo(() => metadata().authors?.join(', '));

  const launcherApiVersion = useApiVersion();
  const api = createMemo(() => local.plugin.manifest.api);

  const isApiCompatible = createMemo(() => {
    if (!launcherApiVersion.data) {
      return false;
    }

    return checkIsApiCompatible(launcherApiVersion.data, api().version);
  });

  return (
    <div class={cn('flex flex-col', local.class)} {...others}>
      <div class='flex items-end gap-2'>
        <h2 class='text-xl font-bold'>{metadata().name}</h2>
        <div class='flex items-end gap-1'>
          <span class='text-muted-foreground'>{metadata().version}</span>
          <Show when={Boolean(launcherApiVersion.data) && !isApiCompatible()}>
            <CombinedTooltip
              label={t('plugin.incompatibleApiVersion')}
              as={IconMdiAlertOutline}
              class='mb-[3px] text-warning-foreground'
            />
          </Show>
        </div>
      </div>
      <div class='flex gap-4'>
        <span>{authorsStr()}</span>
      </div>
      <span class='my-1'>{metadata().description}</span>
    </div>
  );
};
