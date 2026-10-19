import type { Component, ComponentProps } from 'solid-js';

import IconMdiCubeOutline from '~icons/mdi/cube-outline';
import { createSignal, Show, splitProps } from 'solid-js';

import { cn } from '@/shared/utils';

import { ResponsiveIcon } from './ResponsiveIcon';

export type ImageProps = ComponentProps<'div'> &
  Pick<ComponentProps<'img'>, 'src' | 'alt'> & {
    imgClass?: string;
  };

export const Image: Component<ImageProps> = (props) => {
  const [local, others] = splitProps(props, [
    'src',
    'alt',
    'class',
    'imgClass',
  ]);

  // Track the src that failed rather than a boolean, so a new src is retried
  // without an effect.
  const [failedSrc, setFailedSrc] = createSignal<string>();

  const handleError = () => {
    setFailedSrc(local.src);
  };

  return (
    <div
      class={cn(
        `
          size-16 rounded-md bg-secondary/secondary flex items-center
          justify-center border
        `,
        local.class,
      )}
      {...others}
    >
      <Show
        when={local.src && failedSrc() !== local.src}
        fallback={
          <ResponsiveIcon
            class='text-foreground/85'
            icon={IconMdiCubeOutline}
          />
        }
      >
        <img
          class={cn('size-full', local.imgClass)}
          src={local.src}
          alt={local.alt ?? ''}
          onError={handleError}
        />
      </Show>
    </div>
  );
};
