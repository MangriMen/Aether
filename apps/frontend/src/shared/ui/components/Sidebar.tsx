import type { Component, ComponentProps } from 'solid-js';

import { splitProps } from 'solid-js';

import { cn } from '../../lib';

export type SidebarProps = ComponentProps<'div'>;

export const Sidebar: Component<SidebarProps> = (props) => {
  const [local, others] = splitProps(props as SidebarProps, [
    'class',
    'children',
  ]);

  return (
    <div
      class={cn('px-2 pb-2 flex flex-col items-center', local.class)}
      {...others}
    >
      {local.children}
    </div>
  );
};
