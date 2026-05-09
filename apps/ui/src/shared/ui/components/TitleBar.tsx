import type { Component, ComponentProps } from 'solid-js';

import { splitProps } from 'solid-js';

import { cn } from '../../lib';

export type TitleBarProps = ComponentProps<'div'>;

export const TitleBar: Component<TitleBarProps> = (props) => {
  const [local, others] = splitProps(props, ['class']);
  return (
    <div
      class={cn('flex h-titlebar select-none bg-card/card', local.class)}
      {...others}
    />
  );
};
