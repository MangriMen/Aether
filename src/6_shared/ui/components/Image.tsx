import type { Component, ComponentProps } from 'solid-js';

import MdiCubeOutline from '@iconify/icons-mdi/cube-outline';
import { Show, splitProps } from 'solid-js';

import { cn } from '@/shared/lib';
import { ResponsiveIcon } from '@/shared/ui';

export type InstanceImageProps = ComponentProps<'div'> &
  Pick<ComponentProps<'img'>, 'src'>;

export const Image: Component<InstanceImageProps> = (props) => {
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
        fallback={<ResponsiveIcon icon={MdiCubeOutline} />}
        when={local.src}
      >
        {(path) => <img alt='Instance' class='size-full p-1' src={path()} />}
      </Show>
    </div>
  );
};
