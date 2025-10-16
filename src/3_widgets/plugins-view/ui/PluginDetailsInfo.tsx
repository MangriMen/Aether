import {
  createMemo,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

import type { Plugin } from '@/entities/plugins';

import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';

export type PluginDetailsInfoProps = ComponentProps<'div'> & {
  plugin: Plugin;
};

export const PluginDetailsInfo: Component<PluginDetailsInfoProps> = (props) => {
  const [local, others] = splitProps(props, ['plugin', 'class']);

  const [{ t }] = useTranslation();

  const metadata = createMemo(() => local.plugin.manifest.metadata);
  const authorsStr = createMemo(() => metadata().authors?.join(', '));

  return (
    <div class={cn('flex flex-col', local.class)} {...others}>
      <div class='flex items-end gap-2'>
        <h2 class='text-xl font-bold'>{metadata().name}</h2>
        <span class='text-muted-foreground'>
          {t('common.version')}: {metadata().version}
        </span>
      </div>
      <span class='text-muted-foreground'>
        {t('common.authors')}: {authorsStr()}
      </span>
    </div>
  );
};
