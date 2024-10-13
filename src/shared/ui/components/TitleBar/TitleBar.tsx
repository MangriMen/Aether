import { Component, splitProps } from 'solid-js';

import { cn } from '@/shared/lib';

import { TitleBarProps } from './types';

export const TitleBar: Component<TitleBarProps> = (props) => {
  const [local, others] = splitProps(props, ['class']);
  return (
    <div
      class={cn(
        'fixed inset-x-0 top-0 flex h-[30px] select-none justify-end bg-secondary-dark',
        local.class,
      )}
      {...others}
    />
  );
};
