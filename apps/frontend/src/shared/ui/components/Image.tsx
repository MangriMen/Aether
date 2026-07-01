import type { Component, ComponentProps } from 'solid-js';

import IconMdiCubeOutline from '~icons/mdi/cube-outline';
import { createSignal, Show, splitProps } from 'solid-js';

import { ResponsiveIcon } from '..';
import { cn } from '../../lib';

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

  const [isValidSrc, setIsValidSrc] = createSignal(true);

  const handleError = () => {
    setIsValidSrc(false);
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
        when={local.src && isValidSrc()}
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
          alt={local.alt}
          onError={handleError}
        />
      </Show>
    </div>
  );
};
