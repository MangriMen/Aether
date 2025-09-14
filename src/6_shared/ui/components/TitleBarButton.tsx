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
        'size-[30px] p-0 inline-flex items-center justify-center hover:bg-secondary',
        local.class,
      )}
      variant='ghost'
      {...others}
    />
  );
};
