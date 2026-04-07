import type { Component, ComponentProps } from 'solid-js';

import { splitProps } from 'solid-js';

import { cn } from '@/shared/lib';

import { IconButton } from './IconButton';

export type TitleBarButtonProps = ComponentProps<'div'>;

export const TitleBarButton: Component<TitleBarButtonProps> = (props) => {
  const [local, others] = splitProps(props, ['class']);

  return (
    <IconButton
      class={cn(
        'min-w-max w-[40px] h-titlebar hover:text-primary-foreground',
        local.class,
      )}
      variant='ghost'
      {...others}
    />
  );
};
