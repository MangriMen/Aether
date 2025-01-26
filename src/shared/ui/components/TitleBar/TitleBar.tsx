import type { Component } from 'solid-js';
import { splitProps } from 'solid-js';

import { cn } from '@/shared/lib';

import type { TitleBarProps } from './types';

export const TitleBar: Component<TitleBarProps> = (props) => {
  const [local, others] = splitProps(props, ['class']);
  return (
    <div
      class={cn('flex h-[40px] select-none bg-secondary-dark', local.class)}
      {...others}
    />
  );
};
