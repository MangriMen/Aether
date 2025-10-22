import type { Component, ComponentProps } from 'solid-js';

import IconMdiCubeOutline from '~icons/mdi/cube-outline';
import { createSignal, Show, splitProps } from 'solid-js';

import { cn } from '@/shared/lib';
import { ResponsiveIcon } from '@/shared/ui';

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
        'size-24 rounded-lg border bg-secondary flex items-center justify-center p-1',
        local.class,
      )}
      {...others}
    >
      <Show
        when={local.src && isValidSrc()}
        fallback={<ResponsiveIcon icon={IconMdiCubeOutline} />}
      >
        <img
          class='size-full p-1'
          src={local.src}
          alt={local.alt}
          onError={handleError}
        />
      </Show>
    </div>
  );
};
