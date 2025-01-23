import { Icon } from '@iconify-icon/solid';
import { Component, ComponentProps, Show, splitProps } from 'solid-js';

export type InstanceImageProps = ComponentProps<'div'> &
  Pick<ComponentProps<'img'>, 'src'>;

export const InstanceImage: Component<InstanceImageProps> = (props) => {
  const [local, others] = splitProps(props, ['src', 'class']);

  return (
    <div class='size-24 rounded-lg border bg-secondary' {...others}>
      <Show
        when={local.src}
        fallback={<Icon class='text-[96px]' icon='mdi-cube-outline' />}
      >
        {(path) => <img src={path()} />}
      </Show>
    </div>
  );
};
