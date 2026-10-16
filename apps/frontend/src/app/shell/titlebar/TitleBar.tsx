import type { Component, ComponentProps } from 'solid-js';

import { splitProps } from 'solid-js';

import { cn } from '@/shared/lib';

export type TitleBarProps = ComponentProps<'div'>;

export const TitleBar: Component<TitleBarProps> = (props) => {
  const [local, others] = splitProps(props, ['class']);
  return (
    <div
      class={cn('h-titlebar bg-card/card flex select-none', local.class)}
      {...others}
    />
  );
};
