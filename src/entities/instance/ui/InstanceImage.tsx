import MdiCubeOutline from '@iconify/icons-mdi/cube-outline';
import type { Component, ComponentProps } from 'solid-js';
import { Show, splitProps } from 'solid-js';

import { cn } from '@/shared/lib';
import { ResponsiveIcon } from '@/shared/ui';

export type InstanceImageProps = ComponentProps<'div'> &
  Pick<ComponentProps<'img'>, 'src'>;

export const InstanceImage: Component<InstanceImageProps> = (props) => {
  const [local, others] = splitProps(props, ['src', 'class']);

  return (
    <div
      class={cn(
        'size-24 rounded-lg border bg-secondary flex items-center justify-center p-1',
        local.class,
      )}
      {...others}
    >
      <Show
        when={local.src}
        fallback={<ResponsiveIcon icon={MdiCubeOutline} />}
      >
        {(path) => <img src={path()} alt="Instance" />}
      </Show>
    </div>
  );
};
