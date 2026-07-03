import type { Component, ComponentProps } from 'solid-js';

import { splitProps } from 'solid-js';

import { cn } from '../../lib';
import { IconButton } from './IconButton';

export type TitleBarButtonProps = ComponentProps<'div'>;

export const TitleBarButton: Component<TitleBarButtonProps> = (props) => {
  const [local, others] = splitProps(props, ['class']);

  return (
    <IconButton
      class={cn(
        `
          size-10
          hover:text-primary-foreground
          min-w-max
        `,
        local.class,
      )}
      variant='ghost'
      {...others}
    />
  );
};
