import { Component, splitProps } from 'solid-js';

import { cn } from '@/shared/lib';
import { IconButton } from '@/shared/ui';

import { InstanceButtonProps } from './types';

export const InstanceButton: Component<InstanceButtonProps> = (props) => {
  const [local, others] = splitProps(props, ['class']);

  return (
    <IconButton
      class={cn(
        'absolute p-0 pr-0.5 bottom-1/3 left-1/2 opacity-0 transition-[bottom,opacity] group-hover:bottom-1/4 group-hover:opacity-100',
        local.class,
      )}
      {...others}
    />
  );
};
