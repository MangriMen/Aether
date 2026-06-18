import type { Component, ComponentProps } from 'solid-js';

import IconMdiCubeOutline from '~icons/mdi/cube-outline';
import { createSignal, Show, splitProps } from 'solid-js';

import { ResponsiveIcon } from '..';
import { cn } from '../../lib';

export type InstanceImageProps = ComponentProps<'div'> &
  Pick<ComponentProps<'img'>, 'src' | 'alt'>;

export const Image: Component<InstanceImageProps> = (props) => {
  const [local, others] = splitProps(props, ['src', 'alt', 'class']);

  const [isValidSrc, setIsValidSrc] = createSignal(true);

  const handleError = () => {
    setIsValidSrc(false);
  };

  return (
    <div
      class={cn(
        'size-16 rounded-lg flex items-center justify-center bg-secondary/secondary border',
        local.class,
      )}
      {...others}
    >
      <Show
        when={local.src && isValidSrc()}
        fallback={
          <ResponsiveIcon
            class='text-foreground/85'
            icon={IconMdiCubeOutline}
          />
        }
      >
        <img
          class='size-full'
          src={local.src}
          alt={local.alt}
          onError={handleError}
        />
      </Show>
    </div>
  );
};
