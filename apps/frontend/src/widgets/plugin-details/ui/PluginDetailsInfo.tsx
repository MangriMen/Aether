import {
  createMemo,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

import { PluginInfoHeader, type Plugin } from '@/entities/plugins';
import { cn } from '@/shared/lib';

export type PluginDetailsInfoProps = ComponentProps<'div'> & {
  plugin: Plugin;
};

export const PluginDetailsInfo: Component<PluginDetailsInfoProps> = (props) => {
  const [local, others] = splitProps(props, ['plugin', 'class']);

  const metadata = createMemo(() => local.plugin.manifest.metadata);
  const authorsStr = createMemo(() => metadata().authors?.join(', '));

  return (
    <div class={cn('flex flex-col', local.class)} {...others}>
      <PluginInfoHeader
        name={metadata().name}
        version={metadata().version}
        apiVersion={local.plugin.manifest.api.version}
        authors={authorsStr()}
        description={metadata().description}
      />
    </div>
  );
};
